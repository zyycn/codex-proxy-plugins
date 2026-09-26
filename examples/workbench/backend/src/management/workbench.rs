use super::{
    response::{ApiError, ApiResult, json_reply},
    validation::{bounded, decode_json, require_empty_body, require_empty_json},
};
use crate::{
    evidence::{EvidenceInput, EvidenceLog, EvidenceSnapshot, ExampleStatus},
    host_calls,
};
use gateway_plugin_sdk::{
    call::{
        host::{ClientKey, LogLevel, LogRequest},
        management::ManagementRequest,
    },
    client::TypedCall,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotResponse {
    contract_version: u32,
    keys: Vec<ClientKey>,
    keys_next_cursor: Option<String>,
    examples: Vec<ExampleStatus>,
    facts: EvidenceSnapshot,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct EchoRequest {
    message: String,
}

pub(super) async fn snapshot(
    evidence: &EvidenceLog,
    call: TypedCall<ManagementRequest>,
) -> ApiResult {
    require_empty_body(&call.request, &call.payload)?;
    let keys = host_calls::list_keys(&call.host, None, 100).await?;
    json_reply(&SnapshotResponse {
        contract_version: 1,
        keys: keys.keys,
        keys_next_cursor: keys.next_cursor,
        examples: evidence.examples(),
        facts: evidence.snapshot(),
    })
}

pub(super) fn echo(call: TypedCall<ManagementRequest>) -> ApiResult {
    let request: EchoRequest = decode_json(&call)?;
    if !bounded(&request.message, 1, 4 * 1024) {
        return Err(ApiError::invalid(
            "文本须为 1–4096 个 UTF-8 字节，且不含控制字符",
        ));
    }
    json_reply(&request)
}

pub(super) async fn log(evidence: &EvidenceLog, call: TypedCall<ManagementRequest>) -> ApiResult {
    require_empty_json(&call.request, &call.payload)?;
    let result = host_calls::log(
        &call.host,
        &LogRequest {
            event: "capability_workbench.demo_log".to_owned(),
            level: LogLevel::Info,
            fields: BTreeMap::from([("source".to_owned(), json!("management"))]),
        },
    )
    .await?;
    evidence.record(EvidenceInput::passed("host_services", "log_submitted"));
    json_reply(&result)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ModelsRequest {
    client_key_id: String,
}

pub(super) async fn list_models(call: TypedCall<ManagementRequest>) -> ApiResult {
    let request: ModelsRequest = decode_json(&call)?;
    if !bounded(&request.client_key_id, 1, 256) {
        return Err(ApiError::invalid(
            "clientKeyId must be a non-empty bounded identifier",
        ));
    }
    let result = host_calls::list_models(&call.host, request.client_key_id).await?;
    json_reply(&result)
}
