use gateway_plugin_sdk::Stage;
use serde_json::json;

use crate::support::{Peer, no_callback, result};

#[tokio::test]
async fn observer_receives_both_event_types_and_records_remain_bounded() {
    let mut peer = Peer::start().await;
    peer.mark_request().await;
    let frame = peer.call("observer.observe", Stage::Observation,
        json!({"event":"websocket_response","data":{"event_id":"ws","request_id":"test-request","config_revision":1,"operation":"generate",
            "protocol":"openai","provider":"openai","attempt_index":1,"sequence":1,"payload_included":false}}), Vec::new()).await;
    result(&frame);
    let snapshot = peer.snapshot().await;
    let status = snapshot["examples"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == "request-observation")
        .unwrap();
    assert_eq!(status["status"], "passed");
    assert_eq!(
        snapshot["facts"]["latest"]["observer"]["event"],
        "websocket_response"
    );
    let frame = peer
        .call(
            "observer.observe",
            Stage::Observation,
            json!({"event":"request_completed","data":{"event_id":"done","request_id":"test-request","config_revision":1,
                "operation":"generate","provider":"openai","completed_at_ms":1,
                "terminal":{"outcome":"succeeded","send_state":"sent","attempt_count":1},
                "usage":{"total_tokens":18}}}),
            Vec::new(),
        )
        .await;
    result(&frame);
    assert_eq!(
        peer.snapshot().await["facts"]["latest"]["observer"]["details"]["usage"]["total_tokens"],
        18
    );
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
        snapshot["facts"]["latest"]["observer"]["event"],
        "request_completed"
    );
}
