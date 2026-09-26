use gateway_plugin_sdk::{ErrorCode, PluginFault};
use serde_json::json;

use crate::support::Peer;

#[tokio::test]
async fn text_fetch_truncates_only_incomplete_utf8_and_preserves_exact_limit() {
    const LIMIT: usize = 256 * 1024;
    for (tail, prefix_len, expected_status, truncated) in [
        ("中".as_bytes(), LIMIT - 1, 200, true),
        ("😀".as_bytes(), LIMIT - 2, 200, true),
        (&b"\xff"[..], LIMIT - 1, 502, false),
        (&b"\xffab"[..], LIMIT - 1, 502, true),
        (&b"a"[..], LIMIT - 1, 200, false),
    ] {
        let mut source = vec![b'a'; prefix_len];
        source.extend_from_slice(tail);
        let mut chunks = source.chunks(64 * 1024);
        let mut closed = false;
        let mut peer = Peer::start().await;
        let (status, body) = peer
            .api(
                "POST",
                "api/fetch-text",
                Some(json!({"url":"https://example.test/text"})),
                |method, _, _| match method {
                    "host.http.do_stream" => Ok((
                        json!({"status":200,"headers":[],"stream":"test-stream"}),
                        Vec::new(),
                    )),
                    "host.http.stream_read" => {
                        let chunk = chunks.next();
                        Ok((
                            json!({"eof":chunk.is_none()}),
                            chunk.unwrap_or_default().to_vec(),
                        ))
                    }
                    "host.http.stream_close" => {
                        closed = true;
                        Ok((json!({}), Vec::new()))
                    }
                    _ => panic!("出现未预期的宿主回调：{method}"),
                },
            )
            .await;
        assert_eq!(status, expected_status);
        assert_eq!(closed, truncated);
        if status == 200 {
            let expected_len = if truncated { prefix_len } else { LIMIT };
            assert_eq!(body["truncated"], truncated);
            assert_eq!(body["bytes"], expected_len);
            assert_eq!(body["text"], "a".repeat(expected_len));
        } else {
            assert_eq!(body["error"]["code"], "invalid_response");
        }
    }
}

#[tokio::test]
async fn text_fetch_accepts_text_and_closes_rejected_sources() {
    for (source_status, content_type, expected_status) in [
        (200, "text/html; charset=utf-8", 200),
        (200, "application/problem+json", 200),
        (200, "application/octet-stream", 502),
        (301, "text/plain", 502),
        (404, "text/plain", 502),
    ] {
        let mut peer = Peer::start().await;
        let mut reads = 0;
        let mut closed = false;
        let (status, body) = peer.api("POST", "api/fetch-text", Some(json!({"url":"https://example.test/text"})), |method, _, _| {
            match method {
                "host.http.do_stream" => Ok((json!({"status":source_status,"headers":[["content-type",content_type]],"stream":"test-stream"}), Vec::new())),
                "host.http.stream_read" => {
                    reads += 1;
                    Ok((json!({"eof": reads > 1}), if reads == 1 { "示例文本".as_bytes().to_vec() } else { Vec::new() }))
                }
                "host.http.stream_close" => { closed = true; Ok((json!({}), Vec::new())) }
                _ => panic!("出现未预期的宿主回调：{method}"),
            }
        }).await;
        assert_eq!(status, expected_status);
        if expected_status == 200 {
            assert_eq!(body["text"], "示例文本");
        } else {
            assert!(closed);
            assert_eq!(reads, 0);
        }
    }
}

#[tokio::test]
async fn text_fetch_reports_network_failures_without_exposing_internal_faults() {
    for (code, message, expected_status, expected_message) in [
        (
            ErrorCode::Upstream,
            "域名解析失败，请检查宿主 DNS",
            502,
            "域名解析失败，请检查宿主 DNS",
        ),
        (
            ErrorCode::PermissionDenied,
            "目标地址被安全策略拦截，请检查宿主 DNS 或代理设置",
            502,
            "目标地址被安全策略拦截，请检查宿主 DNS 或代理设置",
        ),
        (
            ErrorCode::PermissionDenied,
            "callback resource is not authorized",
            502,
            "网页访问被拒绝，请检查插件网络权限",
        ),
        (
            ErrorCode::Timeout,
            "deadline elapsed",
            504,
            "读取网页超时，请稍后重试",
        ),
        (ErrorCode::Cancelled, "cancelled", 499, "已取消读取网页"),
        (
            ErrorCode::Capacity,
            "capacity exhausted",
            503,
            "网络请求繁忙，请稍后重试",
        ),
        (
            ErrorCode::Upstream,
            "managed HTTP operation failed",
            502,
            "读取网页失败，请检查网络或代理",
        ),
        (
            ErrorCode::Fault,
            "internal details: secret-test-only",
            502,
            "读取网页失败，请稍后重试",
        ),
    ] {
        let mut peer = Peer::start().await;
        let (status, body) = peer
            .api(
                "POST",
                "api/fetch-text",
                Some(json!({"url":"https://example.test/text"})),
                |method, _, _| {
                    assert_eq!(method, "host.http.do_stream");
                    Err(PluginFault::new(code, message))
                },
            )
            .await;
        assert_eq!(status, expected_status);
        assert_eq!(body["error"]["message"], expected_message);
        assert!(!body.to_string().contains("secret-test-only"));
    }
}

#[tokio::test]
async fn text_fetch_removes_page_fragments_and_keeps_the_bounded_stream() {
    let mut peer = Peer::start().await;
    let mut reads = 0;
    let mut closed = false;
    let (status, body) = peer.api(
        "POST", "api/fetch-text", Some(json!({"url":"https://example.test/text#section"})),
        |method, params, _| match method {
            "host.http.do_stream" => {
                assert_eq!(params["url"], "https://example.test/text");
                Ok((json!({"status":200,"headers":[["content-type","text/plain"]],"stream":"test-stream"}), Vec::new()))
            }
            "host.http.stream_read" => {
                reads += 1;
                Ok((json!({"eof":false}), vec![b'a'; 64 * 1024]))
            }
            "host.http.stream_close" => {
                closed = true;
                Ok((json!({}), Vec::new()))
            }
            _ => panic!("出现未预期的宿主回调：{method}"),
        },
    ).await;
    assert_eq!(status, 200);
    assert_eq!(body["url"], "https://example.test/text");
    assert_eq!(body["bytes"], 256 * 1024);
    assert_eq!(body["truncated"], true);
    assert_eq!(reads, 5);
    assert!(closed);
}

#[tokio::test]
async fn text_fetch_keeps_stream_read_timeout_distinct_from_open_failure() {
    let mut peer = Peer::start().await;
    let (status, body) = peer
        .api(
            "POST",
            "api/fetch-text",
            Some(json!({"url":"https://example.test/text"})),
            |method, _, _| match method {
                "host.http.do_stream" => Ok((
                    json!({"status":200,"headers":[],"stream":"test-stream"}),
                    Vec::new(),
                )),
                "host.http.stream_read" => {
                    Err(PluginFault::new(ErrorCode::Timeout, "response deadline"))
                }
                _ => panic!("出现未预期的宿主回调：{method}"),
            },
        )
        .await;
    assert_eq!(status, 504);
    assert_eq!(body["error"]["message"], "读取网页超时，请稍后重试");
}
