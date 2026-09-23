use gateway_plugin_sdk::Stage;
use serde_json::{Value, json};

use crate::support::Peer;

#[tokio::test]
async fn ping_returns_a_result_without_creating_accounts() {
    let mut peer = Peer::start().await;
    let frame = peer
        .call(
            "command_line.execute",
            Stage::CommandLine,
            json!({}),
            serde_json::to_vec(&json!({"name":"ping","arguments":{}})).unwrap(),
        )
        .await;
    let result: Value = serde_json::from_slice(&frame.payload).unwrap();
    assert_eq!(result["exit_code"], 0);
    assert_eq!(result["accounts"], json!([]));
    assert!(result["stdout"].as_str().unwrap().contains("pong"));
}
