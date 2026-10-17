//! 通过真实插件子进程验证入口透传、控制消息确认及 RPC 资源和失败行为

mod support;

use gateway_plugin_sdk::{ErrorCode, Message, PluginFault, Stage};
use serde_json::{Value, json};
use support::{Peer, SCOPE_HEADER, http_call, websocket_call};

#[tokio::test]
async fn registration_matches_the_normalized_author_manifest() {
    let mut peer = Peer::start().await;
    peer.call(1, Stage::Registration, json!({}), b"").await;
    let manifest = codex_proxy_plugin_omp_compat::manifest().unwrap();
    assert_eq!(
        peer.result(1).await.0,
        json!({"contributes":manifest.contributes})
    );
    peer.shutdown().await;
}

#[tokio::test]
async fn aliases_preserve_query_body_headers_and_host_authentication_result() {
    let mut peer = Peer::start().await;
    for (index, method) in ["GET", "POST"].into_iter().enumerate() {
        let id = 1 + index as u64 * 2;
        let mut input = http_call(method, "/v1/codex/responses?trace=a%2Fb&x=1&x=2");
        input["request"]["body"] = json!({"kind":"handle","handle":"upload"});
        input["request"]["headers"] = json!([
            {"name":"authorization","value":b"Bearer fixture-key"},
            {"name":"x-many","value":b"first"},
            {"name":"x-many","value":b"second"},
            {"name":"X-Codex-Proxy-Omp-Compat","value":b"omp-test"}
        ]);
        let mut expected = input["request"].clone();
        expected["uri"] = json!("/v1/responses?trace=a%2Fb&x=1&x=2");
        let headers = expected["headers"].as_array_mut().unwrap();
        headers.pop();
        if method == "GET" {
            headers.push(json!({"name":SCOPE_HEADER,"value":b"omp-test"}));
            headers.push(json!({"name":"connection","value":SCOPE_HEADER.as_bytes()}));
        }
        peer.call(id, Stage::Http, input, b"").await;
        let callback = peer.callback(id, "host.middleware.next").await;
        assert_eq!(callback.params, expected);
        assert!(callback.payload.is_empty(), "不能提前读取上传正文");
        let response = json!({
            "status":401,"version":"HTTP/1.1","headers":[],
            "body":{"kind":"handle","handle":"auth-error"},
            "response":"original-response","session":false
        });
        peer.reply(callback.id, response.clone(), b"").await;
        let result = peer.http_result(id).await;
        assert_eq!(result.0, response);
        assert!(result.1.is_empty());
    }
    peer.shutdown().await;
}

#[tokio::test]
async fn other_paths_and_methods_do_not_enable_steering_even_with_a_forged_header() {
    let mut peer = Peer::start().await;
    for (index, (method, uri)) in [
        ("GET", "/v1/responses"),
        ("POST", "/v1/codex/responses/"),
        ("GET", "/v1/codex/responses-extra"),
        ("GET", "/v1/models"),
        ("DELETE", "/v1/codex/responses"),
    ]
    .into_iter()
    .enumerate()
    {
        let id = 1 + index as u64 * 2;
        let mut input = http_call(method, uri);
        input["request"]["headers"] = json!([{"name":SCOPE_HEADER,"value":b"omp-test"}]);
        let mut expected = input["request"].clone();
        expected["headers"] = json!([]);
        peer.call(id, Stage::Http, input, b"").await;
        let callback = peer.callback(id, "host.middleware.next").await;
        assert_eq!(callback.params, expected);
        assert!(callback.payload.is_empty());
        let response = support::http_response();
        peer.reply(callback.id, response.clone(), b"").await;
        assert_eq!(peer.http_result(id).await, (response, vec![]));
    }
    peer.shutdown().await;
}

