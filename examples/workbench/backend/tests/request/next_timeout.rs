//! 真实工作台子进程在 request 与 attempt 阶段等待 next 超过默认 SDK 预算的回归测试

use std::{collections::BTreeSet, process::Stdio, time::Duration};

use codex_proxy_plugin_workbench::{PLUGIN_ID, manifest};
use gateway_plugin_sdk::client::{read_frame, write_frame};
use gateway_plugin_sdk::{CallContext, Frame, Handshake, Message, PROTOCOL_VERSION, Stage};
use serde_json::json;
use tokio::process::{ChildStdout, Command};

use crate::support::result;

const NEXT_WAIT: Duration = Duration::from_secs(125);

#[tokio::test]
async fn middleware_next_wait_outlives_default_sdk_timeout_in_real_process() {
    // 可指定待打包的 release 二进制，默认使用 Cargo 为集成测试构建的真实插件
    let binary = std::env::var_os("CPR_WORKBENCH_TEST_BINARY")
        .unwrap_or_else(|| env!("CARGO_BIN_EXE_codex-proxy-plugin-workbench").into());
    let mut child = Command::new(binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut writer = child.stdin.take().unwrap();
    let mut reader = child.stdout.take().unwrap();
    write_frame(
        &mut writer,
        &Frame::control(Message::Hello {
            handshake: Handshake {
                protocol_version: PROTOCOL_VERSION,
                artifact_sha256: "a".repeat(64),
                plugin_id: PLUGIN_ID.to_owned(),
                instance_id: "test-instance".to_owned(),
                generation: 1,
                incarnation: "test-session".to_owned(),
                configuration: json!({}),
                contributes: manifest().unwrap().contributes,
            },
        }),
    )
    .await
    .unwrap();
    assert!(matches!(
        receive(&mut reader).await.message,
        Message::Ready { .. }
    ));

    let cases = [
        (1, Stage::Request, "request", false),
        (3, Stage::Request, "request", true),
        (5, Stage::Attempt, "attempt", false),
        (7, Stage::Attempt, "attempt", true),
    ];
    for (id, stage, mount, scoped) in cases {
        let request_id = format!("test-request-{id}");
        let mut input = json!({"input": "Hello"});
        if scoped {
            input["metadata"] = json!({
                "capability_workbench": "true",
                "capability_workbench_uppercase": "true",
            });
        }
        write_frame(
            &mut writer,
            &Frame {
                message: Message::Call {
                    id,
                    method: "middleware.handle".to_owned(),
                    context: CallContext {
                        call_id: id,
                        instance_id: "test-instance".to_owned(),
                        generation: 1,
                        incarnation: "test-session".to_owned(),
                        stage,
                        timeout_ms: 120_000,
                        resource_stream: false,
                        resource_scope_id: format!("test-scope-{id}"),
                        request_id: Some(request_id.clone()),
                        attempt_id: (stage == Stage::Attempt).then(|| format!("test-attempt-{id}")),
                        account_id: None,
                        credential_revision: None,
                    },
                    params: json!({
                        "request_id": request_id,
                        "settings_sources": null,
                        "client_key_id": "test-key",
                        "account_group_ids": [],
                        "mount": mount,
                        "attempt_index": (stage == Stage::Attempt).then_some(1),
                        "operation": "generate",
                        "protocol": "openai",
                        "endpoint": "responses",
                        "transport": "http_sse",
                        "headers": [],
                    }),
                },
                payload: serde_json::to_vec(&input).unwrap(),
            },
        )
        .await
        .unwrap();
        write_frame(
            &mut writer,
            &Frame::control(Message::Credit {
                id,
                bytes: 1024 * 1024,
                frames: 128,
            }),
        )
        .await
        .unwrap();
    }

    let mut callbacks = Vec::new();
    let mut waiting = BTreeSet::new();
    for _ in cases {
        let frame = receive(&mut reader).await;
        let Message::Callback {
            id,
            parent_id,
            method,
            params,
        } = frame.message
        else {
            panic!("预期 next 回调，实际收到：{frame:?}");
        };
        assert_eq!(method, "host.middleware.next");
        assert!(cases.iter().any(|case| case.0 == parent_id));
        assert!(waiting.insert(parent_id));
        if parent_id == 3 {
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&frame.payload).unwrap(),
                json!({"input": "HELLO"})
            );
        } else {
            assert_eq!(params["body"], "preserve");
            assert!(frame.payload.is_empty());
        }
        callbacks.push((id, parent_id));
    }

    // 所有 next 已发出后再实际等待，覆盖默认 120 秒期限且不推进虚拟时钟
    let unexpected = tokio::time::timeout(NEXT_WAIT, read_frame(&mut reader)).await;
    assert!(
        unexpected.is_err(),
        "等待 next 时插件提前结束调用：{unexpected:?}"
    );
    for (id, parent_id) in callbacks {
        write_frame(
            &mut writer,
            &Frame::control(Message::Result {
                id,
                result: json!({
                    "response": format!("response-{parent_id}"),
                    "protocol": "openai", "status": 200, "headers": [],
                }),
            }),
        )
        .await
        .unwrap();
    }

    let mut replied = BTreeSet::new();
    let mut ended = BTreeSet::new();
    for _ in 0..cases.len() * 2 {
        let frame = receive(&mut reader).await;
        match frame.message {
            Message::Result { id, .. } => {
                assert!(waiting.contains(&id) && replied.insert(id));
                assert_eq!(result(&frame)["response"], format!("response-{id}"));
                assert_eq!(
                    result(&frame)["header_mutations"].as_array().unwrap().len(),
                    usize::from(id == 3)
                );
                assert!(frame.payload.is_empty());
            }
            Message::End { id, error } => {
                assert!(error.is_none());
                assert!(replied.contains(&id) && ended.insert(id));
            }
            _ => panic!("预期中间件成功响应与终态，实际收到：{frame:?}"),
        }
    }
    assert_eq!(ended, waiting);
    write_frame(&mut writer, &Frame::control(Message::Shutdown))
        .await
        .unwrap();
    assert!(
        tokio::time::timeout(Duration::from_secs(10), child.wait())
            .await
            .unwrap()
            .unwrap()
            .success()
    );
}

async fn receive(reader: &mut ChildStdout) -> Frame {
    tokio::time::timeout(Duration::from_secs(10), read_frame(reader))
        .await
        .expect("插件响应超时")
        .unwrap()
}
