use gateway_plugin_sdk::{ErrorCode, PluginFault, Stage};
use serde_json::{Value, json};

use crate::support::{Peer, no_callback, result};

#[tokio::test]
async fn invalid_management_requests_return_json_without_host_callbacks() {
    let mut peer = Peer::start().await;
    for (method, path, query, content_type, payload, status, code) in [
        (
            "GET",
            "api/snapshot",
            "page=1",
            None,
            Vec::new(),
            400,
            "invalid_request",
        ),
        (
            "GET",
            "api/tasks",
            "",
            Some("application/json"),
            b"{}".to_vec(),
            400,
            "invalid_request",
        ),
        (
            "POST",
            "api/echo",
            "",
            Some("text/plain"),
            b"hello".to_vec(),
            400,
            "invalid_content_type",
        ),
        (
            "POST",
            "api/echo",
            "",
            Some("application/json"),
            b"{".to_vec(),
            400,
            "invalid_request",
        ),
        (
            "POST",
            "api/echo",
            "",
            Some("application/json"),
            vec![b' '; 512 * 1024 + 1],
            400,
            "invalid_request",
        ),
        (
            "POST",
            "api/log",
            "",
            Some("application/json"),
            b"null".to_vec(),
            400,
            "invalid_request",
        ),
        (
            "POST",
            "api/log",
            "",
            Some("application/json"),
            b"{\"extra\":true}".to_vec(),
            400,
            "invalid_request",
        ),
        (
            "POST",
            "api/echo",
            "",
            Some("application/json"),
            b"{\"message\":\"hello\",\"extra\":true}".to_vec(),
            400,
            "invalid_request",
        ),
        ("GET", "api/missing", "", None, Vec::new(), 404, "not_found"),
    ] {
        let frame = peer
            .call(
                "management.handle",
                Stage::Management,
                json!({"method":method,"path":path,"query":query,"content_type":content_type}),
                payload,
            )
            .await;
        assert_eq!(
            result(&frame),
            &json!({"status":status,"content_type":"application/json"})
        );
        let body: Value = serde_json::from_slice(&frame.payload).unwrap();
        assert_eq!(body["error"]["code"], code, "{method} {path}");
    }
}

#[tokio::test]
async fn management_accepts_json_charset_and_hides_host_fault_details() {
    let mut peer = Peer::start().await;
    let frame = peer.call("management.handle", Stage::Management,
        json!({"method":"POST","path":"api/echo","query":"","content_type":"Application/JSON; charset=utf-8"}),
        br#"{"message":"hello"}"#.to_vec()).await;
    assert_eq!(result(&frame)["status"], 200);
    assert_eq!(
        serde_json::from_slice::<Value>(&frame.payload).unwrap(),
        json!({"message":"hello"})
    );

    let (status, body) = peer
        .api("GET", "api/snapshot", None, |_, _, _| {
            Err(PluginFault::new(ErrorCode::Fault, "private-host-detail"))
        })
        .await;
    assert_eq!(
        (status, body),
        (
            502,
            json!({"error":{"code":"host_callback","message":"宿主操作未完成"}})
        )
    );

    let (status, _) = peer
        .api(
            "POST",
            "api/models",
            Some(json!({"clientKeyId":""})),
            no_callback,
        )
        .await;
    assert_eq!(status, 400);
}
