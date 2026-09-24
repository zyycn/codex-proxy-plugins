use super::scope::ScopeTracker;
use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    PluginFault,
    call::policy::{AccountScheduleDecision, AccountScheduleRequest},
    client::{TypedCall, TypedReply},
};
use serde_json::json;
use std::cmp::Reverse;

pub(crate) async fn schedule_account(
    evidence: &EvidenceLog,
    scope: &ScopeTracker,
    call: TypedCall<AccountScheduleRequest>,
) -> Result<TypedReply<AccountScheduleDecision>, PluginFault> {
    if !scope.contains(&call.request.request_id) {
        return Ok(TypedReply::new(AccountScheduleDecision::Delegate));
    }
    let (decision, selected) = schedule_decision(&call.request);
    let mut item = EvidenceInput::passed("scheduler", "account_ordered");
    item.request_id = Some(&call.request.request_id);
    item.provider = Some(&call.request.provider);
    item.account_id = selected.as_deref();
    item.model = call.request.model.as_deref();
    item.details.insert(
        "candidateCount".to_owned(),
        json!(call.request.candidates.len()),
    );
    evidence.record(item);
    Ok(TypedReply::new(decision))
}

fn schedule_decision(
    request: &AccountScheduleRequest,
) -> (AccountScheduleDecision, Option<String>) {
    let selected = request
        .candidates
        .iter()
        .min_by_key(|candidate| {
            (
                candidate.in_flight,
                candidate.failure_rate_basis_points.unwrap_or_default(),
                Reverse(candidate.weight),
                candidate.account_id.as_str(),
            )
        })
        .map(|candidate| candidate.account_id.clone());
    let decision = selected
        .as_ref()
        .map_or(AccountScheduleDecision::Delegate, |account_id| {
            AccountScheduleDecision::Pick {
                account_id: account_id.clone(),
            }
        });
    (decision, selected)
}
