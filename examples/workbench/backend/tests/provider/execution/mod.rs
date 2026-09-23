mod engine;
mod stream;

use gateway_plugin_sdk::call::provider::{ExecutionInput, PrepareExecution};
use serde_json::{Value, json};

pub(super) fn input(body: Value) -> Vec<u8> {
    let request: PrepareExecution = serde_json::from_value(json!({
        "protocol":"openai","operation":"generate","model":"demo-echo","account_id":"test-account",
        "credential_revision":1,"credential":{},"context":{},"request_profile":null,
    }))
    .unwrap();
    ExecutionInput {
        request,
        body: serde_json::to_vec(&body).unwrap(),
    }
    .encode()
    .unwrap()
}
