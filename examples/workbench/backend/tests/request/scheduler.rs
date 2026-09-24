use gateway_plugin_sdk::Stage;
use serde_json::json;

use crate::support::{Peer, result};

#[tokio::test]
async fn routing_does_not_guess_a_provider_from_multiple_candidates() {
    let mut peer = Peer::start().await;
    let frame = peer.call(
        "policy.route_model",
        Stage::Routing,
        json!({"request_id":"test-request","operation":"generate","protocol":"openai","model":"grok-test","available_providers":["openai","xai"]}),
        serde_json::to_vec(&json!({"metadata":{"capability_workbench":"true"}})).unwrap(),
    ).await;
    assert_eq!(result(&frame), &json!({"decision":"unhandled"}));
}

#[tokio::test]
async fn unmarked_requests_are_delegated_and_marked_requests_use_load_order() {
    let mut peer = Peer::start().await;
    for (marked, expected) in [
        (false, json!({"decision":"delegate"})),
        (true, json!({"decision":"pick","account_id":"idle"})),
    ] {
        if marked {
            peer.mark_request().await;
        }
        let frame = peer.call("policy.schedule_account", Stage::Scheduling,
            json!({"request_id":"test-request","attempt_index":1,"provider":"openai","model":"test-model","candidates":[
                {"account_id":"busy","weight":100,"in_flight":1,"maximum_concurrency":2},
                {"account_id":"idle","weight":100,"in_flight":0,"maximum_concurrency":2}
            ]}), Vec::new()).await;
        assert_eq!(result(&frame), &expected);
    }
}
