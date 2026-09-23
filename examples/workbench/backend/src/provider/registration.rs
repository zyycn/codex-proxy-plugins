use super::{
    AUTO_MODEL, ECHO_MODEL, PROVIDER_ID, credentials::credential_schema, models::models,
    profiles::request_profiles,
};
use gateway_plugin_sdk::call::{
    auth::CredentialOperation,
    provider::{
        ProviderDescriptor,
        account::{AccountConfigurationDescriptor, AccountOperation},
        billing::{BillingDescriptor, PriceBand, TokenPrices},
        models::ModelDiscovery,
    },
};
use std::collections::BTreeMap;

pub(crate) fn descriptor() -> ProviderDescriptor {
    let credential_operations = vec![
        CredentialOperation::Import,
        CredentialOperation::Rotate,
        CredentialOperation::Refresh,
    ];
    let credential_input_schemas = BTreeMap::from([
        (CredentialOperation::Import, credential_schema(true)),
        (CredentialOperation::Rotate, credential_schema(false)),
    ]);
    ProviderDescriptor {
        id: PROVIDER_ID.to_owned(),
        models: models(),
        credential_operations,
        credential_input_schemas,
        account_configuration: Some(AccountConfigurationDescriptor {
            public_fields: vec!["label".to_owned()],
        }),
        account_operations: vec![
            AccountOperation::FactsChanged,
            AccountOperation::Profile,
            AccountOperation::Subscription,
        ],
        exhaustive: true,
        billing: Some(billing()),
        model_discovery: Some(ModelDiscovery {
            include_static: true,
            cache_ttl_seconds: 300,
        }),
        request_profiles: Some(request_profiles()),
        http_endpoints: Vec::new(),
        continuation_state: None,
    }
}

fn billing() -> BillingDescriptor {
    let prices = TokenPrices {
        input: "0.1000".to_owned(),
        output: "0.2000".to_owned(),
        cache_read: "0.0100".to_owned(),
        cache_write: "0.1000".to_owned(),
    };
    BillingDescriptor::TokenV1 {
        prices: BTreeMap::from([
            (
                ECHO_MODEL.to_owned(),
                BTreeMap::from([(PriceBand::Standard, prices.clone())]),
            ),
            (
                AUTO_MODEL.to_owned(),
                BTreeMap::from([(PriceBand::Standard, prices)]),
            ),
        ]),
    }
}
