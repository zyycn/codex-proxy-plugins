use gateway_plugin_sdk::{Message, Stage};
use serde_json::{Value, json};

use crate::support::Peer;

#[tokio::test]
async fn imports_only_accept_fixed_demo_accounts_without_real_credentials() {
    let mut peer = Peer::start().await;
    let frame = peer
        .call(
            "provider.credentials.import",
            Stage::Management,
            json!({}),
            serde_json::to_vec(&json!({"demoAccount":"alpha"})).unwrap(),
        )
        .await;
    let value: Value = serde_json::from_slice(&frame.payload).unwrap();
    assert_eq!(value["accounts"][0]["upstream_account_id"], "alpha");
    for input in [
        json!({"token":"not-accepted"}),
        json!({"demoAccount":"unknown"}),
    ] {
        let frame = peer
            .call(
                "provider.credentials.import",
                Stage::Management,
                json!({}),
                serde_json::to_vec(&input).unwrap(),
            )
            .await;
        assert!(matches!(frame.message, Message::Error { .. }));
    }
}
