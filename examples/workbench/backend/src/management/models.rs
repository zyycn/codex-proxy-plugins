use super::validation::valid_identifier;
use super::{
    response::{api_error, host_error, json_reply},
    validation::{decode_json, require_json},
};
use crate::{evidence::EvidenceLog, host_calls};
use gateway_plugin_sdk::{
    PluginFault,
    call::management::{ManagementRequest, ManagementResponse},
    client::{TypedCall, TypedReply},
};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ModelsRequest {
    client_key_id: String,
}

pub(super) async fn list(
    _evidence: &EvidenceLog,
    call: TypedCall<ManagementRequest>,
) -> Result<TypedReply<ManagementResponse>, PluginFault> {
    if let Err(reply) = require_json(&call.request) {
        return reply;
    }
    let request = match decode_json::<ModelsRequest>(&call.payload) {
        Ok(request) if valid_identifier(&request.client_key_id) => request,
        _ => {
            return api_error(
                400,
                "invalid_request",
                "clientKeyId must be a non-empty bounded identifier",
            );
        }
    };
    match host_calls::list_models(&call.host, request.client_key_id).await {
        Ok(result) => json_reply(200, &result),
        Err(error) => host_error(error),
    }
}
