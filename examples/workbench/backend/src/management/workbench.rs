use super::validation::{bounded, require_empty_body, require_empty_json};
use super::{
    response::{api_error, host_error, json_reply},
    validation::{decode_json, require_json},
};
use crate::evidence::{EvidenceInput, EvidenceSnapshot, ExampleStatus};
use crate::{evidence::EvidenceLog, host_calls};
use gateway_plugin_sdk::call::host::{LogLevel, LogRequest};
use gateway_plugin_sdk::{
    PluginFault,
    call::management::{ManagementRequest, ManagementResponse},
    client::{TypedCall, TypedReply},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

const MAXIMUM_ECHO_BYTES: usize = 4 * 1024;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotResponse {
    contract_version: u32,
    keys: Vec<gateway_plugin_sdk::call::host::ClientKey>,
    keys_next_cursor: Option<String>,
    examples: Vec<ExampleStatus>,
    facts: EvidenceSnapshot,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct EchoRequest {
    message: String,
}

pub(super) async fn snapshot(
    evidence: &EvidenceLog,
    call: TypedCall<ManagementRequest>,
) -> Result<TypedReply<ManagementResponse>, PluginFault> {
    if let Err(reply) = require_empty_body(&call.request, &call.payload) {
        return reply;
    }
    match host_calls::list_keys(&call.host, None, 100).await {
        Ok(keys) => json_reply(
            200,
            &SnapshotResponse {
                contract_version: 1,
                keys: keys.keys,
                keys_next_cursor: keys.next_cursor,
                examples: evidence.examples(),
                facts: evidence.snapshot(),
            },
        ),
        Err(error) => host_error(error),
    }
}

pub(super) async fn echo(
    _evidence: &EvidenceLog,
    call: TypedCall<ManagementRequest>,
) -> Result<TypedReply<ManagementResponse>, PluginFault> {
    if let Err(reply) = require_json(&call.request) {
        return reply;
    }
    match decode_json::<EchoRequest>(&call.payload).and_then(validate_echo) {
        Ok(message) => json_reply(200, &message),
        Err(error) => api_error(400, "invalid_request", error),
    }
}

pub(super) async fn log(
    evidence: &EvidenceLog,
    call: TypedCall<ManagementRequest>,
) -> Result<TypedReply<ManagementResponse>, PluginFault> {
    if let Err(reply) = require_empty_json(&call.request, &call.payload) {
        return reply;
    }
    let request = LogRequest {
        event: "capability_workbench.demo_log".to_owned(),
        level: LogLevel::Info,
        fields: BTreeMap::from([("source".to_owned(), json!("management"))]),
    };
    match host_calls::log(&call.host, &request).await {
        Ok(result) => {
            evidence.record(EvidenceInput::passed("host_services", "log_submitted"));
            json_reply(200, &result)
        }
        Err(error) => host_error(error),
    }
}

fn validate_echo(request: EchoRequest) -> Result<EchoRequest, &'static str> {
    if bounded(&request.message, 1, MAXIMUM_ECHO_BYTES) {
        Ok(request)
    } else {
        Err("文本须为 1–4096 个 UTF-8 字节，且不含控制字符")
    }
}
