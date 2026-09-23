use super::scope::{ScopeTracker, body_has_scope_marker};
use crate::evidence::{EvidenceInput, EvidenceLog};
use crate::provider::{AUTO_MODEL, ECHO_MODEL, PROVIDER_ID};
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
    let scoped = marked
        || scope.contains(&call.request.request_id)
        || matches!(call.request.model.as_str(), ECHO_MODEL | AUTO_MODEL);
    if !scoped {
        return Ok(TypedReply::new(ModelRouteDecision::Unhandled));
    }
    if marked {
        scope.mark(&call.request.request_id);
    }
    let decision = if call.request.model == AUTO_MODEL
        && (call.request.available_providers.is_empty()
            || call
                .request
                .available_providers
                .iter()
                .any(|provider| provider == PROVIDER_ID))
    {
        ModelRouteDecision::Route {
            provider: Some(PROVIDER_ID.to_owned()),
            model: Some(ECHO_MODEL.to_owned()),
        }
    } else {
        ModelRouteDecision::Unhandled
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
