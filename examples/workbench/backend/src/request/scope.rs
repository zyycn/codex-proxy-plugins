use serde_json::Value;
use std::{
    collections::{BTreeSet, VecDeque},
    sync::Mutex,
};

#[derive(Default)]
pub(crate) struct ScopeTracker {
    state: Mutex<ScopeState>,
}

#[derive(Default)]
struct ScopeState {
    order: VecDeque<String>,
    request_ids: BTreeSet<String>,
}

impl ScopeTracker {
    pub fn mark(&self, request_id: &str) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.request_ids.insert(request_id.to_owned()) {
            state.order.push_back(request_id.to_owned());
        }
        while state.order.len() > 1_024 {
            if let Some(expired) = state.order.pop_front() {
                state.request_ids.remove(&expired);
            }
        }
    }

    pub fn contains(&self, request_id: &str) -> bool {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .request_ids
            .contains(request_id)
    }

    pub fn take(&self, request_id: &str) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let removed = state.request_ids.remove(request_id);
        if removed {
            state.order.retain(|candidate| candidate != request_id);
        }
        removed
    }
}

pub(super) fn body_has_scope_marker(body: &[u8]) -> bool {
    serde_json::from_slice::<Value>(body)
        .ok()
        .as_ref()
        .and_then(|document| document.get("metadata"))
        .and_then(Value::as_object)
        .and_then(|metadata| metadata.get("capability_workbench"))
        .and_then(Value::as_str)
        == Some("true")
}
