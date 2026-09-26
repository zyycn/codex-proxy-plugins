use gateway_plugin_sdk::{
    ErrorCode, PluginFault,
    call::middleware::MiddlewareMount,
    client::{MiddlewareCall, MiddlewareResponse},
};

use super::scope::{ScopeTracker, body_has_scope_marker};
use crate::{
    evidence::{EvidenceInput, EvidenceLog, EvidenceOutcome},
    host_calls,
};
use serde_json::{Value, json};
const RESPONSE_HEADER: &str = "x-cpr-capability-workbench";

pub(crate) async fn middleware(
    evidence: &EvidenceLog,
    scope: &ScopeTracker,
    call: MiddlewareCall,
) -> Result<MiddlewareResponse, PluginFault> {
    let request_id = call.context.request_id.clone();
    if body_has_scope_marker(&call.request.body)
        && let Some(request_id) = &request_id
    {
        scope.mark(request_id);
    }
    let scoped = request_id
        .as_deref()
        .is_some_and(|request_id| scope.contains(request_id));
    if !scoped {
        return call.next.run(call.request).await;
    }
    let MiddlewareCall {
        context: _,
        mut request,
        next,
        cancellation: _,
        host,
    } = call;
    let mount = request.head.mount;
    let provider = request.head.provider.clone();
    let model = request.head.model.clone();
    let account_id = request.head.account_id.clone();
    let uppercased = if mount == MiddlewareMount::Request {
        prepare_input(&mut request)?
    } else {
        false
    };
    request.append_header(
        match mount {
            MiddlewareMount::Request => "x-cpr-workbench-request",
            MiddlewareMount::Attempt => "x-cpr-workbench-attempt",
        },
        b"observed".to_vec(),
    );
    let affinity = if mount == MiddlewareMount::Attempt
        && let Some(provider) = &provider
    {
        host_calls::affinity(&host, provider.clone(), "capability-workbench".to_owned())
            .await
            .ok()
            .and_then(|result| result.account_id)
    } else {
        None
    };
    let response = next.run(request).await;
    let mut item = EvidenceInput::passed("middleware", "downstream_failed");
    item.request_id = request_id.as_deref();
    item.provider = provider.as_deref();
    item.account_id = account_id.as_deref();
    item.model = model.as_deref();
    match response {
        Ok(mut response) => {
            if mount == MiddlewareMount::Request {
                response.append_header(RESPONSE_HEADER, b"observed".to_vec());
            }
            let succeeded = response.status < 400;
            item.event = if succeeded {
                "request_processed"
            } else {
                "downstream_error_response"
            };
            if !succeeded {
                item.outcome = EvidenceOutcome::Failed;
            }
            item.details.insert(
                "mount".to_owned(),
                json!(match mount {
                    MiddlewareMount::Request => "request",
                    MiddlewareMount::Attempt => "attempt",
                }),
            );
            item.details
                .insert("uppercased".to_owned(), json!(uppercased));
            item.details
                .insert("responseStatus".to_owned(), json!(response.status));
            if let Some(account_id) = affinity {
                item.details
                    .insert("affinityAccountId".to_owned(), json!(account_id));
            }
            evidence.record(item);
            Ok(response)
        }
        Err(error) => {
            item.outcome = EvidenceOutcome::Failed;
            evidence.record(item);
            Err(error)
        }
    }
}

fn prepare_input(
    request: &mut gateway_plugin_sdk::client::MiddlewareRequest,
) -> Result<bool, PluginFault> {
    let Ok(mut document) = serde_json::from_slice::<Value>(&request.body) else {
        return Ok(false);
    };
    let Some(metadata) = document.get_mut("metadata").and_then(Value::as_object_mut) else {
        return Ok(false);
    };
    // 示例控制字段只由本插件消费，不传给可能不接受 metadata 的上游。
    let marked = metadata.remove("capability_workbench").is_some();
    let uppercase = metadata.remove("capability_workbench_uppercase");
    if !marked && uppercase.is_none() {
        return Ok(false);
    }
    if metadata.is_empty()
        && let Some(object) = document.as_object_mut()
    {
        object.remove("metadata");
    }
    let uppercased = request.head.body_visible
        && uppercase.as_ref().and_then(Value::as_str) == Some("true")
        && document.get_mut("input").is_some_and(uppercase_text);
    request.replace_body(
        serde_json::to_vec(&document)
            .map_err(|_| PluginFault::new(ErrorCode::Fault, "中间件请求编码失败"))?,
    );
    Ok(uppercased)
}

fn uppercase_text(value: &mut Value) -> bool {
    match value {
        Value::String(text) => {
            *text = text.to_uppercase();
            true
        }
        Value::Array(items) => {
            let mut changed = false;
            for item in items {
                changed = uppercase_text(item) || changed;
            }
            changed
        }
        Value::Object(object) => {
            if let Some(text) = object.get_mut("text") {
                uppercase_text(text)
            } else if let Some(content) = object.get_mut("content") {
                uppercase_text(content)
            } else {
                false
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => false,
    }
}
