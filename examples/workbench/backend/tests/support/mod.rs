use std::time::Duration;

use codex_proxy_plugin_workbench::{PLUGIN_ID, manifest, plugin};
use gateway_plugin_sdk::{
    CallContext, Frame, Handshake, Message, PROTOCOL_VERSION, PluginFault, Stage,
    client::{PluginSession, SessionConfig, read_frame, write_frame},
};
use serde_json::{Value, json};
use tokio::{
    io::{DuplexStream, ReadHalf, WriteHalf},
    task::JoinHandle,
};

const MAXIMUM_FRAME_BYTES: usize = 1024 * 1024;
pub type CallbackReply = Result<(Value, Vec<u8>), PluginFault>;

// 从公开会话入口验证真实处理器；只替换宿主资源，不复制生产模块或扩大可见性。
pub struct Peer {
    reader: ReadHalf<DuplexStream>,
    writer: WriteHalf<DuplexStream>,
    task: JoinHandle<()>,
    next_id: u64,
}

impl Peer {
    pub async fn start() -> Self {
        let (host, transport) = tokio::io::duplex(MAXIMUM_FRAME_BYTES * 2);
        let (reader, writer) = tokio::io::split(transport);
        let task = tokio::spawn(async move {
            let session = PluginSession::accept(reader, writer, SessionConfig::default())
                .await
                .unwrap();
            session.run(plugin().unwrap()).await.unwrap();
        });
        let (reader, writer) = tokio::io::split(host);
        let mut peer = Self {
            reader,
            writer,
            task,
            next_id: 0,
        };
        let manifest = manifest().unwrap();
        peer.send(Frame::control(Message::Hello {
            handshake: Handshake {
                protocol_version: PROTOCOL_VERSION,
                artifact_sha256: "a".repeat(64),
                plugin_id: PLUGIN_ID.to_owned(),
                instance_id: "test-instance".to_owned(),
                generation: 1,
                incarnation: "test-session".to_owned(),
                configuration: json!({}),
                permissions: manifest.permissions.into_iter().collect(),
                contributes: manifest.contributes,
            },
        }))
        .await;
        assert!(matches!(
            peer.receive().await.message,
            Message::Ready { .. }
        ));
        peer
    }

    pub async fn call(
        &mut self,
        method: &str,
        stage: Stage,
        params: Value,
        payload: Vec<u8>,
    ) -> Frame {
        self.call_with(method, stage, params, payload, |method, _, _| {
            panic!("出现未预期的宿主回调：{method}");
        })
        .await
    }

    pub async fn call_with(
        &mut self,
        method: &str,
        stage: Stage,
        params: Value,
        payload: Vec<u8>,
        mut callback: impl FnMut(&str, &Value, &[u8]) -> CallbackReply,
    ) -> Frame {
        self.next_id += 1;
        let id = self.next_id * 2 - 1;
        self.send(Frame {
            message: Message::Call {
                id,
                method: method.to_owned(),
                context: CallContext {
                    call_id: id,
                    instance_id: "test-instance".to_owned(),
                    generation: 1,
                    incarnation: "test-session".to_owned(),
                    stage,
                    timeout_ms: 10_000,
                    resource_scope_id: "test-scope".to_owned(),
                    request_id: Some("test-request".to_owned()),
                    attempt_id: None,
                    account_id: None,
                    credential_revision: None,
                },
                params,
            },
            payload,
        })
        .await;
        if method == "middleware.handle" {
            self.send(Frame::control(Message::Credit {
                id,
                bytes: 1024 * 1024,
                frames: 128,
            }))
            .await;
        }
        loop {
            let frame = self.receive().await;
            match frame.message {
                Message::Callback {
                    id: callback_id,
                    parent_id,
                    method,
                    params,
                } => {
                    assert_eq!(parent_id, id);
                    let response = match callback(&method, &params, &frame.payload) {
                        Ok((result, payload)) => Frame {
                            message: Message::Result {
                                id: callback_id,
                                result,
                            },
                            payload,
                        },
                        Err(error) => Frame::control(Message::Error {
                            id: callback_id,
                            error,
                        }),
                    };
                    self.send(response).await;
                }
                Message::Result { id: reply_id, .. } | Message::Error { id: reply_id, .. } => {
                    assert_eq!(reply_id, id);
                    return frame;
                }
                _ => panic!("出现未预期的插件消息：{frame:?}"),
            }
        }
    }

    pub async fn api(
        &mut self,
        method: &str,
        path: &str,
        data: Option<Value>,
        callback: impl FnMut(&str, &Value, &[u8]) -> CallbackReply,
    ) -> (u64, Value) {
        let params = json!({
            "method": method, "path": path, "query": "",
            "content_type": data.as_ref().map(|_| "application/json"),
        });
        let payload = data.map_or_else(Vec::new, |value| serde_json::to_vec(&value).unwrap());
        let frame = self
            .call_with(
                "management.handle",
                Stage::Management,
                params,
                payload,
                callback,
            )
            .await;
        let result = result(&frame);
        (
            result["status"].as_u64().unwrap(),
            serde_json::from_slice(&frame.payload).unwrap(),
        )
    }

    pub async fn mark_request(&mut self) {
        let frame = self.call(
            "policy.route_model",
            Stage::Routing,
            json!({"request_id":"test-request","operation":"generate","protocol":"openai","model":"test-model","available_providers":["openai"]}),
            serde_json::to_vec(&json!({"metadata":{"capability_workbench":"true"}})).unwrap(),
        ).await;
        assert_eq!(
            result(&frame),
            &json!({"decision":"route","provider":"openai"})
        );
    }

    pub async fn snapshot(&mut self) -> Value {
        let (status, value) = self
            .api("GET", "api/snapshot", None, |method, _, _| {
                assert_eq!(method, "host.keys.list");
                Ok((json!({"keys":[],"next_cursor":null}), Vec::new()))
            })
            .await;
        assert_eq!(status, 200);
        value
    }

    pub async fn send(&mut self, frame: Frame) {
        write_frame(&mut self.writer, &frame, MAXIMUM_FRAME_BYTES)
            .await
            .unwrap();
    }

    pub async fn receive(&mut self) -> Frame {
        tokio::time::timeout(
            Duration::from_secs(10),
            read_frame(&mut self.reader, MAXIMUM_FRAME_BYTES),
        )
        .await
        .expect("插件响应超时")
        .unwrap()
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub fn result(frame: &Frame) -> &Value {
    let Message::Result { result, .. } = &frame.message else {
        panic!("预期成功响应，实际收到：{frame:?}");
    };
    result
}

pub fn no_callback(method: &str, _: &Value, _: &[u8]) -> CallbackReply {
    panic!("出现未预期的宿主回调：{method}");
}
