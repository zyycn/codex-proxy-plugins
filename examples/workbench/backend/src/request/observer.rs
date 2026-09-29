use super::scope::ScopeTracker;
use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    PluginFault,
    call::observation::Event,
    client::{Empty, TypedCall, TypedReply},
};
use serde_json::json;

pub(crate) async fn observe(
    evidence: &EvidenceLog,
    scope: &ScopeTracker,
    call: TypedCall<Event>,
) -> Result<TypedReply<Empty>, PluginFault> {
    match call.request {
        Event::RequestCompleted(request) => {
            if scope.take(&request.request_id) {
                let mut item = EvidenceInput::passed("observer", "request_completed");
                item.request_id = Some(&request.request_id);
                item.provider = request.provider.as_deref();
                item.account_id = request.account_id.as_deref();
                item.model = request
                    .upstream_model
                    .as_deref()
                    .or(request.requested_model.as_deref());
                item.details
                    .insert("eventId".to_owned(), json!(request.event_id));
                item.details
                    .insert("operation".to_owned(), json!(request.operation));
                item.details
                    .insert("completedAtMs".to_owned(), json!(request.completed_at_ms));
                item.details
                    .insert("terminal".to_owned(), json!(request.terminal));
                item.details
                    .insert("usage".to_owned(), json!(request.usage));
                evidence.record(item);
            }
        }
        Event::WebSocketResponse(request) => {
            if scope.contains(&request.request_id) {
                let mut item = EvidenceInput::passed("observer", "websocket_response");
                item.request_id = Some(&request.request_id);
                item.provider = Some(&request.provider);
                item.account_id = request.account_id.as_deref();
                item.model = request.requested_model.as_deref();
                item.details
                    .insert("eventId".to_owned(), json!(request.event_id));
                item.details
                    .insert("sequence".to_owned(), json!(request.sequence));
                item.details.insert(
                    "payloadIncluded".to_owned(),
                    json!(request.payload_included),
                );
                if let Some(event_type) = request.event_type {
                    item.details
                        .insert("eventType".to_owned(), json!(event_type));
                }
                evidence.record(item);
            }
        }
    }
    Ok(TypedReply::new(Empty {}))
}
