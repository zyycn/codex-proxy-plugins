use super::response::{ApiError, JSON_CONTENT_TYPE};
use gateway_plugin_sdk::{call::management::ManagementRequest, client::TypedCall};
use serde::Deserialize;
use serde_json::{Map, Value};
use url::Url;

pub(super) fn require_empty_body(
    request: &ManagementRequest,
    payload: &[u8],
) -> Result<(), ApiError> {
    if request.content_type.is_none() && payload.is_empty() {
        Ok(())
    } else {
        Err(ApiError::invalid("此接口不接受请求正文"))
    }
}

pub(super) fn require_empty_json(
    request: &ManagementRequest,
    payload: &[u8],
) -> Result<(), ApiError> {
    require_json(request)?;
    if serde_json::from_slice::<Map<String, Value>>(payload).is_ok_and(|value| value.is_empty()) {
        Ok(())
    } else {
        Err(ApiError::invalid("此接口需要空 JSON 对象"))
    }
}

pub(super) fn require_json(request: &ManagementRequest) -> Result<(), ApiError> {
    if request
        .content_type
        .as_deref()
        .and_then(|content_type| content_type.split(';').next())
        .is_some_and(|content_type| content_type.trim().eq_ignore_ascii_case(JSON_CONTENT_TYPE))
    {
        Ok(())
    } else {
        Err(ApiError::new(
            400,
            "invalid_content_type",
            "此接口需要 application/json 内容类型",
        ))
    }
}

pub(super) fn decode_json<T: for<'de> Deserialize<'de>>(
    call: &TypedCall<ManagementRequest>,
) -> Result<T, ApiError> {
    require_json(&call.request)?;
    serde_json::from_slice(&call.payload).map_err(|_| ApiError::invalid("请求正文不是有效 JSON"))
}

pub(super) fn valid_source_url(value: &str) -> bool {
    bounded(value, 1, 4_096)
        && Url::parse(value).is_ok_and(|url| {
            matches!(url.scheme(), "http" | "https")
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
        })
}

pub(super) fn bounded(value: &str, minimum: usize, maximum: usize) -> bool {
    (minimum..=maximum).contains(&value.len()) && !value.chars().any(char::is_control)
}

pub(super) fn bounded_text(value: &str, maximum: usize) -> bool {
    // 正文保留段落与缩进；标识符仍使用不允许控制字符的校验。
    value.len() <= maximum
        && !value
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
}
