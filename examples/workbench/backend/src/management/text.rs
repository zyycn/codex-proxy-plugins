use super::{
    response::{api_error, json_reply},
    validation::{decode_json, require_json},
};
use crate::evidence::EvidenceInput;
use crate::{evidence::EvidenceLog, host_calls};
use gateway_plugin_sdk::call::host::HttpRequest;
use gateway_plugin_sdk::{
    ErrorCode, PluginFault,
    call::management::{ManagementRequest, ManagementResponse},
    client::{TypedCall, TypedReply},
};
use serde::{Deserialize, Serialize};
use url::Url;

const MAXIMUM_FETCH_BYTES: usize = 256 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FetchTextRequest {
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct FetchTextResponse {
    url: String,
    status: u16,
    content_type: Option<String>,
    text: String,
    truncated: bool,
    bytes: usize,
}

enum FetchError {
    Input(&'static str),
    Host(PluginFault),
    Status(u16),
    Response(&'static str),
}

pub(super) async fn fetch(
    evidence: &EvidenceLog,
    call: TypedCall<ManagementRequest>,
) -> Result<TypedReply<ManagementResponse>, PluginFault> {
    if let Err(reply) = require_json(&call.request) {
        return reply;
    }
    let request = match decode_json::<FetchTextRequest>(&call.payload) {
        Ok(request) => request,
        Err(error) => return api_error(400, "invalid_request", error),
    };
    match fetch_text(&call.host, request).await {
        Ok(result) => {
            evidence.record(EvidenceInput::passed("host_services", "text_fetched"));
            json_reply(200, &result)
        }
        Err(FetchError::Input(message)) => api_error(400, "invalid_request", message),
        Err(FetchError::Host(error)) => network_error(error),
        Err(FetchError::Status(status)) => api_error(
            502,
            "upstream_status",
            if (300..400).contains(&status) {
                "网页发生重定向，请使用跳转后的地址".to_owned()
            } else {
                format!("网页返回 HTTP {status}，请确认地址可访问")
            },
        ),
        Err(FetchError::Response(message)) => api_error(502, "invalid_response", message),
    }
}

fn network_error(error: PluginFault) -> Result<TypedReply<ManagementResponse>, PluginFault> {
    let (status, code, message) = match error.code {
        // host.http 的业务错误只包含宿主生成的安全分类文案，不转发上游正文或底层错误。
        ErrorCode::Upstream | ErrorCode::PermissionDenied => (
            502,
            "network",
            if error.message == "managed HTTP operation failed" {
                "读取网页失败，请检查网络或代理".to_owned()
            } else if error.code == ErrorCode::PermissionDenied
                && error.message == "callback resource is not authorized"
            {
                "网页访问被拒绝，请检查插件网络权限".to_owned()
            } else {
                error.message
            },
        ),
        ErrorCode::Timeout => (504, "timeout", "读取网页超时，请稍后重试".to_owned()),
        ErrorCode::Cancelled => (499, "cancelled", "已取消读取网页".to_owned()),
        ErrorCode::Capacity => (503, "capacity", "网络请求繁忙，请稍后重试".to_owned()),
        ErrorCode::InvalidInput => (400, "invalid_request", "网页请求参数无效".to_owned()),
        _ => (502, "host_callback", "读取网页失败，请稍后重试".to_owned()),
    };
    api_error(status, code, message)
}

async fn fetch_text(
    host: &gateway_plugin_sdk::client::HostClient,
    request: FetchTextRequest,
) -> Result<FetchTextResponse, FetchError> {
    let mut url =
        Url::parse(&request.url).map_err(|_| FetchError::Input("请提供完整的 HTTP(S) 地址"))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || request.url.len() > 4_096
    {
        return Err(FetchError::Input("请提供不含凭据的完整 HTTP(S) 地址"));
    }
    // 页内定位不参与 HTTP 请求，移除后再交给宿主校验目标地址。
    url.set_fragment(None);
    let normalized_url = url.to_string();
    let response = host_calls::open_http(
        host,
        &HttpRequest {
            method: "GET".to_owned(),
            url: normalized_url.clone(),
            headers: vec![(
                "accept".to_owned(),
                "text/plain, text/html, application/json;q=0.8".to_owned(),
            )],
        },
    )
    .await
    .map_err(FetchError::Host)?;
    let status = response.status;
    let content_type = response
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.clone());
    let stream = response.stream;
    if !successful_source_status(status) {
        if let Some(stream) = stream {
            let _ = host_calls::close_http(host, stream).await;
        }
        return Err(FetchError::Status(status));
    }
    if content_type.as_deref().is_some_and(|value| !textual(value)) {
        if let Some(stream) = stream {
            let _ = host_calls::close_http(host, stream).await;
        }
        return Err(FetchError::Response("远程响应不是文本内容"));
    }
    let stream = stream.ok_or(FetchError::Response("宿主网络响应未提供数据流"))?;
    let mut body = Vec::new();
    let mut truncated = false;
    loop {
        let (eof, chunk) = host_calls::read_http(host, stream.clone(), 64 * 1024)
            .await
            .map_err(FetchError::Host)?;
        body.extend_from_slice(&chunk);
        if body.len() > MAXIMUM_FETCH_BYTES {
            truncated = true;
            body.truncate(MAXIMUM_FETCH_BYTES);
            host_calls::close_http(host, stream)
                .await
                .map_err(FetchError::Host)?;
            break;
        }
        if eof {
            break;
        }
    }
    let text = decode_text_prefix(body, truncated)?;
    Ok(FetchTextResponse {
        url: normalized_url,
        status: response.status,
        content_type,
        bytes: text.len(),
        text,
        truncated,
    })
}

fn decode_text_prefix(mut body: Vec<u8>, truncated: bool) -> Result<String, FetchError> {
    for _ in 0..=3 {
        match String::from_utf8(body) {
            Ok(text) => return Ok(text),
            Err(error) if truncated && error.utf8_error().error_len().is_none() => {
                body = error.into_bytes();
                body.pop();
            }
            Err(_) => return Err(FetchError::Response("远程文本不是有效 UTF-8")),
        }
    }
    Err(FetchError::Response("远程文本末尾包含无效 UTF-8"))
}

fn textual(content_type: &str) -> bool {
    let media_type = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    media_type.starts_with("text/")
        || matches!(
            media_type.as_str(),
            "application/json"
                | "application/ld+json"
                | "application/xml"
                | "application/xhtml+xml"
                | "application/javascript"
        )
        || media_type.ends_with("+json")
        || media_type.ends_with("+xml")
}

fn successful_source_status(status: u16) -> bool {
    (200..300).contains(&status)
}
