use serde_json::json;

use crate::support::{Peer, no_callback};

#[tokio::test]
async fn echo_preserves_text_and_rejects_invalid_inputs() {
    let mut peer = Peer::start().await;
    let (status, body) = peer
        .api(
            "POST",
            "api/echo",
            Some(json!({"message":"你好，插件"})),
            no_callback,
        )
        .await;
    assert_eq!((status, body), (200, json!({"message":"你好，插件"})));
    for message in [String::new(), "line\nbreak".to_owned(), "x".repeat(4097)] {
        let (status, _) = peer
            .api(
                "POST",
                "api/echo",
                Some(json!({"message":message})),
                no_callback,
            )
            .await;
        assert_eq!(status, 400);
    }
}

#[tokio::test]
async fn models_and_log_forward_only_their_declared_host_calls() {
    let mut peer = Peer::start().await;
    let (status, body) = peer
        .api(
            "POST",
            "api/models",
            Some(json!({"clientKeyId":"test-key"})),
            |method, params, _| {
                assert_eq!(method, "host.models.list");
                assert_eq!(params["client_key_id"], "test-key");
                Ok((json!({"models":["test-model"]}), Vec::new()))
            },
        )
        .await;
    assert_eq!((status, body), (200, json!({"models":["test-model"]})));
    let (status, body) = peer
        .api("POST", "api/log", Some(json!({})), |method, params, _| {
            assert_eq!(method, "host.log");
            assert_eq!(params["fields"], json!({"source":"management"}));
            Ok((json!({"recorded":true}), Vec::new()))
        })
        .await;
    assert_eq!((status, body), (200, json!({"recorded":true})));
}
