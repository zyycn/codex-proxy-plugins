use gateway_plugin_sdk::{ErrorCode, PluginFault};
use serde_json::json;

use crate::support::{Peer, no_callback};

#[tokio::test]
async fn task_history_reads_empty_saved_and_unsupported_state() {
    let value = json!({"selectedId":null,"entries":[]});
    let mut peer = Peer::start().await;
    for (record, expected_status, expected_body) in [
        (json!(null), 200, json!({"version":null,"value":null})),
        (
            json!({"schema_version":1,"version":4,"value":value}),
            200,
            json!({"version":4,"value":value}),
        ),
        (
            json!({"schema_version":2,"version":4,"value":value}),
            500,
            json!({"error":{"code":"invalid_state","message":"已保存的工作台记录格式不受支持"}}),
        ),
        (
            json!({"schema_version":1,"version":4,"value":false}),
            500,
            json!({"error":{"code":"invalid_state","message":"已保存的工作台记录无效"}}),
        ),
    ] {
        let response = peer
            .api("GET", "api/tasks", None, |method, params, _| {
                assert_eq!(method, "host.state.get");
                assert_eq!(params, &json!({"namespace":"workbench","key":"tasks"}));
                Ok((json!({"record":record}), Vec::new()))
            })
            .await;
        assert_eq!(response, (expected_status, expected_body));
    }
}

#[tokio::test]
async fn task_history_preserves_multiline_text() {
    let mut peer = Peer::start().await;
    let task = json!({
        "id":"multiline","title":"多段文本","sourceKind":"text",
        "source":"第一段\r\n\r\n\t第二段","sourceUrl":null,
        "task":"summarize","instruction":"保留段落\n使用中文",
        "result":"# 摘要\n\n- 第一项\n- 第二项","modelId":"test-model",
        "clientKeyId":"test-key","updatedAt":1,
    });
    let data = json!({"expectedVersion":null,"value":{"selectedId":"multiline","entries":[task]}});
    let (status, _) = peer
        .api(
            "POST",
            "api/tasks",
            Some(data.clone()),
            |method, params, _| {
                assert_eq!(method, "host.state.put");
                assert_eq!(params["value"], data["value"]);
                Ok((json!({"version":1}), Vec::new()))
            },
        )
        .await;
    assert_eq!(status, 200);

    for (field, value) in [
        ("source", "正文\0".to_owned()),
        ("result", "结果\u{1b}".to_owned()),
        ("instruction", "中".repeat(1_366)),
        ("modelId", "test-model\n".to_owned()),
    ] {
        let mut invalid = data.clone();
        invalid["value"]["entries"][0][field] = json!(value);
        let (status, _) = peer
            .api("POST", "api/tasks", Some(invalid), no_callback)
            .await;
        assert_eq!(status, 400, "字段 {field} 必须保留校验边界");
    }
}

#[tokio::test]
async fn task_history_checks_selection_unique_ids_and_write_version() {
    let mut peer = Peer::start().await;
    let task = json!({
        "id":"one","title":"示例","sourceKind":"text","source":"输入","sourceUrl":null,
        "task":"summarize","instruction":"","result":"输出","modelId":"test-model",
        "clientKeyId":"test-key","updatedAt":1,
    });
    for value in [
        json!({"selectedId":"missing","entries":[task.clone()]}),
        json!({"selectedId":null,"entries":[task.clone(),task.clone()]}),
    ] {
        let (status, _) = peer
            .api(
                "POST",
                "api/tasks",
                Some(json!({"expectedVersion":null,"value":value})),
                no_callback,
            )
            .await;
        assert_eq!(status, 400);
    }
    let data = json!({"expectedVersion":2,"value":{"selectedId":"one","entries":[task]}});
    let (status, body) = peer
        .api(
            "POST",
            "api/tasks",
            Some(data.clone()),
            |method, params, _| {
                assert_eq!(method, "host.state.put");
                assert_eq!(params["expected_version"], 2);
                assert_eq!(params["namespace"], "workbench");
                Ok((json!({"version":3}), Vec::new()))
            },
        )
        .await;
    assert_eq!((status, body), (200, json!({"version":3})));
    let (status, body) = peer
        .api("POST", "api/tasks", Some(data), |_, _, _| {
            Err(PluginFault::new(ErrorCode::Conflict, "测试版本冲突"))
        })
        .await;
    assert_eq!(status, 409);
    assert_eq!(body["error"]["code"], "conflict");
}
