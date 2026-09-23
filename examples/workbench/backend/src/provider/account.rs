use super::PROVIDER_ID;
use super::{DEMO_FUTURE_MS, credentials::title_case};
use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    PluginFault,
    client::{TypedCall, TypedReply},
};
use gateway_plugin_sdk::{
    call::provider::account::{
        AccountConfiguration, AccountInvalidation, AccountRequest, Profile,
        ProfileActivityInsights, ProfileSummary, Subscription,
    },
    client::Empty,
};
use serde_json::{Map, Value, json};

pub(crate) async fn account_configuration(
    evidence: &EvidenceLog,
    call: TypedCall<AccountRequest>,
) -> Result<TypedReply<AccountConfiguration>, PluginFault> {
    let mut values = Map::new();
    if let Some(label) = call.request.credential.get("label") {
        values.insert("label".to_owned(), label.clone());
    }
    let mut item = EvidenceInput::passed("authentication", "account_configuration_read");
    item.account_id = Some(&call.request.account_id);
    item.provider = Some(PROVIDER_ID);
    evidence.record(item);
    Ok(TypedReply::new(AccountConfiguration { values }))
}

pub(crate) async fn account_changed(
    evidence: &EvidenceLog,
    call: TypedCall<AccountInvalidation>,
) -> Result<TypedReply<Empty>, PluginFault> {
    let (event, account_id, count) = match &call.request {
        AccountInvalidation::Unavailable { account_id } => {
            ("account_unavailable", Some(account_id.as_str()), 1)
        }
        AccountInvalidation::FactsChanged { account_ids } => (
            "account_facts_changed",
            account_ids.first().map(String::as_str),
            account_ids.len(),
        ),
    };
    let mut item = EvidenceInput::passed("account_management", event);
    item.account_id = account_id;
    item.provider = Some(PROVIDER_ID);
    item.details.insert("accountCount".to_owned(), json!(count));
    evidence.record(item);
    Ok(TypedReply::new(Empty {}))
}

pub(crate) async fn profile(
    evidence: &EvidenceLog,
    call: TypedCall<AccountRequest>,
) -> Result<TypedReply<Profile>, PluginFault> {
    let account = call
        .request
        .credential
        .get("demoAccount")
        .and_then(Value::as_str)
        .unwrap_or("demo");
    let mut item = EvidenceInput::passed("account_management", "profile_queried");
    item.account_id = Some(&call.request.account_id);
    item.provider = Some(PROVIDER_ID);
    evidence.record(item);
    Ok(TypedReply::new(Profile {
        display_name: Some(format!("Demo {}", title_case(account))),
        username: Some(format!("demo-{account}")),
        image_url: None,
        has_stats_error: false,
        summary: ProfileSummary {
            total_text_tokens: Some(1_024),
            peak_tokens: Some(128),
            longest_task_duration_ms: Some(750),
            current_streak_days: Some(2),
            longest_streak_days: Some(4),
        },
        daily_usage: None,
        activity_insights: ProfileActivityInsights::default(),
    }))
}

pub(crate) async fn subscription(
    evidence: &EvidenceLog,
    call: TypedCall<AccountRequest>,
) -> Result<TypedReply<Option<Subscription>>, PluginFault> {
    let mut item = EvidenceInput::passed("account_management", "subscription_queried");
    item.account_id = Some(&call.request.account_id);
    item.provider = Some(PROVIDER_ID);
    evidence.record(item);
    Ok(TypedReply::new(Some(Subscription {
        starts_at_ms: Some(1_735_689_600_000),
        expires_at_ms: DEMO_FUTURE_MS,
        will_renew: Some(false),
        billing_period: Some("demo".to_owned()),
        billing_currency: Some("USD".to_owned()),
        observed_at_ms: 1_735_689_600_000,
    })))
}
