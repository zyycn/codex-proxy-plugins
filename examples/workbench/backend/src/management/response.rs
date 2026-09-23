use gateway_plugin_sdk::{
    ErrorCode, PluginFault, call::management::ManagementResponse, client::TypedReply,
};
use serde::Serialize;

pub(super) const JSON_CONTENT_TYPE: &str = "application/json";

#[derive(Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}
#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}
pub(super) fn host_error(
    error: PluginFault,
) -> Result<TypedReply<ManagementResponse>, PluginFault> {
    if error.code == ErrorCode::Conflict {
        api_error(409, "conflict", "记录已更新，请重新加载后再保存")
    } else {
        api_error(502, "host_callback", "宿主操作未完成")
    }
}

pub(super) fn json_reply(
    status: u16,
    body: &impl Serialize,
) -> Result<TypedReply<ManagementResponse>, PluginFault> {
    let payload = serde_json::to_vec(body)
        .map_err(|_| PluginFault::new(ErrorCode::Fault, "管理接口响应编码失败"))?;
    Ok(TypedReply::new(ManagementResponse {
        status,
        content_type: JSON_CONTENT_TYPE.to_owned(),
    })
    .with_payload(payload))
}

pub(super) fn api_error(
    status: u16,
    code: &'static str,
    message: impl Into<String>,
) -> Result<TypedReply<ManagementResponse>, PluginFault> {
    json_reply(
        status,
        &ErrorEnvelope {
            error: ErrorBody {
                code,
                message: message.into(),
            },
        },
    )
}

pub(super) fn encoding_error() -> Result<TypedReply<ManagementResponse>, PluginFault> {
    api_error(500, "encoding", "管理接口响应编码失败")
}
