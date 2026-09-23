use super::super::PROVIDER_ID;
use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    call::provider::{ExecutionEvent, WireEvent, WirePayload},
    client::{CallCancellation, StreamSender},
};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};

const STREAM_INITIAL_DELAY: Duration = Duration::from_millis(300);
const STREAM_CHUNK_DELAY: Duration = Duration::from_millis(40);

pub(super) struct ExecutionEvidence {
    pub(super) request_id: Option<String>,
    pub(super) model: String,
    pub(super) account_id: String,
    pub(super) output_bytes: usize,
}

pub(super) fn response_event(event: &str, mut data: Value) -> ExecutionEvent {
    data["type"] = json!(event);
    ExecutionEvent::wire(WireEvent {
        protocol: "openai".to_owned(),
        payload: WirePayload::Json {
            event: Some(event.to_owned()),
            data,
            id: None,
            retry: None,
            raw_sse: None,
        },
    })
}

pub(super) async fn drive_echo_stream(
    sender: StreamSender,
    chunks: Vec<Vec<u8>>,
    billing_index: usize,
    cancellation: CallCancellation,
    evidence: Arc<EvidenceLog>,
    execution: ExecutionEvidence,
) {
    if !wait_for_stream(&cancellation, STREAM_INITIAL_DELAY).await {
        return;
    }
    for (index, chunk) in chunks.into_iter().enumerate() {
        if !wait_for_stream(&cancellation, STREAM_CHUNK_DELAY).await {
            return;
        }
        let sent = tokio::select! {
            biased;
            () = cancellation.cancelled() => false,
            result = sender.send(chunk) => result.is_ok(),
        };
        if !sent {
            return;
        }
        if index == billing_index {
            let mut billing = EvidenceInput::passed("billing", "billable_usage_emitted");
            billing.request_id = execution.request_id.as_deref();
            billing.provider = Some(PROVIDER_ID);
            billing.account_id = Some(&execution.account_id);
            billing.model = Some(&execution.model);
            evidence.record(billing);
        }
    }
    drop(sender);
    let mut completed = EvidenceInput::passed("executor", "execution_completed");
    completed.request_id = execution.request_id.as_deref();
    completed.provider = Some(PROVIDER_ID);
    completed.account_id = Some(&execution.account_id);
    completed.model = Some(&execution.model);
    completed
        .details
        .insert("outputBytes".to_owned(), json!(execution.output_bytes));
    evidence.record(completed);
}

async fn wait_for_stream(cancellation: &CallCancellation, delay: Duration) -> bool {
    tokio::select! {
        biased;
        () = cancellation.cancelled() => false,
        () = tokio::time::sleep(delay) => true,
    }
}
