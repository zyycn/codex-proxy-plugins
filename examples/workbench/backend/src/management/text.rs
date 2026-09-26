use super::{
    response::{ApiError, ApiResult, json_reply},
    validation::decode_json,
};
use crate::{
    evidence::{EvidenceInput, EvidenceLog},
    host_calls,
};
use gateway_plugin_sdk::{
    ErrorCode, PluginFault,
    call::{host::HttpRequest, management::ManagementRequest},
    client::{HostClient, TypedCall},
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

pub(super) async fn fetch(evidence: &EvidenceLog, call: TypedCall<ManagementRequest>) -> ApiResult {
    let request: FetchTextRequest = decode_json(&call)?;
    let result = fetch_text(&call.host, request).await?;
    evidence.record(EvidenceInput::passed("host_services", "text_fetched"));
    json_reply(&result)
}

fn network_error(error: PluginFault) -> ApiError {
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
    ApiError::new(status, code, message)
}

async fn fetch_text(
    host: &HostClient,
    request: FetchTextRequest,
) -> Result<FetchTextResponse, ApiError> {
    let mut url =
        Url::parse(&request.url).map_err(|_| ApiError::invalid("请提供完整的 HTTP(S) 地址"))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || request.url.len() > 4_096
    {
        return Err(ApiError::invalid("请提供不含凭据的完整 HTTP(S) 地址"));
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
    .map_err(network_error)?;
    let status = response.status;
    let content_type = response
        .headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-type"))
        .map(|(_, value)| value.clone());
    let stream = response
        .stream
        .ok_or_else(|| ApiError::new(502, "invalid_response", "宿主网络响应未提供数据流"))?;
    let source_error = if !(200..300).contains(&status) {
        Some(ApiError::new(
            502,
            "upstream_status",
            if (300..400).contains(&status) {
                "网页发生重定向，请使用跳转后的地址".to_owned()
            } else {
                format!("网页返回 HTTP {status}，请确认地址可访问")
            },
        ))
    } else if content_type.as_deref().is_some_and(|value| !textual(value)) {
        Some(ApiError::new(
            502,
            "invalid_response",
            "远程响应不是文本内容",
        ))
    } else {
        None
    };
    if let Some(error) = source_error {
        let _ = host_calls::close_http(host, stream).await;
        return Err(error);
    }
    let mut body = Vec::new();
    let mut truncated = false;
    loop {
        let (eof, chunk) = host_calls::read_http(host, stream.clone(), 64 * 1024)
            .await
            .map_err(network_error)?;
        body.extend_from_slice(&chunk);
        if body.len() > MAXIMUM_FETCH_BYTES {
            truncated = true;
            body.truncate(MAXIMUM_FETCH_BYTES);
            host_calls::close_http(host, stream)
                .await
                .map_err(network_error)?;
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

fn decode_text_prefix(mut body: Vec<u8>, truncated: bool) -> Result<String, ApiError> {
    if truncated
        && let Err(error) = std::str::from_utf8(&body)
        && error.error_len().is_none()
    {
        // 截断可能落在多字节字符中，只保留已确认有效的 UTF-8 前缀。
        body.truncate(error.valid_up_to());
    }
    String::from_utf8(body)
        .map_err(|_| ApiError::new(502, "invalid_response", "远程文本不是有效 UTF-8"))
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
