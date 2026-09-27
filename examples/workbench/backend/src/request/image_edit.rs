use base64::{Engine as _, engine::general_purpose::STANDARD};
use gateway_plugin_sdk::{
    PluginFault,
    call::middleware::{
        MiddlewareBodyFrame, MiddlewareBodyFraming, MiddlewareHeader, MiddlewareMount,
    },
    client::{MiddlewareBody, MiddlewareCall, MiddlewareRequest, MiddlewareResponse},
};
use serde_json::{Map, Value, json};

use crate::evidence::{EvidenceInput, EvidenceLog, EvidenceOutcome};

const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;
const MAX_TEXT_BYTES: usize = 32 * 1024;

fn content_type(request: &MiddlewareRequest) -> Option<&str> {
    request
        .head
        .headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case("content-type"))
        .and_then(|header| std::str::from_utf8(&header.value).ok())
}

pub(super) fn matches(request: &MiddlewareRequest) -> bool {
    request.head.mount == MiddlewareMount::Request
        && request.head.protocol == "openai"
        && request.head.endpoint == "/v1/images/edits"
        && request.head.body_visible
        && content_type(request).is_some_and(|value| {
            value
                .split(';')
                .next()
                .unwrap_or_default()
                .trim()
                .eq_ignore_ascii_case("multipart/form-data")
        })
}

pub(super) async fn handle(
    evidence: &EvidenceLog,
    mut call: MiddlewareCall,
) -> Result<MiddlewareResponse, PluginFault> {
    let body = match convert(
        content_type(&call.request).unwrap_or_default(),
        &call.request.body,
    )
    .await
    {
        Ok(body) => body,
        Err(message) => return Ok(bad_request(message)),
    };
    call.request.replace_body(body);
    call.request.remove_header("content-type");
    call.request
        .append_header("content-type", b"application/json".to_vec());
    // 长度、认证和账号头由宿主管理；插件只转换正文及其媒体类型。
    let result = call.next.run(call.request).await;
    let mut item = EvidenceInput::passed("middleware", "image_edit_multipart_adapted");
    item.request_id = call.context.request_id.as_deref();
    match &result {
        Ok(response) => {
            item.details
                .insert("responseStatus".to_owned(), json!(response.status));
            if response.status >= 400 {
                item.outcome = EvidenceOutcome::Failed;
            }
        }
        Err(_) => item.outcome = EvidenceOutcome::Failed,
    }
    evidence.record(item);
    result
}

async fn convert(content_type: &str, body: &[u8]) -> Result<Vec<u8>, &'static str> {
    if body.len() > MAX_BODY_BYTES {
        return Err("图片编辑示例最多接收 16 MiB multipart 正文");
    }
    let boundary = multer::parse_boundary(content_type).map_err(|_| "multipart boundary 无效")?;
    let mut multipart = multer::Multipart::with_reader(body, boundary);
    let mut document = Map::new();
    let mut images = Vec::new();
    let mut response_format_seen = false;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| "multipart 正文无效")?
    {
        let name = field.name().ok_or("multipart 字段缺少名称")?.to_owned();
        if matches!(name.as_str(), "image" | "image[]") {
            if images.len() >= 16 {
                return Err("图片编辑示例最多接收 16 张图片");
            }
            let mime = field
                .content_type()
                .map(|mime| mime.essence_str().to_owned())
                .ok_or("图片必须声明 image/png、image/jpeg 或 image/webp")?;
            if !matches!(mime.as_str(), "image/png" | "image/jpeg" | "image/webp") {
                return Err("图片必须声明 image/png、image/jpeg 或 image/webp");
            }
            let bytes = field.bytes().await.map_err(|_| "图片字段不完整")?;
            if bytes.is_empty() {
                return Err("图片内容不能为空");
            }
            images.push(
                json!({"image_url": format!("data:{mime};base64,{}", STANDARD.encode(bytes))}),
            );
            continue;
        }
        if !matches!(
            name.as_str(),
            "model" | "prompt" | "n" | "size" | "quality" | "background" | "response_format"
        ) {
            // 原生 Images JSON 尚未确认的字段不能静默丢弃，尤其是 mask。
            return Err(
                "图片编辑示例仅支持 image/image[]、model、prompt、n、size、quality、background 和 response_format=b64_json",
            );
        }
        if field.file_name().is_some() {
            return Err("图片之外的参数必须是文本字段");
        }
        let bytes = field.bytes().await.map_err(|_| "文本字段不完整")?;
        if bytes.len() > MAX_TEXT_BYTES {
            return Err("图片编辑文本字段不能超过 32 KiB");
        }
        let text = std::str::from_utf8(&bytes).map_err(|_| "文本字段必须是 UTF-8")?;
        if name == "response_format" {
            if response_format_seen || text != "b64_json" {
                return Err("仅支持单个 response_format=b64_json");
            }
            response_format_seen = true;
            continue;
        }
        let value = if name == "n" {
            let n = text
                .parse::<u64>()
                .ok()
                .filter(|n| *n > 0)
                .ok_or("n 必须是正整数")?;
            json!(n)
        } else {
            Value::String(text.to_owned())
        };
        if document.insert(name, value).is_some() {
            return Err("图片编辑文本参数不能重复");
        }
    }
    if images.is_empty() {
        return Err("至少上传一张 image 或 image[] 图片");
    }
    for required in ["model", "prompt"] {
        if document
            .get(required)
            .and_then(Value::as_str)
            .is_none_or(|value| value.trim().is_empty())
        {
            return Err("model 和 prompt 不能为空");
        }
    }
    document.insert("images".to_owned(), Value::Array(images));
    Ok(Value::Object(document).to_string().into_bytes())
}

fn bad_request(message: &str) -> MiddlewareResponse {
    let body = json!({"error":{"type":"invalid_request_error","message":message}})
        .to_string()
        .into_bytes();
    MiddlewareResponse::direct(
        "openai",
        400,
        vec![MiddlewareHeader {
            name: "content-type".to_owned(),
            value: b"application/json".to_vec(),
        }],
        MiddlewareBody::from_frames(
            MiddlewareBodyFraming::JsonDocument,
            vec![MiddlewareBodyFrame::new(body, true)],
        ),
    )
}
