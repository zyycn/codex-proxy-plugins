use super::PROVIDER_ID;
use super::{AUTO_MODEL, ECHO_MODEL};
use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    PluginFault,
    client::{TypedCall, TypedReply},
};
use gateway_plugin_sdk::{
    Stage,
    call::provider::{
        ModelDescriptor, OperationKind, account::AccountRequest, models::AccountModels,
    },
};

pub(crate) async fn discover_models(
    evidence: &EvidenceLog,
    call: TypedCall<AccountRequest>,
) -> Result<TypedReply<AccountModels>, PluginFault> {
    let mut item = EvidenceInput::passed("models", "catalog_queried");
    item.account_id = Some(&call.request.account_id);
    item.provider = Some(PROVIDER_ID);
    evidence.record(item);
    if call.context.stage == Stage::Maintenance {
        evidence.record(EvidenceInput::passed("maintenance", "model_catalog_worker"));
    }
    Ok(TypedReply::new(AccountModels {
        models: models(),
        exhaustive: true,
        prepared_account_facts: None,
    }))
}

pub(super) fn models() -> Vec<ModelDescriptor> {
    [ECHO_MODEL, AUTO_MODEL]
        .into_iter()
        .map(|id| ModelDescriptor {
            id: id.to_owned(),
            operations: vec![OperationKind::Generate],
            features: Vec::new(),
            maximum_output_tokens: Some(4_096),
        })
        .collect()
}
