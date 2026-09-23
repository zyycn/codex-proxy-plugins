use gateway_plugin_sdk::{ErrorCode, Message, Stage};
use serde_json::json;

use super::input;
use crate::support::{Peer, result};

#[tokio::test]
async fn prepare_rejects_empty_text_and_bounds_pending_executions() {
    let mut peer = Peer::start().await;
    let empty = peer
        .call(
            "provider.prepare",
            Stage::Attempt,
            json!({}),
            input(json!({"input":[]})),
        )
        .await;
    assert!(
        matches!(empty.message, Message::Error { error, .. } if error.code == ErrorCode::InvalidInput)
    );
    for _ in 0..32 {
        let frame = peer
            .call(
                "provider.prepare",
                Stage::Attempt,
                json!({}),
                input(json!({"input":"示例"})),
            )
            .await;
        result(&frame);
    }
    let full = peer
        .call(
            "provider.prepare",
            Stage::Attempt,
            json!({}),
            input(json!({"input":"示例"})),
        )
        .await;
    assert!(
        matches!(full.message, Message::Error { error, .. } if error.code == ErrorCode::Capacity)
    );
    let discarded = peer
        .call(
            "provider.discard",
            Stage::Attempt,
            json!({"token":"echo-1"}),
            Vec::new(),
        )
        .await;
    result(&discarded);
    let reusable = peer
        .call(
            "provider.prepare",
            Stage::Attempt,
            json!({}),
            input(json!({"input":"示例"})),
        )
        .await;
    result(&reusable);
}
