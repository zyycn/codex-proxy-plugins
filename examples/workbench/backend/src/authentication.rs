use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    PluginFault,
    call::frontend_authentication::{
        FrontendAuthenticationIdentifier, FrontendAuthenticationRequest,
        FrontendAuthenticationResult,
    },
    client::{Empty, TypedCall, TypedReply},
};

pub(crate) async fn frontend_identifier(
    _call: TypedCall<Empty>,
) -> Result<TypedReply<FrontendAuthenticationIdentifier>, PluginFault> {
    Ok(TypedReply::new(FrontendAuthenticationIdentifier {
        identifier: "capability-workbench".to_owned(),
    }))
}

pub(crate) async fn frontend_authenticate(
    evidence: &EvidenceLog,
    call: TypedCall<FrontendAuthenticationRequest>,
) -> Result<TypedReply<FrontendAuthenticationResult>, PluginFault> {
    let (event, result) = if call.request.authorization == "CapabilityWorkbench demo" {
        (
            "demo_identity_authenticated",
            FrontendAuthenticationResult::Authenticated {
                principal: "capability-workbench-demo-user".to_owned(),
            },
        )
    } else if call
        .request
        .authorization
        .starts_with("CapabilityWorkbench ")
    {
        (
            "invalid_demo_identity_rejected",
            FrontendAuthenticationResult::Rejected {},
        )
    } else {
        (
            "unrelated_identity_delegated",
            FrontendAuthenticationResult::NotMatched {},
        )
    };
    evidence.record(EvidenceInput::passed("frontend_authentication", event));
    Ok(TypedReply::new(result))
}
