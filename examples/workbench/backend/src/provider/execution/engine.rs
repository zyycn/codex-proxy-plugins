use super::super::{AUTO_MODEL, ECHO_MODEL, PROVIDER_ID};
use super::input::{extract_input_text, split_text, usage};
use super::stream::{ExecutionEvidence, drive_echo_stream, response_event};
use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    ErrorCode, PluginFault,
    call::provider::{
        CanonicalEvent, ContentKind, ExecutePrepared, ExecutionInput, FinishReason, OperationKind,
        PreparedExecution, ProbeOperation, WireEvent, WirePayload, billing::PriceBand,
    },
    client::{Empty, ResponseStream, TypedCall, TypedReply, methods::ConnectionTest},
};
use serde_json::{Map, json};
use std::{
    collections::HashMap,
    num::NonZeroUsize,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

const MAXIMUM_PREPARED_EXECUTIONS: usize = 32;

struct PreparedEcho {
    text: String,
    model: String,
    account_id: String,
    request_id: Option<String>,
}

pub(crate) struct ProviderEngine {
    evidence: Arc<EvidenceLog>,
    next_token: AtomicU64,
    prepared: Mutex<HashMap<String, PreparedEcho>>,
}

impl ProviderEngine {
    pub(crate) fn new(evidence: Arc<EvidenceLog>) -> Self {
        Self {
            evidence,
            next_token: AtomicU64::new(0),
            prepared: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) async fn prepare(
        &self,
        call: TypedCall<ExecutionInput>,
    ) -> Result<TypedReply<PreparedExecution>, PluginFault> {
        if call.request.request.operation != OperationKind::Generate
            || call.request.request.protocol != "openai"
        {
            return Err(PluginFault::new(
                ErrorCode::Unsupported,
                "演示服务商仅支持 OpenAI 文本生成",
            ));
        }
        let model = call
            .request
            .request
            .model
            .clone()
            .filter(|model| model == ECHO_MODEL || model == AUTO_MODEL)
            .ok_or_else(|| PluginFault::new(ErrorCode::InvalidInput, "演示模型无效"))?;
        let text = extract_input_text(&call.request.body)?;
        let account_id = call.request.request.account_id.clone();
        let token_number = self.next_token.fetch_add(1, Ordering::Relaxed) + 1;
        let token = format!("echo-{token_number}");
        self.remember_prepared(
            token.clone(),
            PreparedEcho {
                text,
                model,
                account_id,
                request_id: call.context.request_id,
            },
        )?;
        Ok(TypedReply::new(PreparedExecution {
            token,
            transport: "plugin".to_owned(),
        }))
    }

    fn remember_prepared(&self, token: String, prepared: PreparedEcho) -> Result<(), PluginFault> {
        let mut executions = self
            .prepared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if executions.len() >= MAXIMUM_PREPARED_EXECUTIONS {
            return Err(PluginFault::new(
                ErrorCode::Capacity,
                "等待中的演示请求过多",
            ));
        }
        executions.insert(token, prepared);
        Ok(())
    }

    pub(crate) async fn execute(
        &self,
        call: TypedCall<ExecutePrepared>,
    ) -> Result<TypedReply<Empty>, PluginFault> {
        let prepared = self
            .prepared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&call.request.token)
            .ok_or_else(|| PluginFault::new(ErrorCode::InvalidInput, "找不到已准备的演示请求"))?;
        let response_id = format!("demo-response-{}", call.request.token);
        let cancellation = call.cancellation;
        let usage = usage(&prepared.text);
        let item_id = format!("{response_id}-message");
        let part = json!({"type":"output_text","text":prepared.text,"annotations":[]});
        let item = json!({
            "id":item_id,"type":"message","role":"assistant",
            "status":"completed","content":[part],
        });
        // canonical 供宿主计量，原生 wire 供 HTTP/SSE/WS 交付；两者不能互相替代。
        let mut events = vec![
            response_event(
                "response.created",
                json!({"response": {
                    "id":response_id,"object":"response","model":prepared.model,
                    "status":"in_progress","output":[],
                }}),
            )
            .with_fact(CanonicalEvent::Started {
                id: response_id.clone(),
                model: Some(prepared.model.clone()),
            }),
            response_event(
                "response.output_item.added",
                json!({
                    "output_index":0,"item":{
                        "id":item_id,"type":"message","role":"assistant",
                        "status":"in_progress","content":[],
                    },
                }),
            ),
            response_event(
                "response.content_part.added",
                json!({
                    "item_id":item_id,"output_index":0,"content_index":0,
                    "part":{"type":"output_text","text":"","annotations":[]},
                }),
            )
            .with_fact(CanonicalEvent::ContentAdded {
                index: 0,
                kind: ContentKind::Text,
            }),
        ];
        events.extend(split_text(&prepared.text).into_iter().map(|text| {
            response_event(
                "response.output_text.delta",
                json!({
                    "item_id":item_id,"output_index":0,"content_index":0,"delta":text,
                }),
            )
            .with_fact(CanonicalEvent::TextDelta { index: 0, text })
        }));
        events.extend([
            response_event(
                "response.output_text.done",
                json!({
                    "item_id":item_id,"output_index":0,"content_index":0,"text":prepared.text,
                }),
            ),
            response_event(
                "response.content_part.done",
                json!({
                    "item_id":item_id,"output_index":0,"content_index":0,"part":part,
                }),
            ),
            response_event(
                "response.output_item.done",
                json!({"output_index":0,"item":item}),
            ),
        ]);
        let billing_index = events.len();
        let completed = response_event(
            "response.completed",
            json!({"response": {
                "id":response_id,"object":"response","model":prepared.model,
                "status":"completed","output":[item],
                "usage":{
                    "input_tokens":usage.input_tokens,"output_tokens":usage.output_tokens,
                    "total_tokens":usage.input_tokens.zip(usage.output_tokens).map(|(input, output)| input + output),
                },
            }}),
        );
        events.push(
            completed
                .with_fact(CanonicalEvent::BillableUsage {
                    usage,
                    band: PriceBand::Standard,
                })
                .with_fact(CanonicalEvent::Completed {
                    id: response_id,
                    model: Some(prepared.model.clone()),
                    reason: FinishReason::Stop,
                }),
        );
        let chunks = events
            .into_iter()
            .enumerate()
            .map(|(sequence, mut event)| {
                if let Some(WireEvent {
                    payload: WirePayload::Json { data, .. },
                    ..
                }) = &mut event.wire
                {
                    data["sequence_number"] = json!(sequence);
                }
                event
                    .encode()
                    .map_err(|_| PluginFault::new(ErrorCode::Fault, "演示请求事件编码失败"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let execution = ExecutionEvidence {
            request_id: prepared.request_id,
            model: prepared.model,
            account_id: prepared.account_id,
            output_bytes: prepared.text.len(),
        };
        let (sender, stream) = ResponseStream::channel(NonZeroUsize::MIN);
        drop(tokio::spawn(drive_echo_stream(
            sender,
            chunks,
            billing_index,
            cancellation,
            Arc::clone(&self.evidence),
            execution,
        )));
        Ok(TypedReply::new(Empty {}).with_stream(stream))
    }

    pub(crate) async fn discard(
        &self,
        call: TypedCall<ExecutePrepared>,
    ) -> Result<TypedReply<Empty>, PluginFault> {
        self.prepared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&call.request.token);
        Ok(TypedReply::new(Empty {}))
    }

    pub(crate) async fn connection_test(
        &self,
        call: TypedCall<ConnectionTest>,
    ) -> Result<TypedReply<ProbeOperation>, PluginFault> {
        let mut body = Map::new();
        body.insert("model".to_owned(), json!(ECHO_MODEL));
        body.insert("input".to_owned(), json!(call.request.input));
        let mut evidence = EvidenceInput::passed("executor", "connection_test");
        evidence.provider = Some(PROVIDER_ID);
        evidence.model = Some(ECHO_MODEL);
        self.evidence.record(evidence);
        Ok(TypedReply::new(ProbeOperation {
            protocol: "openai".to_owned(),
            body,
        }))
    }
}
