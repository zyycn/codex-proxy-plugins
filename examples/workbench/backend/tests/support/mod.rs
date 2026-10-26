//! 通过公开 RPC 会话验证工作台处理器，模拟宿主回调与调度授权

use std::{collections::BTreeMap, time::Duration};

use codex_proxy_plugin_workbench::{PLUGIN_ID, manifest, plugin};
use gateway_plugin_sdk::{
    CallContext, Frame, Handshake, Message, PROTOCOL_VERSION, PluginFault, ResolvedCallBudgets,
    Stage,
    client::{PluginSession, SessionConfig, read_frame, write_frame},
};
use serde_json::{Value, json};
use tokio::{
    io::{DuplexStream, ReadHalf, WriteHalf},
    task::JoinHandle,
};

const PIPE_BUFFER_BYTES: usize = 1024 * 1024;
pub type CallbackReply = Result<(Value, Vec<u8>), PluginFault>;

// 只替换宿主资源，不复制生产模块或扩大可见性
pub struct Peer {
    reader: ReadHalf<DuplexStream>,
    writer: WriteHalf<DuplexStream>,
    task: JoinHandle<()>,
    next_id: u64,
    budgets: ResolvedCallBudgets,
    suspended: BTreeMap<u64, u64>,
    callbacks: BTreeMap<u64, u64>,
}

impl Peer {
    pub async fn start() -> Self {
        let (host, transport) = tokio::io::duplex(PIPE_BUFFER_BYTES * 2);
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
            budgets: ResolvedCallBudgets::resolve(&Default::default()).unwrap(),
            suspended: BTreeMap::new(),
            callbacks: BTreeMap::new(),
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
                contributes: manifest.contributes,
            },
        }))
        .await;
        let Message::Ready { call_budgets, .. } = peer.receive().await.message else {
            panic!("预期插件握手完成");
        };
        peer.budgets = ResolvedCallBudgets::resolve(&call_budgets).unwrap();
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
                    request: None,
                    call_id: id,
                    instance_id: "test-instance".to_owned(),
                    generation: 1,
                    incarnation: "test-session".to_owned(),
                    stage,
                    timeout_ms: u64::try_from(self.budgets.budget(stage).as_millis()).unwrap(),
                    resource_stream: false,
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
                    assert!(self.callbacks.is_empty());
                    assert!(!self.suspended.contains_key(&id));
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
        write_frame(&mut self.writer, &frame).await.unwrap();
    }

    pub async fn receive(&mut self) -> Frame {
        loop {
            let frame = tokio::time::timeout(Duration::from_secs(10), read_frame(&mut self.reader))
                .await
                .expect("插件响应超时")
                .unwrap();
            match frame.message {
                Message::Suspend { id, epoch } => {
                    assert!(epoch > 0 && self.suspended.insert(id, epoch).is_none());
                }
                Message::Resume { id, epoch } => {
                    assert_eq!(self.suspended.remove(&id), Some(epoch));
                    self.send(Frame::control(Message::Resumed { id, epoch }))
                        .await;
                }
                Message::CallbackConsumed { id, parent_id } => {
                    assert_eq!(self.callbacks.remove(&id), Some(parent_id));
                }
                Message::Callback { id, parent_id, .. } => {
                    assert!(self.callbacks.insert(id, parent_id).is_none());
                    return frame;
                }
                _ => return frame,
            }
        }
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
