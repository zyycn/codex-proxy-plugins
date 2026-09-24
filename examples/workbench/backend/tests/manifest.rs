use std::collections::BTreeSet;

use codex_proxy_plugin_workbench::{PLUGIN_ID, manifest, plugin};
use gateway_plugin_sdk::{Capability, Permission, Stage};

#[test]
fn checked_in_manifest_is_the_complete_author_contract() {
    let manifest = manifest().unwrap();
    assert_eq!(manifest.plugin_id().unwrap(), PLUGIN_ID);
    assert_eq!(manifest.manifest_version, 3);
    assert_eq!(
        manifest.permissions,
        [
            Permission::Network,
            Permission::Models,
            Permission::Requests,
            Permission::PublicEndpoints
        ]
        .into_iter()
        .collect::<BTreeSet<_>>()
    );
    assert_eq!(manifest.contributes.len(), 9);
    assert_eq!(
        manifest.contributes[&Capability::Middleware].stages,
        [Stage::Request, Stage::Attempt]
    );
    for capability in manifest
        .contributes
        .keys()
        .filter(|capability| **capability != Capability::Middleware)
    {
        assert_eq!(
            manifest.contributes[capability].stages,
            capability.fixed_stages()
        );
    }
}

#[test]
fn capabilities_and_typed_handlers_do_not_drift() {
    plugin().unwrap();
}

#[test]
fn only_the_declared_demo_resource_is_public() {
    let manifest = manifest().unwrap();
    assert_eq!(
        manifest.resources.get("web/app.css").map(String::as_str),
        Some("text/css")
    );
    assert_eq!(manifest.state[0].namespace, "workbench");
}
