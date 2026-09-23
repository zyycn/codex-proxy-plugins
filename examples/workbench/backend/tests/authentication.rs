use gateway_plugin_sdk::Stage;
use serde_json::{Value, json};

use crate::support::Peer;

#[tokio::test]
async fn only_the_demo_identity_is_authenticated() {
    let mut peer = Peer::start().await;
    for (authorization, expected) in [
        ("CapabilityWorkbench demo", "authenticated"),
        ("CapabilityWorkbench wrong", "rejected"),
        ("Bearer unrelated", "not_matched"),
    ] {
        let frame = peer
            .call(
                "frontend_auth.authenticate",
                Stage::Authentication,
                json!({}),
                serde_json::to_vec(&json!({"authorization": authorization})).unwrap(),
            )
            .await;
        let result: Value = serde_json::from_slice(&frame.payload).unwrap();
        assert_eq!(result["outcome"], expected);
    }
}
