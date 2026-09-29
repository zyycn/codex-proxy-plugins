use base64::{Engine as _, engine::general_purpose::STANDARD};
use gateway_plugin_sdk::{ErrorCode, Message, PluginFault, Stage};
use serde_json::{Value, json};

use crate::support::{Peer, no_callback, result};

fn head() -> Value {
    json!({"request_id":"test-request","settings_sources":null,"client_key_id":"test-key","account_group_ids":[],"mount":"request","operation":"generate_image","protocol":"openai",
    "endpoint":"/v1/images/edits","transport":"http_json","headers":[
        {"name":"Content-Type","value":b"multipart/form-data; boundary=\"test-boundary\"".to_vec()}
    ]})
}

fn form(fields: &[(&str, &str)], images: &[(&str, &[u8])]) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, value) in fields {
        body.extend_from_slice(format!("--test-boundary\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n").as_bytes());
    }
    for (name, bytes) in images {
        body.extend_from_slice(format!("--test-boundary\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"test.png\"\r\nContent-Type: image/png\r\n\r\n").as_bytes());
        body.extend_from_slice(bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(b"--test-boundary--\r\n");
    body
}

fn valid_form() -> Vec<u8> {
    form(
        &[("model", "gpt-image-2"), ("prompt", "make red")],
        &[("image", b"\x89PNG\0\xff")],
    )
}

#[tokio::test]
async fn image_edit_converts_binary_uploads_and_preserves_downstream_response() {
    let first = b"\x89PNG\0\xff\r\n--not-a-boundary";
    let second = b"second";
    let body = form(
        &[
            ("model", "gpt-image-2"),
            ("prompt", "make red"),
            ("n", "2"),
            ("size", "1024x1024"),
            ("quality", "low"),
            ("background", "opaque"),
            ("response_format", "b64_json"),
        ],
        &[("image", first), ("image[]", second)],
    );
    let mut peer = Peer::start().await;
    let mut calls = 0;
    let frame = peer.call_with("middleware.handle", Stage::Request, head(), body, |method, params, payload| {
        calls += 1;
        assert_eq!(method, "host.middleware.next");
        assert_eq!(params["body"], "replace");
        assert_eq!(params["header_mutations"], json!([
            {"operation":"remove","name":"content-type"},
            {"operation":"append","name":"content-type","value":b"application/json".to_vec()}
        ]));
        let body: Value = serde_json::from_slice(payload).unwrap();
        assert_eq!(body, json!({"model":"gpt-image-2","prompt":"make red","n":2,"size":"1024x1024","quality":"low","background":"opaque",
            "images":[{"image_url":format!("data:image/png;base64,{}", STANDARD.encode(first))},
                {"image_url":format!("data:image/png;base64,{}", STANDARD.encode(second))}]}));
        Ok((json!({"response":"image-response","protocol":"openai","status":200,"headers":[],
            "body":{"handle":"image-body","framing":"json_document"}}), Vec::new()))
    }).await;
    assert_eq!(calls, 1);
    assert_eq!(result(&frame)["response"], "image-response");
    assert_eq!(result(&frame)["body"]["kind"], "pass_through");
}

#[tokio::test]
async fn image_edit_leaves_json_other_endpoints_and_attempts_untouched() {
    for case in ["json", "generation", "attempt"] {
        let mut params = head();
        let mut body = valid_form();
        let mut stage = Stage::Request;
        match case {
            "json" => {
                params["headers"] = json!([]);
                body = br#"{ "images": [] }"#.to_vec();
            }
            "generation" => params["endpoint"] = json!("/v1/images/generations"),
            "attempt" => {
                params["mount"] = json!("attempt");
                params["attempt_index"] = json!(1);
                stage = Stage::Attempt;
            }
            _ => unreachable!(),
        }
        let mut peer = Peer::start().await;
        peer.call_with(
            "middleware.handle",
            stage,
            params,
            body,
            |method, params, payload| {
                assert_eq!(method, "host.middleware.next");
                assert_eq!(params["body"], "preserve");
                assert!(payload.is_empty());
                assert!(
                    params["header_mutations"]
                        .as_array()
                        .is_none_or(Vec::is_empty)
                );
                Ok((
                    json!({"response":"original","protocol":"openai","status":200,"headers":[]}),
                    Vec::new(),
                ))
            },
        )
        .await;
    }
}

#[tokio::test]
async fn image_edit_rejects_invalid_or_unsupported_input_before_next() {
    let required = [("model", "gpt-image-2"), ("prompt", "make red")];
    let mut cases = vec![
        b"broken multipart".to_vec(),
        form(&required, &[]),
        form(&[("model", "gpt-image-2")], &[("image", b"png")]),
    ];
    for extra in [
        ("mask", "mask.png"),
        ("n", "0"),
        ("n", "1.5"),
        ("prompt", "duplicate"),
        ("response_format", "url"),
    ] {
        cases.push(form(
            &[required[0], required[1], extra],
            &[("image", b"png")],
        ));
    }
    cases.push(vec![0; 16 * 1024 * 1024 + 1]);
    for body in cases {
        let mut peer = Peer::start().await;
        let frame = peer
            .call_with(
                "middleware.handle",
                Stage::Request,
                head(),
                body,
                no_callback,
            )
            .await;
        assert_eq!(result(&frame)["status"], 400);
    }
}

#[tokio::test]
async fn image_edit_preserves_upstream_error_status_and_body_handle() {
    let mut peer = Peer::start().await;
    let frame = peer.call_with("middleware.handle", Stage::Request, head(), valid_form(), |_, _, _| {
        Ok((json!({"response":"error-response","protocol":"openai","status":429,"headers":[],
            "body":{"handle":"error-body","framing":"raw_bytes"}}), Vec::new()))
    }).await;
    assert_eq!(result(&frame)["response"], "error-response");
    assert!(result(&frame).get("status").is_none());
    assert_eq!(result(&frame)["body"]["kind"], "pass_through");
}

#[tokio::test]
async fn image_edit_propagates_downstream_cancellation_without_retry() {
    let mut peer = Peer::start().await;
    let mut calls = 0;
    let frame = peer
        .call_with(
            "middleware.handle",
            Stage::Request,
            head(),
            valid_form(),
            |_, _, _| {
                calls += 1;
                Err(PluginFault::new(ErrorCode::Cancelled, "cancelled"))
            },
        )
        .await;
    assert_eq!(calls, 1);
    assert!(
        matches!(frame.message, Message::Error { error, .. } if error.code == ErrorCode::Cancelled)
    );
}