#[tokio::test]
async fn steering_is_rejected_while_response_output_is_in_flight_and_input_is_not_replayed() {
    let mut peer = Peer::start().await;
    // 暂停一个响应输出，控制消息仍能通过独立的入站调用确认
    let completed = b"{\"type\":\"response.completed\",\"response\":{\"id\":\"resp_active\"}}";
    let mut outgoing = websocket_call("outgoing", "text", true);
    outgoing["message"]["payload"] = json!({"kind":"bytes"});
    peer.call(1, Stage::WebSocket, outgoing, completed).await;
    let active = peer.callback(1, "host.middleware.next").await;

    let payload = br#"{"type":"response.steer","previous_response_id":"resp_active\nopaque","input":[{"role":"user","content":"queued input"}]}"#;
    peer.call(
        3,
        Stage::WebSocket,
        websocket_call("incoming", "text", true),
        payload,
    )
    .await;
    let ack = peer.callback(3, "host.middleware.send").await;
    let ack_body: Value = serde_json::from_slice(&ack.payload).unwrap();
    assert_eq!(ack_body["type"], "response.steer.failed");
    assert_eq!(
        ack_body["steer"]["previous_response_id"],
        "resp_active\nopaque"
    );
    assert_eq!(ack_body["error"]["code"], "unsupported_steering");
    assert_eq!(
        ack.params,
        json!({"kind":{"kind":"text"},"payload":{"kind":"bytes"}})
    );
    peer.reply(ack.id, json!({}), b"").await;
    assert_eq!(peer.result(3).await, (Value::Null, vec![]));

    peer.reply(active.id, active.params, &active.payload).await;
    assert_eq!(peer.result(1).await.1, completed);

    let next = br#"{ "type": "response.create", "previous_response_id": "resp_active", "input": [{"role":"user","content":"queued input"}] }"#;
    peer.call(
        5,
        Stage::WebSocket,
        websocket_call("incoming", "text", true),
        next,
    )
    .await;
    let callback = peer.callback(5, "host.middleware.next").await;
    assert_eq!(callback.payload, next);
    peer.reply(callback.id, callback.params, &callback.payload)
        .await;
    assert_eq!(peer.result(5).await.1, next);
    peer.shutdown().await;
}

#[tokio::test]
async fn scoped_text_payload_is_read_in_chunks_without_changing_non_steering_bytes() {
    let mut peer = Peer::start().await;
    let mut input = websocket_call("incoming", "text", true);
    input["message"]["payload"] = json!({"kind":"handle","handle":"text-body"});
    peer.call(1, Stage::WebSocket, input, b"").await;
    let chunks: [&[u8]; 3] = [
        b"{ \"type\":",
        b" \"response.create\",",
        b" \"input\": [] }",
    ];
    for chunk in chunks {
        let callback = peer.callback(1, "host.middleware.body_read").await;
        assert_eq!(callback.params["handle"], "text-body");
        peer.reply(callback.id, json!({"eof":false}), chunk).await;
    }
    let callback = peer.callback(1, "host.middleware.body_read").await;
    peer.reply(callback.id, json!({"eof":true}), b"").await;
    let callback = peer.callback(1, "host.middleware.next").await;
    assert_eq!(callback.payload, chunks.concat());
    peer.reply(callback.id, callback.params, &callback.payload)
        .await;
    assert_eq!(peer.result(1).await.1, chunks.concat());
    peer.shutdown().await;
}

#[tokio::test]
async fn another_instance_scope_survives_http_composition_without_enabling_this_instance() {
    let mut peer = Peer::start().await;
    let other = json!({"name":SCOPE_HEADER,"value":b"another-instance"});
    let mut input = http_call("GET", "/v1/responses");
    input["request"]["headers"] = json!([other, {"name":SCOPE_HEADER,"value":b"omp-test"}]);
    peer.call(1, Stage::Http, input, b"").await;
    let callback = peer.callback(1, "host.middleware.next").await;
    assert_eq!(callback.params["headers"], json!([other]));
    peer.reply(callback.id, support::http_response(), b"").await;
    peer.http_result(1).await;
    let mut input = websocket_call("incoming", "text", false);
    input["headers"] = json!([other]);
    input["message"]["payload"] = json!({"kind":"handle","handle":"other-instance-body"});
    let expected = input["message"].clone();
    peer.call(3, Stage::WebSocket, input, b"").await;
    let callback = peer.callback(3, "host.middleware.next").await;
    assert_eq!(callback.params, expected);
    peer.reply(callback.id, expected, b"").await;
    peer.result(3).await;
    peer.shutdown().await;
}

