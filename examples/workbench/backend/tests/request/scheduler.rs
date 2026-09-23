use gateway_plugin_sdk::Stage;
use serde_json::json;

use crate::support::{Peer, result};

#[tokio::test]
async fn unrelated_providers_are_delegated_and_demo_accounts_use_load_order() {
    let mut peer = Peer::start().await;
    for (provider, expected) in [
        ("openai", json!({"decision":"delegate"})),
        ("demo", json!({"decision":"pick","account_id":"idle"})),
    ] {
        let frame = peer.call("policy.schedule_account", Stage::Scheduling,
            json!({"request_id":"test-request","attempt_index":1,"provider":provider,"model":"demo-echo","candidates":[
                {"account_id":"busy","weight":100,"in_flight":1,"maximum_concurrency":2},
                {"account_id":"idle","weight":100,"in_flight":0,"maximum_concurrency":2}
            ]}), Vec::new()).await;
        assert_eq!(result(&frame), &expected);
    }
}
