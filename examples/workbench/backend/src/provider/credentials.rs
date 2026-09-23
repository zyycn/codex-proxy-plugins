use super::PROVIDER_ID;
use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    ErrorCode, Stage,
    call::auth::{CredentialFacts, ImportedCredentials, RotateCredential},
};
use gateway_plugin_sdk::{
    PluginFault,
    client::{TypedCall, TypedReply},
};
use serde_json::{Map, Value, json};

pub(crate) async fn import_credentials(
    evidence: &EvidenceLog,
    call: TypedCall<Map<String, Value>>,
) -> Result<TypedReply<ImportedCredentials>, PluginFault> {
    let account = demo_account_from_input(&call.request)?;
    evidence.record(EvidenceInput::passed(
        "authentication",
        "credentials_imported",
    ));
    Ok(TypedReply::new(ImportedCredentials {
        accounts: vec![demo_facts(&account, None)],
    }))
}

pub(crate) async fn rotate_credentials(
    evidence: &EvidenceLog,
    call: TypedCall<RotateCredential>,
) -> Result<TypedReply<CredentialFacts>, PluginFault> {
    let mut facts = call.request.current.facts;
    if let Some(replacement) = call.request.replacement {
        validate_public_replacement(&replacement)?;
        if let Some(label) = replacement.get("label") {
            facts.material.insert("label".to_owned(), label.clone());
        }
    }
    evidence.record(EvidenceInput::passed(
        "authentication",
        "credentials_rotated",
    ));
    Ok(TypedReply::new(facts))
}

pub(crate) async fn refresh_credentials(
    evidence: &EvidenceLog,
    call: TypedCall<RotateCredential>,
) -> Result<TypedReply<CredentialFacts>, PluginFault> {
    let facts = call.request.current.facts;
    let mut item = EvidenceInput::passed("authentication", "credentials_refreshed");
    item.account_id = Some(&call.request.current.account_id);
    item.provider = Some(PROVIDER_ID);
    evidence.record(item);
    if call.context.stage == Stage::Maintenance {
        evidence.record(EvidenceInput::passed(
            "maintenance",
            "credential_refresh_worker",
        ));
    }
    Ok(TypedReply::new(facts))
}

pub(crate) fn demo_facts(account: &str, label: Option<&str>) -> CredentialFacts {
    let display = title_case(account);
    let mut material = Map::new();
    material.insert("demoAccount".to_owned(), json!(account));
    material.insert(
        "label".to_owned(),
        json!(label.unwrap_or(if account == "alpha" {
            "主账号"
        } else {
            "备用账号"
        })),
    );
    CredentialFacts {
        name: format!("Demo {display}"),
        authentication_kind: "demo_fixture".to_owned(),
        material,
        email: Some(format!("{account}@demo.invalid")),
        upstream_user_id: Some(format!("demo-user-{account}")),
        upstream_account_id: Some(account.to_owned()),
        plan_type: Some("demo".to_owned()),
        has_refresh_token: false,
        access_token_expires_at_ms: None,
        next_refresh_at_ms: None,
    }
}

pub(super) fn credential_schema(require_account: bool) -> Value {
    let required = if require_account {
        json!(["demoAccount"])
    } else {
        json!([])
    };
    json!({
        "type": "object",
        "properties": {
            "demoAccount": {"enum": ["alpha", "bravo"]},
            "label": {"type": "string", "minLength": 1, "maxLength": 64}
        },
        "required": required,
        "additionalProperties": false
    })
}

fn demo_account_from_input(input: &Map<String, Value>) -> Result<String, PluginFault> {
    if input
        .keys()
        .any(|key| key != "demoAccount" && key != "label")
    {
        return Err(invalid_credentials());
    }
    let account = input
        .get("demoAccount")
        .and_then(Value::as_str)
        .filter(|account| matches!(*account, "alpha" | "bravo"))
        .ok_or_else(invalid_credentials)?;
    if let Some(label) = input.get("label") {
        valid_label(label)?;
    }
    Ok(account.to_owned())
}

fn validate_public_replacement(input: &Map<String, Value>) -> Result<(), PluginFault> {
    if input.keys().any(|key| key != "label") {
        return Err(invalid_credentials());
    }
    if let Some(label) = input.get("label") {
        valid_label(label)?;
    }
    Ok(())
}

fn valid_label(value: &Value) -> Result<(), PluginFault> {
    value
        .as_str()
        .filter(|value| {
            !value.is_empty() && value.len() <= 64 && !value.chars().any(char::is_control)
        })
        .map(|_| ())
        .ok_or_else(invalid_credentials)
}

fn invalid_credentials() -> PluginFault {
    PluginFault::new(ErrorCode::InvalidInput, "演示账号凭据无效")
}

pub(super) fn title_case(value: &str) -> String {
    let mut characters = value.chars();
    characters.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(characters).collect()
    })
}
