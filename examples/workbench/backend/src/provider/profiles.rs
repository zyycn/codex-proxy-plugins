use crate::evidence::{EvidenceInput, EvidenceLog};
use gateway_plugin_sdk::{
    PluginFault,
    call::provider::{
        RequestProfileAttribute, RequestProfileDescriptor, RequestProfileOption,
        RequestProfilePresentation, RequestProfileRefresh, RequestProfileTarget,
    },
    client::{Empty, TypedCall, TypedReply},
};
use serde_json::{Map, json};

pub(crate) async fn refresh_request_profiles(
    evidence: &EvidenceLog,
    _call: TypedCall<Empty>,
) -> Result<TypedReply<RequestProfileRefresh>, PluginFault> {
    evidence.record(EvidenceInput::passed(
        "request_profile",
        "profiles_refreshed",
    ));
    evidence.record(EvidenceInput::passed(
        "maintenance",
        "request_profile_worker",
    ));
    Ok(TypedReply::new(RequestProfileRefresh {
        sequence: 1,
        profiles: request_profiles(),
    }))
}

pub(super) fn request_profiles() -> RequestProfileDescriptor {
    let option = |id: &str, label: &str, style: &str| RequestProfileOption {
        id: id.to_owned(),
        label: label.to_owned(),
        description: Some(format!("{label}演示画像")),
        configuration: Map::from_iter([("profile".to_owned(), json!(id))]),
        resolved: Map::from_iter([("style".to_owned(), json!(style))]),
        presentation: RequestProfilePresentation {
            product: "Capability Workbench".to_owned(),
            version: "0.1.0".to_owned(),
            build: None,
            target: RequestProfileTarget {
                os_type: "demo".to_owned(),
                os_version: "1".to_owned(),
                arch: "portable".to_owned(),
                terminal: "gateway".to_owned(),
            },
            user_agent: format!("capability-workbench/0.1.0 ({style})"),
            attributes: vec![RequestProfileAttribute {
                label: "回显样式".to_owned(),
                value: style.to_owned(),
            }],
            verified_at_ms: Some(1_735_689_600_000),
            release: None,
        },
    };
    RequestProfileDescriptor {
        default_configuration: Map::from_iter([("profile".to_owned(), json!("standard"))]),
        options: vec![
            option("standard", "标准", "unchanged"),
            option("concise", "简洁", "compact"),
        ],
    }
}
