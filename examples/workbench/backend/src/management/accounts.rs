use super::{
    response::{host_error, json_reply},
    validation::require_empty_json,
};
use crate::{
    evidence::{EvidenceInput, EvidenceLog},
    host_calls,
    provider::{PROVIDER_ID, demo_facts},
};
use gateway_plugin_sdk::{
    PluginFault,
    call::{
        host::AuthSaveRequest,
        management::{ManagementRequest, ManagementResponse},
    },
    client::{TypedCall, TypedReply},
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PreparedAccount {
    id: String,
    name: String,
    created: bool,
}

#[derive(Serialize)]
struct PreparedAccounts {
    accounts: Vec<PreparedAccount>,
}

pub(super) async fn prepare(
    evidence: &EvidenceLog,
    call: TypedCall<ManagementRequest>,
) -> Result<TypedReply<ManagementResponse>, PluginFault> {
    if let Err(reply) = require_empty_json(&call.request, &call.payload) {
        return reply;
    }
    match prepare_demo_accounts(&call.host, evidence).await {
        Ok(accounts) => json_reply(200, &PreparedAccounts { accounts }),
        Err(error) => host_error(error),
    }
}

async fn prepare_demo_accounts(
    host: &gateway_plugin_sdk::client::HostClient,
    evidence: &EvidenceLog,
) -> Result<Vec<PreparedAccount>, PluginFault> {
    let mut accounts = Vec::new();
    let mut cursor = None;
    for _ in 0..16 {
        let page =
            host_calls::list_accounts(host, Some(PROVIDER_ID.to_owned()), cursor.take(), 100)
                .await?;
        accounts.extend(page.accounts);
        cursor = page.next_cursor;
        if cursor.is_none() {
            break;
        }
    }
    let mut result = Vec::new();
    for account in ["alpha", "bravo"] {
        if let Some(existing) = accounts
            .iter()
            .find(|candidate| candidate.upstream_account_id.as_deref() == Some(account))
        {
            result.push(PreparedAccount {
                id: existing.account_id.clone(),
                name: existing.name.clone(),
                created: false,
            });
            continue;
        }
        let facts = demo_facts(account, None);
        let name = facts.name.clone();
        let saved = host_calls::save_account(
            host,
            &AuthSaveRequest::Create {
                provider_id: PROVIDER_ID.to_owned(),
                facts,
            },
        )
        .await?;
        result.push(PreparedAccount {
            id: saved.account_id,
            name,
            created: true,
        });
    }
    evidence.record(EvidenceInput::passed(
        "host_services",
        "demo_accounts_prepared",
    ));
    Ok(result)
}
