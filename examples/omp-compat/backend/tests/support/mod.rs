//! 模拟宿主 RPC 对端，通过 SDK 帧读写驱动真实插件进程

use std::{process::Stdio, time::Duration};

use gateway_plugin_sdk::{
    CallContext, Frame, Handshake, Message, PROTOCOL_VERSION, PluginFault, Stage,
    client::{read_frame, write_frame},
};
use serde_json::{Value, json};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

pub const SCOPE_HEADER: &str = "x-codex-proxy-omp-compat";

pub struct Peer {
    child: Child,
    reader: ChildStdout,
    writer: ChildStdin,
}

pub struct Callback {
    pub id: u64,
    pub params: Value,
    pub payload: Vec<u8>,
}

impl Peer {
    pub async fn start() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_codex-proxy-plugin-omp-compat"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let mut peer = Self {
            reader: child.stdout.take().unwrap(),
            writer: child.stdin.take().unwrap(),
            child,
        };
        let manifest = codex_proxy_plugin_omp_compat::manifest().unwrap();
        peer.control(Message::Hello {
            handshake: Handshake {
                protocol_version: PROTOCOL_VERSION,
                artifact_sha256: "a".repeat(64),
                plugin_id: manifest.plugin_id().unwrap(),
                instance_id: "omp-test".into(),
                generation: 1,
                incarnation: "omp-test-incarnation".into(),
                configuration: json!({}),
                contributes: manifest.contributes.clone(),
            },
        })
        .await;
        assert!(matches!(
            peer.receive().await.message,
            Message::Ready {
                protocol_version: PROTOCOL_VERSION,
                ..
            }
        ));
        peer
    }

    pub async fn call(&mut self, id: u64, stage: Stage, params: Value, payload: &[u8]) {
        write_frame(
            &mut self.writer,
            &Frame {
                message: Message::Call {
                    id,
                    method: if stage == Stage::Registration {
                        "plugin.register"
                    } else {
                        "middleware.handle"
                    }
                    .into(),
                    context: CallContext {
                        call_id: id,
                        instance_id: "omp-test".into(),
                        generation: 1,
                        incarnation: "omp-test-incarnation".into(),
                        stage,
                        timeout_ms: 5000,
                        resource_stream: false,
                        resource_scope_id: format!("scope-{id}"),
                        request_id: (stage == Stage::Http).then(|| "req_omp".into()),
                        attempt_id: None,
                        account_id: None,
                        credential_revision: None,
                    },
                    params,
                },
                payload: payload.into(),
            },
        )
        .await
        .unwrap();
        self.control(Message::Credit {
            id,
            bytes: 65536,
            frames: 16,
        })
        .await;
    }

    pub async fn control(&mut self, message: Message) {
        write_frame(&mut self.writer, &Frame::control(message))
            .await
            .unwrap();
    }

    pub async fn reply(&mut self, id: u64, result: Value, payload: &[u8]) {
        write_frame(
            &mut self.writer,
            &Frame {
                message: Message::Result { id, result },
                payload: payload.into(),
            },
        )
        .await
        .unwrap();
    }

    pub async fn error(&mut self, id: u64, error: PluginFault) {
        self.control(Message::Error { id, error }).await;
    }

    pub async fn receive(&mut self) -> Frame {
        tokio::time::timeout(Duration::from_secs(10), read_frame(&mut self.reader))
            .await
            .expect("插件未在期限内返回消息")
            .expect("插件帧无效")
    }

    pub async fn callback(&mut self, expected_parent: u64, expected_method: &str) -> Callback {
        let frame = self.receive().await;
        let Message::Callback {
            id,
            parent_id,
            method,
            params,
        } = frame.message
        else {
            panic!("预期宿主回调，收到 {:?}", frame.message);
        };
        assert_eq!(parent_id, expected_parent);
        assert_eq!(method, expected_method);
        Callback {
            id,
            params,
            payload: frame.payload,
        }
    }

    pub async fn result(&mut self, expected_id: u64) -> (Value, Vec<u8>) {
        let frame = self.receive().await;
        let Message::Result { id, result } = frame.message else {
            panic!("预期调用结果，收到 {:?}", frame.message);
        };
        assert_eq!(id, expected_id);
        (result, frame.payload)
    }

    pub async fn http_result(&mut self, expected_id: u64) -> (Value, Vec<u8>) {
        let result = self.result(expected_id).await;
        assert!(
            matches!(self.receive().await.message, Message::End { id, error: None } if id == expected_id)
        );
        result
    }

    pub async fn shutdown(mut self) {
        self.control(Message::Shutdown).await;
        let status = tokio::time::timeout(Duration::from_secs(10), self.child.wait())
            .await
            .unwrap()
            .unwrap();
        assert!(status.success());
    }
}

pub fn http_call(method: &str, uri: &str) -> Value {
    json!({
        "settings_sources":null,"request_id":"req_omp","call_id":"http_1","parent_call_id":null,
        "request": {"settings":null,"method":method,"uri":uri,"version":"HTTP/1.1",
            "headers":[],"timeout_ms":60000,"body":{"kind":"handle","handle":"original-upload"}}
    })
}

pub fn http_response() -> Value {
    json!({"status":200,"version":"HTTP/1.1","headers":[],"body":{"kind":"handle","handle":"original-response"},"response":"response_1","session":false})
}

pub fn websocket_call(direction: &str, kind: &str, scoped: bool) -> Value {
    json!({"connection_id":"ws_omp","direction":direction,
        "headers":if scoped { json!([{"name":SCOPE_HEADER,"value":b"omp-test"}]) } else { json!([]) },
        "message":{"kind":{"kind":kind},"payload":{"kind":"bytes"}}
    })
}
