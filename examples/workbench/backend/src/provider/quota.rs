use super::DEMO_FUTURE_MS;
use super::PROVIDER_ID;
use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    PluginFault,
    client::{TypedCall, TypedReply},
};
use gateway_plugin_sdk::{
    Stage,
    call::provider::{
        account::AccountRequest,
        quota::{Quota, QuotaAccess, QuotaWindow, QuotaWindowRole},
    },
};

pub(crate) async fn quota(
    evidence: &EvidenceLog,
    call: TypedCall<AccountRequest>,
) -> Result<TypedReply<Quota>, PluginFault> {
    let mut item = EvidenceInput::passed("quota", "quota_queried");
    item.account_id = Some(&call.request.account_id);
    item.provider = Some(PROVIDER_ID);
    evidence.record(item);
    if call.context.stage == Stage::Maintenance {
        evidence.record(EvidenceInput::passed("maintenance", "quota_worker"));
    }
    Ok(TypedReply::new(Quota {
        plan_type: Some("demo".to_owned()),
        refresh_token_expires_at_ms: None,
        access: QuotaAccess::Allowed,
        windows: vec![QuotaWindow {
            key: "demo-month".to_owned(),
            group: "demo".to_owned(),
            label: "演示月度额度".to_owned(),
            limit_id: Some("demo-month".to_owned()),
            limit_name: Some("演示月度额度".to_owned()),
            role: Some(QuotaWindowRole::Monthly),
            account_wide: true,
            window_seconds: Some(30 * 24 * 60 * 60),
            used_percent: Some(12.5),
            reset_at_ms: Some(DEMO_FUTURE_MS),
            limit_reached: false,
            provider_data: None,
        }],
        provider_data: None,
    }))
}
