use super::scope::{ScopeTracker, body_has_scope_marker};
use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    PluginFault,
    call::policy::{ModelRouteDecision, ModelRouteRequest},
    client::{TypedCall, TypedReply},
};
use serde_json::json;

pub(crate) async fn route_model(
    evidence: &EvidenceLog,
    scope: &ScopeTracker,
    call: TypedCall<ModelRouteRequest>,
) -> Result<TypedReply<ModelRouteDecision>, PluginFault> {
    let marked = body_has_scope_marker(&call.payload);
    let scoped = marked || scope.contains(&call.request.request_id);
    if !scoped {
        return Ok(TypedReply::new(ModelRouteDecision::Unhandled));
    }
    if marked {
        scope.mark(&call.request.request_id);
    }
    // 平台候选还未按模型能力过滤；存在多个时交回宿主，避免误选不支持该模型的平台。
    let decision = match call.request.available_providers.as_slice() {
        [provider] => ModelRouteDecision::Route {
            provider: Some(provider.clone()),
            model: None,
        },
        _ => ModelRouteDecision::Unhandled,
    };
    let mut item = EvidenceInput::passed("model_router", "route_decided");
    item.request_id = Some(&call.request.request_id);
    item.model = Some(&call.request.model);
    item.details.insert(
        "decision".to_owned(),
        match &decision {
            ModelRouteDecision::Route { provider, model } => {
                json!({"kind":"route","provider":provider,"model":model})
            }
            ModelRouteDecision::Unhandled => json!({"kind":"unhandled"}),
            ModelRouteDecision::Reject => json!({"kind":"reject"}),
        },
    );
    evidence.record(item);
    Ok(TypedReply::new(decision))
}
