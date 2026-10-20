//! OMP 专属入口和控制消息适配，模型执行与传输继续由宿主负责

use std::borrow::Cow;

use gateway_plugin_sdk::{
    Manifest, ManifestError, PluginFault,
    call::middleware::MiddlewareHeader,
    client::{
        AuthorError, ComposedPlugin, MiddlewareCall, MiddlewareResult, PluginBuilder,
        WebSocketDirection, WebSocketKind, WebSocketMessage,
    },
};
use serde::Deserialize;

const OMP_PATH: &str = "/v1/codex/responses";
const RESPONSES_PATH: &str = "/v1/responses";
// 消息视图没有握手 URI，使用连接已有的握手头携带适配范围，避免维护连接缓存
const SCOPE_HEADER: &str = "x-codex-proxy-omp-compat";

/// 读取与打包 CLI 相同的作者清单
///
/// # Errors
///
/// 清单不满足 SDK 合同时失败
pub fn manifest() -> Result<Manifest, ManifestError> {
    Manifest::from_author_slice(include_bytes!("../../plugin.json"))
}

/// 根据清单组合 HTTP 与 WebSocket 处理器
///
/// # Errors
///
/// 清单与处理器不匹配时失败
pub fn plugin() -> Result<ComposedPlugin, AuthorError> {
    PluginBuilder::from_json(include_bytes!("../../plugin.json"))?
        .middleware(adapt)?
        .build()
}

async fn adapt(call: MiddlewareCall) -> Result<MiddlewareResult, PluginFault> {
    match call {
        MiddlewareCall::Http(mut call) => {
            // 适配范围只能由本次入口确定，不能继承客户端伪造的标记
            call.request.headers.retain(|header| {
                !header.name.eq_ignore_ascii_case(SCOPE_HEADER)
                    || header.value != call.context.instance_id.as_bytes()
            });
            let (path, query) = call
                .request
                .uri
                .split_once('?')
                .map_or((call.request.uri.as_str(), ""), |(path, _)| {
                    (path, &call.request.uri[path.len()..])
                });
            if path == OMP_PATH && matches!(call.request.method.as_str(), "GET" | "POST") {
                call.request.uri = format!("{RESPONSES_PATH}{query}");
                if call.request.method == "GET" {
                    call.request.headers.push(MiddlewareHeader {
                        name: SCOPE_HEADER.into(),
                        value: call.context.instance_id.as_bytes().to_vec(),
                    });
                    // 宿主已有的 hop-by-hop 过滤会阻止连接标记进入模型请求的上游头
                    call.request.headers.push(MiddlewareHeader {
                        name: "connection".into(),
                        value: SCOPE_HEADER.as_bytes().to_vec(),
                    });
                }
            }
            call.next
                .run(call.request)
                .await
                .map(MiddlewareResult::Http)
        }
        MiddlewareCall::WebSocket(mut call)
            if call.direction == WebSocketDirection::Incoming
                && call.message.kind == WebSocketKind::Text
                && call.headers.iter().any(|header| {
                    header.name.eq_ignore_ascii_case(SCOPE_HEADER)
                        && header.value == call.context.instance_id.as_bytes()
                }) =>
        {
            let kind = call.message.kind;
            let payload = call.message.payload.collect().await?;
            if let Some(rejection) = steering_rejection(&payload) {
                call.sender
                    .send(WebSocketMessage::new(WebSocketKind::Text, rejection))
                    .await?;
                // 只有确认实际写入后才丢弃控制消息，宿主不会执行或重放其中的输入
                Ok(MiddlewareResult::WebSocket(None))
            } else {
                call.message = WebSocketMessage::new(kind, payload);
                call.next
                    .run(call.message)
                    .await
                    .map(MiddlewareResult::WebSocket)
            }
        }
        call => call.forward().await,
    }
}

fn steering_rejection(payload: &[u8]) -> Option<Vec<u8>> {
    #[derive(Deserialize)]
    struct SteeringFrame<'a> {
        #[serde(borrow, rename = "type")]
        message_type: Cow<'a, str>,
        #[serde(borrow)]
        previous_response_id: Option<Cow<'a, str>>,
    }
    // 只解析关联字段，未知的 input 树由反序列化器跳过
    let frame: SteeringFrame<'_> = serde_json::from_slice(payload).ok()?;
    if frame.message_type != "response.steer" {
        return None;
    }
    Some(
        serde_json::json!({
            "type": "response.steer.failed",
            "steer": { "previous_response_id": frame.previous_response_id },
            "error": {
                "type": "invalid_request_error",
                "code": "unsupported_steering",
                "message": "Mid-turn steering is not supported; submit this input in the next response.create."
            }
        })
        .to_string()
        .into_bytes(),
    )
}
