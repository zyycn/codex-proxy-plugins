use super::{
    response::{ApiError, ApiResult},
    tasks, text, workbench,
};
use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    PluginFault,
    call::management::{ManagementRequest, ManagementResponse},
    client::{TypedCall, TypedReply},
};
use serde_json::json;

const MAXIMUM_BODY_BYTES: usize = 512 * 1024;

pub(crate) async fn handle(
    evidence: &EvidenceLog,
    call: TypedCall<ManagementRequest>,
) -> Result<TypedReply<ManagementResponse>, PluginFault> {
    route(evidence, call).await.or_else(ApiError::into_reply)
}

async fn route(evidence: &EvidenceLog, call: TypedCall<ManagementRequest>) -> ApiResult {
    if call.request.path != "api/snapshot" {
        let route = format!("{} {}", call.request.method, call.request.path);
        let mut management = EvidenceInput::passed("management", "route_called");
        management.details.insert("route".to_owned(), json!(route));
        evidence.record(management);
    }
    if !call.request.query.is_empty() {
        return Err(ApiError::invalid("此接口不支持查询参数"));
    }
    if call.payload.len() > MAXIMUM_BODY_BYTES {
        return Err(ApiError::invalid("请求正文超过工作台限制"));
    }
    match (call.request.method.as_str(), call.request.path.as_str()) {
        ("GET", "api/snapshot") => workbench::snapshot(evidence, call).await,
        ("POST", "api/echo") => workbench::echo(call),
        ("POST", "api/models") => workbench::list_models(call).await,
        ("GET", "api/tasks") => tasks::get(call).await,
        ("POST", "api/tasks") => tasks::save(call).await,
        ("POST", "api/fetch-text") => text::fetch(evidence, call).await,
        ("POST", "api/log") => workbench::log(evidence, call).await,
        _ => Err(ApiError::new(404, "not_found", "未找到插件管理接口")),
    }
}