#[tokio::test]
async fn unscoped_binary_and_outgoing_messages_keep_lazy_payload_handles() {
    let mut peer = Peer::start().await;
    for (index, (direction, kind, scoped)) in [
        ("incoming", "text", false),
        ("incoming", "binary", true),
        ("outgoing", "text", true),
        ("incoming", "ping", true),
        ("incoming", "pong", true),
    ]
    .into_iter()
    .enumerate()
    {
        let id = 1 + index as u64 * 2;
        let mut input = websocket_call(direction, kind, scoped);
        input["message"]["payload"] = json!({"kind":"handle","handle":"unread"});
        let expected = input["message"].clone();
        peer.call(id, Stage::WebSocket, input, b"").await;
        let callback = peer.callback(id, "host.middleware.next").await;
        assert_eq!(callback.params, expected);
        peer.reply(callback.id, expected.clone(), b"").await;
        assert_eq!(peer.result(id).await.0, expected);
    }
    peer.shutdown().await;
}

#[tokio::test]
async fn malformed_and_unknown_text_frames_reach_the_original_decoder_unchanged() {
    let mut peer = Peer::start().await;
    for (index, payload) in [
        b"not-json".as_slice(),
        br#"{"type":"response.steer","previous_response_id":42}"#,
        br#"{"type":"response.interrupt","response_id":"resp_1"}"#,
        br#"{"type":"future.control","input":[1,2]}"#,
    ]
    .into_iter()
    .enumerate()
    {
        let id = 1 + index as u64 * 2;
        peer.call(
            id,
            Stage::WebSocket,
            websocket_call("incoming", "text", true),
            payload,
        )
        .await;
        let callback = peer.callback(id, "host.middleware.next").await;
        assert_eq!(callback.payload, payload);
        peer.reply(callback.id, callback.params, &callback.payload)
            .await;
        assert_eq!(peer.result(id).await.1, payload);
    }
    peer.shutdown().await;
}

#[tokio::test]
async fn failed_ack_write_is_reported_without_forwarding_or_retrying_the_control_message() {
    let mut peer = Peer::start().await;
    peer.call(
        1,
        Stage::WebSocket,
        websocket_call("incoming", "text", true),
        br#"{"type":"response.steer","previous_response_id":"resp_1","input":[]}"#,
    )
    .await;
    let callback = peer.callback(1, "host.middleware.send").await;
    peer.error(
        callback.id,
        PluginFault::new(ErrorCode::Fault, "writer closed"),
    )
    .await;
    let frame = peer.receive().await;
    assert!(
        matches!(frame.message, Message::Error { id: 1, error } if error.code == ErrorCode::Fault)
    );
    peer.shutdown().await;
}

#[tokio::test]
async fn cancellation_during_payload_read_does_not_execute_queued_input() {
    let mut peer = Peer::start().await;
    let mut input = websocket_call("incoming", "text", true);
    input["message"]["payload"] = json!({"kind":"handle","handle":"pending"});
    peer.call(1, Stage::WebSocket, input, b"").await;
    let _pending = peer.callback(1, "host.middleware.body_read").await;
    peer.control(Message::Cancel { id: 1 }).await;
    let frame = peer.receive().await;
    assert!(matches!(frame.message, Message::Cancelled { id: 1 }));
    peer.shutdown().await;
}
