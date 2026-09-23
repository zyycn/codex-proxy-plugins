use gateway_plugin_sdk::Stage;
use serde_json::{Value, json};

use crate::support::{Peer, result};

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
