use super::{
    response::encoding_error,
    validation::{bounded, bounded_text, require_empty_body, valid_source_url},
};
use super::{
    response::{api_error, host_error, json_reply},
    validation::{decode_json, require_json},
};
use crate::{evidence::EvidenceLog, host_calls};
use gateway_plugin_sdk::call::host::StatePutRequest;
use gateway_plugin_sdk::{
    PluginFault,
    call::management::{ManagementRequest, ManagementResponse},
    client::{TypedCall, TypedReply},
};
use serde::{Deserialize, Serialize};

const STATE_NAMESPACE: &str = "workbench";
const STATE_KEY: &str = "tasks";

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Tasks {
    selected_id: Option<String>,
    entries: Vec<Task>,
}

impl Tasks {
    fn validate(&self) -> bool {
        if self.entries.len() > 8
            || self
                .selected_id
                .as_ref()
                .is_some_and(|selected| !self.entries.iter().any(|entry| &entry.id == selected))
        {
            return false;
        }
        let mut ids = std::collections::BTreeSet::new();
        self.entries
            .iter()
            .all(|entry| ids.insert(&entry.id) && entry.validate())
            && serde_json::to_vec(self).is_ok_and(|value| value.len() <= 256 * 1024)
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Task {
    id: String,
    title: String,
    source_kind: SourceKind,
    source: String,
    source_url: NullableString,
    task: TaskKind,
    instruction: String,
    result: String,
    model_id: String,
    client_key_id: String,
    updated_at: u64,
}

impl Task {
    fn validate(&self) -> bool {
        let source_matches = match (&self.source_kind, &self.source_url.0) {
            (SourceKind::Text, None) => true,
            (SourceKind::Url, Some(url)) => valid_source_url(url),
            (SourceKind::Text, Some(_)) | (SourceKind::Url, None) => false,
        };
        source_matches
            && bounded(&self.id, 1, 128)
            && bounded(&self.title, 0, 256)
            && bounded_text(&self.source, 256 * 1024)
            && bounded_text(&self.instruction, 4 * 1024)
            && bounded_text(&self.result, 256 * 1024)
            && bounded(&self.model_id, 0, 256)
            && bounded(&self.client_key_id, 0, 256)
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(transparent)]
struct NullableString(Option<String>);

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum SourceKind {
    Text,
    Url,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum TaskKind {
    Summarize,
    Translate,
    Rewrite,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SaveTasksRequest {
    expected_version: Option<u64>,
    value: Tasks,
}

#[derive(Serialize)]
struct TasksResponse {
    version: Option<u64>,
    value: Option<Tasks>,
}

pub(super) async fn get(
    _evidence: &EvidenceLog,
    call: TypedCall<ManagementRequest>,
) -> Result<TypedReply<ManagementResponse>, PluginFault> {
    if let Err(reply) = require_empty_body(&call.request, &call.payload) {
        return reply;
    }
    match host_calls::get_state(&call.host, STATE_NAMESPACE, STATE_KEY).await {
        Ok(result) => match result.record {
            Some(record) if record.schema_version == 1 => {
                match serde_json::from_value::<Tasks>(record.value) {
                    Ok(value) => json_reply(
                        200,
                        &TasksResponse {
                            version: Some(record.version),
                            value: Some(value),
                        },
                    ),
                    Err(_) => api_error(500, "invalid_state", "已保存的工作台记录无效"),
                }
            }
            Some(_) => api_error(500, "invalid_state", "已保存的工作台记录格式不受支持"),
            None => json_reply(
                200,
                &TasksResponse {
                    version: None,
                    value: None,
                },
            ),
        },
        Err(error) => host_error(error),
    }
}

pub(super) async fn save(
    _evidence: &EvidenceLog,
    call: TypedCall<ManagementRequest>,
) -> Result<TypedReply<ManagementResponse>, PluginFault> {
    if let Err(reply) = require_json(&call.request) {
        return reply;
    }
    let request = match decode_json::<SaveTasksRequest>(&call.payload) {
        Ok(request) if request.value.validate() => request,
        _ => {
            return api_error(400, "invalid_request", "任务记录无效或超过数量与长度限制");
        }
    };
    let value = match serde_json::to_value(request.value) {
        Ok(value) => value,
        Err(_) => return encoding_error(),
    };
    match host_calls::put_state(
        &call.host,
        &StatePutRequest {
            namespace: STATE_NAMESPACE.to_owned(),
            key: STATE_KEY.to_owned(),
            value,
            expected_version: request.expected_version,
        },
    )
    .await
    {
        Ok(result) => json_reply(200, &result),
        Err(error) => host_error(error),
    }
}
