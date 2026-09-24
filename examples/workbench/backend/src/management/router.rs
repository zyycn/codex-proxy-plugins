use super::{models, response::api_error, tasks, text, workbench};
use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    PluginFault,
    call::management::{ManagementRequest, ManagementResponse},
    client::{TypedCall, TypedReply},
};
use serde_json::json;
use std::sync::Arc;

const MAXIMUM_BODY_BYTES: usize = 512 * 1024;

pub(crate) async fn handle(
    evidence: Arc<EvidenceLog>,
    call: TypedCall<ManagementRequest>,
) -> Result<TypedReply<ManagementResponse>, PluginFault> {
    if call.request.path != "api/snapshot" {
        let route = format!("{} {}", call.request.method, call.request.path);
        let mut management = EvidenceInput::passed("management", "route_called");
        management.details.insert("route".to_owned(), json!(route));
        evidence.record(management);
    }
    if !call.request.query.is_empty() {
        return api_error(400, "invalid_request", "此接口不支持查询参数");
    }
    if call.payload.len() > MAXIMUM_BODY_BYTES {
        return api_error(400, "invalid_request", "请求正文超过工作台限制");
    }
    match (call.request.method.as_str(), call.request.path.as_str()) {
        ("GET", "api/snapshot") => workbench::snapshot(&evidence, call).await,
        ("POST", "api/echo") => workbench::echo(&evidence, call).await,
        ("POST", "api/models") => models::list(&evidence, call).await,
        ("GET", "api/tasks") => tasks::get(&evidence, call).await,
        ("POST", "api/tasks") => tasks::save(&evidence, call).await,
        ("POST", "api/fetch-text") => text::fetch(&evidence, call).await,
        ("POST", "api/log") => workbench::log(&evidence, call).await,
        _ => api_error(404, "not_found", "未找到插件管理接口"),
    }
}
