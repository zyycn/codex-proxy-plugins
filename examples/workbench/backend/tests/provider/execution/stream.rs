use gateway_plugin_sdk::{
    Message, Stage,
    call::provider::{ExecutionEvent, WirePayload},
};
use serde_json::json;

use super::input;
use crate::support::{Peer, result};

#[tokio::test]
async fn stream_preserves_text_with_a_bounded_number_of_chunks() {
    let mut peer = Peer::start().await;
    let text = "示例".repeat(1000);
    let prepared = peer
        .call(
            "provider.prepare",
            Stage::Attempt,
            json!({}),
            input(json!({"input":text})),
        )
        .await;
    let token = result(&prepared)["token"].clone();
    let started = peer
        .call(
            "provider.execute",
            Stage::Execution,
            json!({"token":token}),
            Vec::new(),
        )
        .await;
    let Message::Result { .. } = started.message else {
        panic!("执行未启动")
    };
    let mut output = String::new();
    let mut chunks = 0;
    loop {
        let frame = peer.receive().await;
        match frame.message {
            Message::Stream { .. } => {
                let event = ExecutionEvent::decode(&frame.payload).unwrap();
                if let Some(wire) = event.wire
                    && let WirePayload::Json { data, .. } = wire.payload
                    && data["type"] == "response.output_text.delta"
                {
                    chunks += 1;
                    output.push_str(data["delta"].as_str().unwrap());
                }
            }
            Message::End { error, .. } => {
                assert!(error.is_none());
                break;
            }
            _ => panic!("出现未预期的流消息：{frame:?}"),
        }
    }
    assert_eq!(output, text);
    assert!(chunks <= 48);
}
