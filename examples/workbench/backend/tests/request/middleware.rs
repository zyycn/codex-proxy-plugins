use gateway_plugin_sdk::{ErrorCode, Message, PluginFault, Stage};
use serde_json::{Value, json};

use crate::support::{Peer, result};

#[tokio::test]
async fn middleware_consumes_only_demo_metadata_and_preserves_attempt_input() {
    for (stage, mount, uppercase, expected) in [
        (Stage::Request, "request", false, "Hello"),
        (Stage::Request, "request", true, "HELLO"),
        (Stage::Attempt, "attempt", true, "Hello"),
    ] {
        let mut peer = Peer::start().await;
        let input = json!({"input":"Hello","metadata":{
            "capability_workbench":"true",
            "capability_workbench_uppercase":if uppercase { "true" } else { "false" },
            "customer":"keep-me"
        }});
        let frame = peer.call_with("middleware.handle", stage,
            json!({"request_id":"test-request","mount":mount,"operation":"generate","protocol":"openai",
                "endpoint":"responses","transport":"http_sse","model":"test-model","headers":[],"body_visible":true,
                "attempt_index":if mount == "attempt" { Some(1) } else { None }}),
            serde_json::to_vec(&input).unwrap(), |method, params, payload| {
                assert_eq!(method, "host.middleware.next");
                if mount == "request" {
                    assert_eq!(serde_json::from_slice::<Value>(payload).unwrap(),
                        json!({"input":expected,"metadata":{"customer":"keep-me"}}));
                } else {
                    assert_eq!(params["body"], "preserve");
                    assert!(payload.is_empty());
                }
                Ok((json!({"response":"response-1","protocol":"openai","status":503,"headers":[]}), Vec::new()))
            }).await;
        // SDK 返回相对 next 的增量；省略 status 表示保留上游的 503。
        assert!(result(&frame).get("status").is_none());
        assert_eq!(result(&frame)["response"], "response-1");
        assert_eq!(
            result(&frame)["header_mutations"].as_array().unwrap().len(),
            usize::from(mount == "request")
        );
        assert!(matches!(peer.receive().await.message, Message::End { .. }));
        let snapshot = peer.snapshot().await;
        let evidence = &snapshot["facts"]["latest"]["middleware"];
        assert_eq!(evidence["outcome"], "failed");
        assert_eq!(evidence["details"]["responseStatus"], 503);
        assert_eq!(
            evidence["details"]["uppercased"],
            uppercase && mount == "request"
        );
    }
}

#[tokio::test]
async fn middleware_propagates_downstream_fault_and_records_failure() {
    let mut peer = Peer::start().await;
    peer.mark_request().await;
    let frame = peer.call_with("middleware.handle", Stage::Request,
        json!({"request_id":"test-request","mount":"request","operation":"generate","protocol":"openai",
            "endpoint":"responses","transport":"http_sse","headers":[],"body_visible":true}),
        br#"{"input":"Hello"}"#.to_vec(), |method, params, payload| {
            assert_eq!(method, "host.middleware.next");
            assert_eq!(params["body"], "preserve");
            assert!(payload.is_empty());
            Err(PluginFault::new(ErrorCode::Cancelled, "request cancelled"))
        }).await;
    assert!(
        matches!(frame.message, Message::Error { error, .. } if error.code == ErrorCode::Cancelled)
    );
    let snapshot = peer.snapshot().await;
    assert_eq!(
        snapshot["facts"]["latest"]["middleware"]["event"],
        "downstream_failed"
    );
    assert_eq!(
        snapshot["facts"]["latest"]["middleware"]["outcome"],
        "failed"
    );
}

#[tokio::test]
async fn middleware_preserves_unrelated_requests_and_transforms_only_marked_text() {
    for (metadata, transformed) in [json!({}), json!({"capability_workbench":true})]
        .map(|metadata| (metadata, false))
        .into_iter()
        .chain([(
            json!({"capability_workbench":"true","capability_workbench_uppercase":"true"}),
            true,
        )])
    {
        let mut peer = Peer::start().await;
        let frame = peer.call_with("middleware.handle", Stage::Request,
            json!({"request_id":"test-request","mount":"request","operation":"generate","protocol":"openai",
                "endpoint":"responses","transport":"http_sse","model":"ordinary-model","headers":[],"body_visible":true}),
            serde_json::to_vec(&json!({"input":[{"role":"user","content":[{"type":"input_text","text":"Hello"}]},{"unrelated":"leave me"}],"metadata":metadata})).unwrap(),
            |method, params, payload| {
                assert_eq!(method, "host.middleware.next");
                if transformed {
                    let body: Value = serde_json::from_slice(payload).unwrap();
                    assert_eq!(body["input"][0]["content"][0]["text"], "HELLO");
                    assert_eq!(body["input"][1]["unrelated"], "leave me");
                    assert!(body.get("metadata").is_none());
                } else {
                    assert_eq!(params["body"], "preserve");
                    assert!(payload.is_empty());
                    assert!(params.get("header_mutations").is_none_or(|value| value.as_array().unwrap().is_empty()));
                }
                Ok((json!({"response":"response-1","protocol":"openai","status":200,"headers":[]}), Vec::new()))
            }).await;
        let headers = result(&frame)["header_mutations"].as_array().unwrap();
        assert_eq!(headers.len(), usize::from(transformed));
    }
}
