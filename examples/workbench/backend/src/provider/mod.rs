mod account;
mod credentials;
mod execution;
mod models;
mod profiles;
mod quota;
mod registration;

pub(crate) use account::{account_changed, account_configuration, profile, subscription};
pub(crate) use credentials::{
    demo_facts, import_credentials, refresh_credentials, rotate_credentials,
};
pub(crate) use execution::ProviderEngine;
pub(crate) use models::discover_models;
pub(crate) use profiles::refresh_request_profiles;
pub(crate) use quota::quota;
pub(crate) use registration::descriptor;

pub(crate) const PROVIDER_ID: &str = "demo";
pub(crate) const ECHO_MODEL: &str = "demo-echo";
pub(crate) const AUTO_MODEL: &str = "demo-auto";

const DEMO_FUTURE_MS: i64 = 4_102_444_800_000;
