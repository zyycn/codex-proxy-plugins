use gateway_plugin_sdk::{
    ErrorCode, PluginFault,
    call::middleware::MiddlewareMount,
    client::{MiddlewareCall, MiddlewareResponse},
};
use std::sync::Arc;

use super::scope::{ScopeTracker, body_has_scope_marker};
use crate::{
    evidence::{EvidenceInput, EvidenceLog, EvidenceOutcome},
    host_calls,
};
use serde_json::{Value, json};
const RESPONSE_HEADER: &str = "x-cpr-capability-workbench";

pub(crate) async fn middleware(
    evidence: Arc<EvidenceLog>,
    scope: Arc<ScopeTracker>,
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
        context,
        mut request,
        next,
        cancellation: _,
        host,
    } = call;
    let mount = request.head.mount;
    let provider = request.head.provider.clone();
    let model = request.head.model.clone();
    let account_id = request.head.account_id.clone();
    let request_id = context.request_id.clone();
    let uppercased = if mount == MiddlewareMount::Request {
        let uppercased = uppercase_designated_input(&mut request)?;
        consume_scope_metadata(&mut request)?;
        uppercased
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
    let affinity = if mount == MiddlewareMount::Attempt {
        if let Some(provider) = provider.clone() {
            match host_calls::affinity(&host, provider, "capability-workbench".to_owned()).await {
                Ok(result) => Some(result.account_id),
                Err(_) => None,
            }
        } else {
            None
        }
    } else {
        None
    };
    let response = next.run(request).await;
    match response {
        Ok(mut response) => {
            append_client_response_marker(mount, &mut response);
            let succeeded = response.status < 400;
            let mut item = EvidenceInput::passed(
                "middleware",
                if succeeded {
                    "request_processed"
                } else {
                    "downstream_error_response"
                },
            );
            if !succeeded {
                item.outcome = EvidenceOutcome::Failed;
            }
            item.request_id = request_id.as_deref();
            item.provider = provider.as_deref();
            item.account_id = account_id.as_deref();
            item.model = model.as_deref();
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
            if let Some(account_id) = affinity.flatten() {
                item.details
                    .insert("affinityAccountId".to_owned(), json!(account_id));
            }
            evidence.record(item);
            Ok(response)
        }
        Err(error) => {
            let mut item = EvidenceInput::passed("middleware", "downstream_failed");
            item.outcome = EvidenceOutcome::Failed;
            item.request_id = request_id.as_deref();
            item.provider = provider.as_deref();
            item.account_id = account_id.as_deref();
            item.model = model.as_deref();
            evidence.record(item);
            Err(error)
        }
    }
}

fn append_client_response_marker(
    mount: MiddlewareMount,
    response: &mut gateway_plugin_sdk::client::MiddlewareResponse,
) {
    if mount == MiddlewareMount::Request {
        response.append_header(RESPONSE_HEADER, b"observed".to_vec());
    }
}

fn uppercase_designated_input(
    request: &mut gateway_plugin_sdk::client::MiddlewareRequest,
) -> Result<bool, PluginFault> {
    if !request.head.body_visible || request.body.is_empty() {
        return Ok(false);
    }
    let mut document: Value = match serde_json::from_slice(&request.body) {
        Ok(document) => document,
        Err(_) => return Ok(false),
    };
    let designated = document
        .get("metadata")
        .and_then(Value::as_object)
        .and_then(|metadata| metadata.get("capability_workbench_uppercase"))
        .and_then(Value::as_str)
        == Some("true");
    if !designated {
        return Ok(false);
    }
    let Some(input) = document.get_mut("input") else {
        return Ok(false);
    };
    let changed = uppercase_text(input);
    if changed {
        request.replace_body(
            serde_json::to_vec(&document)
                .map_err(|_| PluginFault::new(ErrorCode::Fault, "中间件请求编码失败"))?,
        );
    }
    Ok(changed)
}

fn consume_scope_metadata(
    request: &mut gateway_plugin_sdk::client::MiddlewareRequest,
) -> Result<(), PluginFault> {
    let Ok(mut document) = serde_json::from_slice::<Value>(&request.body) else {
        return Ok(());
    };
    let Some(metadata) = document.get_mut("metadata").and_then(Value::as_object_mut) else {
        return Ok(());
    };
    // 示例控制字段只由本插件消费，不传给可能不接受 metadata 的上游。
    let marked = metadata.remove("capability_workbench").is_some();
    let uppercase = metadata.remove("capability_workbench_uppercase").is_some();
    if !marked && !uppercase {
        return Ok(());
    }
    if metadata.is_empty()
        && let Some(object) = document.as_object_mut()
    {
        object.remove("metadata");
    }
    request.replace_body(
        serde_json::to_vec(&document)
            .map_err(|_| PluginFault::new(ErrorCode::Fault, "中间件请求编码失败"))?,
    );
    Ok(())
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
