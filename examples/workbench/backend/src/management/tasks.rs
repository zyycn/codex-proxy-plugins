use super::{
    response::{ApiError, ApiResult, json_reply},
    validation::{bounded, bounded_text, decode_json, require_empty_body, valid_source_url},
};
use crate::host_calls;
use gateway_plugin_sdk::{
    call::{host::StatePutRequest, management::ManagementRequest},
    client::TypedCall,
};
use serde::{Deserialize, Serialize};

const STATE_NAMESPACE: &str = "workbench";
const STATE_KEY: &str = "tasks";

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
struct NullableString(Option<String>);

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum SourceKind {
    Text,
    Url,
}

#[derive(Deserialize, Serialize)]
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

pub(super) async fn get(call: TypedCall<ManagementRequest>) -> ApiResult {
    require_empty_body(&call.request, &call.payload)?;
    let result = host_calls::get_state(&call.host, STATE_NAMESPACE, STATE_KEY).await?;
    let (version, value) = match result.record {
        Some(record) => {
            if record.schema_version != 1 {
                return Err(ApiError::new(
                    500,
                    "invalid_state",
                    "已保存的工作台记录格式不受支持",
                ));
            }
            let value = serde_json::from_value(record.value)
                .map_err(|_| ApiError::new(500, "invalid_state", "已保存的工作台记录无效"))?;
            (Some(record.version), Some(value))
        }
        None => (None, None),
    };
    json_reply(&TasksResponse { version, value })
}

pub(super) async fn save(call: TypedCall<ManagementRequest>) -> ApiResult {
    let request: SaveTasksRequest = decode_json(&call)?;
    if !request.value.validate() {
        return Err(ApiError::invalid("任务记录无效或超过数量与长度限制"));
    }
    let result = host_calls::put_state(
        &call.host,
        &StatePutRequest {
            namespace: STATE_NAMESPACE.to_owned(),
            key: STATE_KEY.to_owned(),
            value: serde_json::to_value(request.value)?,
            expected_version: request.expected_version,
        },
    )
    .await?;
    json_reply(&result)
}
