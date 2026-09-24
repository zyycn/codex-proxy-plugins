use super::scope::ScopeTracker;
use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    PluginFault,
    call::{observation::ObserveWebSocketResponse, policy::ObserveRequest},
    client::{Empty, TypedCall, TypedReply},
};
use serde_json::{Map, json};

pub(crate) async fn observe_request(
    evidence: &EvidenceLog,
    scope: &ScopeTracker,
    call: TypedCall<ObserveRequest>,
) -> Result<TypedReply<Empty>, PluginFault> {
    let marked = scope.take(&call.request.request_id);
    if !marked {
        return Ok(TypedReply::new(Empty {}));
    }
    let mut details = Map::new();
    details.insert("eventId".to_owned(), json!(call.request.event_id));
    details.insert("operation".to_owned(), json!(call.request.operation));
    details.insert(
        "completedAtMs".to_owned(),
        json!(call.request.completed_at_ms),
    );
    if let Some(terminal) = &call.request.terminal {
        details.insert("terminal".to_owned(), json!(terminal));
    }
    if let Some(usage) = &call.request.usage {
        details.insert("usage".to_owned(), json!(usage));
    }
    for capability in ["request_lifecycle", "usage"] {
        let mut item = EvidenceInput::passed(capability, "request_observed");
        item.request_id = Some(&call.request.request_id);
        item.provider = call.request.provider.as_deref();
        item.account_id = call.request.account_id.as_deref();
        item.model = call
            .request
            .upstream_model
            .as_deref()
            .or(call.request.requested_model.as_deref());
        item.details = details.clone();
        evidence.record(item);
    }
    Ok(TypedReply::new(Empty {}))
}

pub(crate) async fn observe_websocket(
    evidence: &EvidenceLog,
    scope: &ScopeTracker,
    call: TypedCall<ObserveWebSocketResponse>,
) -> Result<TypedReply<Empty>, PluginFault> {
    if !scope.contains(&call.request.request_id) {
        return Ok(TypedReply::new(Empty {}));
    }
    let mut item = EvidenceInput::passed("web_socket_observer", "response_event_observed");
    item.request_id = Some(&call.request.request_id);
    item.provider = Some(&call.request.provider);
    item.account_id = call.request.account_id.as_deref();
    item.model = call.request.requested_model.as_deref();
    item.details
        .insert("eventId".to_owned(), json!(call.request.event_id));
    item.details
        .insert("sequence".to_owned(), json!(call.request.sequence));
    item.details.insert(
        "payloadIncluded".to_owned(),
        json!(call.request.payload_included),
    );
    if let Some(event_type) = call.request.event_type {
        item.details
            .insert("eventType".to_owned(), json!(event_type));
    }
    evidence.record(item);
    Ok(TypedReply::new(Empty {}))
}
