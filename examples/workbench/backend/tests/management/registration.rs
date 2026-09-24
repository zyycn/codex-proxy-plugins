use gateway_plugin_sdk::Stage;
use serde_json::{Value, json};

use crate::support::Peer;

#[tokio::test]
async fn registration_exposes_all_routes_and_only_css_is_public() {
    let mut peer = Peer::start().await;
    let frame = peer
        .call(
            "management.register",
            Stage::Registration,
            json!({}),
            Vec::new(),
        )
        .await;
    let registration: Value = serde_json::from_slice(&frame.payload).unwrap();
    assert_eq!(registration["routes"].as_array().unwrap().len(), 7);
    let public: Vec<_> = registration["resources"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|resource| resource["public"] == true)
        .map(|resource| resource["path"].as_str().unwrap())
        .collect();
    assert_eq!(public, ["web/app.css"]);
}
