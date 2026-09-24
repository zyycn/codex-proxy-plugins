use gateway_plugin_sdk::Stage;
use serde_json::json;

use crate::support::{Peer, no_callback, result};

#[tokio::test]
async fn observation_requires_all_capabilities_and_records_remain_bounded() {
    let mut peer = Peer::start().await;
    peer.mark_request().await;
    let frame = peer.call("websocket.response_event", Stage::Observation,
        json!({"event_id":"ws","request_id":"test-request","config_revision":1,"operation":"generate",
            "protocol":"openai","provider":"openai","attempt_index":1,"sequence":1,"payload_included":false}), Vec::new()).await;
    result(&frame);
    let snapshot = peer.snapshot().await;
    let status = snapshot["examples"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "request-observation")
        .unwrap();
    assert_eq!(status["status"], "pending");
    let frame = peer
        .call(
            "policy.observe_request",
            Stage::Observation,
            json!({}),
            serde_json::to_vec(
                &json!({"event_id":"done","request_id":"test-request","config_revision":1,
            "operation":"generate","provider":"openai","completed_at_ms":1}),
            )
            .unwrap(),
        )
        .await;
    result(&frame);
    for _ in 0..66 {
        peer.api(
            "POST",
            "api/echo",
            Some(json!({"message":"示例"})),
            no_callback,
        )
        .await;
    }
    let snapshot = peer.snapshot().await;
    let status = snapshot["examples"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "request-observation")
        .unwrap();
    assert_eq!(status["status"], "passed");
    assert_eq!(snapshot["facts"]["entries"].as_array().unwrap().len(), 64);
    assert_eq!(
        snapshot["facts"]["latest"]["web_socket_observer"]["event"],
        "response_event_observed"
    );
}
