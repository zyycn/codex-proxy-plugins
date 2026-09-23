use crate::{
    authentication, command,
    evidence::EvidenceLog,
    management,
    provider::{self, ProviderEngine},
    request::{self, ScopeTracker},
};
use gateway_plugin_sdk::client::{AuthorError, ComposedPlugin, PluginBuilder, methods};
use std::sync::Arc;

/// 通过公开的类型化 SDK 组装工作台处理器。
///
/// # 错误
///
/// 清单、服务商描述与处理器声明不一致时返回错误。
pub fn plugin() -> Result<ComposedPlugin, AuthorError> {
    let evidence = Arc::new(EvidenceLog::default());
    let scope = Arc::new(ScopeTracker::default());
    let provider_engine = Arc::new(ProviderEngine::new(Arc::clone(&evidence)));
    let middleware_evidence = Arc::clone(&evidence);
    let middleware_scope = Arc::clone(&scope);
    let route_evidence = Arc::clone(&evidence);
    let route_scope = Arc::clone(&scope);
    let schedule_evidence = Arc::clone(&evidence);
    let request_evidence = Arc::clone(&evidence);
    let request_scope = Arc::clone(&scope);
    let websocket_evidence = Arc::clone(&evidence);
    let websocket_scope = Arc::clone(&scope);
    let command_evidence = Arc::clone(&evidence);
    let frontend_evidence = Arc::clone(&evidence);
    let management_evidence = Arc::clone(&evidence);
    let prepare_engine = Arc::clone(&provider_engine);
    let execute_engine = Arc::clone(&provider_engine);
    let discard_engine = Arc::clone(&provider_engine);
    let connection_engine = Arc::clone(&provider_engine);
    let models_evidence = Arc::clone(&evidence);
    let quota_evidence = Arc::clone(&evidence);
    let import_evidence = Arc::clone(&evidence);
    let rotate_evidence = Arc::clone(&evidence);
    let refresh_evidence = Arc::clone(&evidence);
    let configuration_evidence = Arc::clone(&evidence);
    let changed_evidence = Arc::clone(&evidence);
    let profile_evidence = Arc::clone(&evidence);
    let subscription_evidence = Arc::clone(&evidence);
    let profiles_evidence = Arc::clone(&evidence);

    PluginBuilder::from_json(include_bytes!("../../plugin.json"))?
        .provider(provider::descriptor())
        .middleware(move |call| {
            let evidence = Arc::clone(&middleware_evidence);
            let scope = Arc::clone(&middleware_scope);
            async move { request::middleware(evidence, scope, call).await }
        })?
        .on(methods::ROUTE_MODEL, move |call| {
            let evidence = Arc::clone(&route_evidence);
            let scope = Arc::clone(&route_scope);
            async move { request::route_model(&evidence, &scope, call).await }
        })?
        .on(methods::SCHEDULE_ACCOUNT, move |call| {
            let evidence = Arc::clone(&schedule_evidence);
            async move { request::schedule_account(&evidence, call).await }
        })?
        .on(methods::OBSERVE_REQUEST, move |call| {
            let evidence = Arc::clone(&request_evidence);
            let scope = Arc::clone(&request_scope);
            async move { request::observe_request(&evidence, &scope, call).await }
        })?
        .on(methods::OBSERVE_WEBSOCKET, move |call| {
            let evidence = Arc::clone(&websocket_evidence);
            let scope = Arc::clone(&websocket_scope);
            async move { request::observe_websocket(&evidence, &scope, call).await }
        })?
        .on(
            methods::FRONTEND_IDENTIFIER,
            authentication::frontend_identifier,
        )?
        .on(methods::FRONTEND_AUTHENTICATE, move |call| {
            let evidence = Arc::clone(&frontend_evidence);
            async move { authentication::frontend_authenticate(&evidence, call).await }
        })?
        .management(management::registration(), move |call| {
            let evidence = Arc::clone(&management_evidence);
            async move { management::handle(evidence, call).await }
        })?
        .command_line(command::command_registration(), move |call| {
            let evidence = Arc::clone(&command_evidence);
            async move { command::command_line(&evidence, call).await }
        })?
        .on(methods::PREPARE_EXECUTION, move |call| {
            let engine = Arc::clone(&prepare_engine);
            async move { engine.prepare(call).await }
        })?
        .on(methods::EXECUTE, move |call| {
            let engine = Arc::clone(&execute_engine);
            async move { engine.execute(call).await }
        })?
        .on(methods::DISCARD_EXECUTION, move |call| {
            let engine = Arc::clone(&discard_engine);
            async move { engine.discard(call).await }
        })?
        .on(methods::CONNECTION_TEST, move |call| {
            let engine = Arc::clone(&connection_engine);
            async move { engine.connection_test(call).await }
        })?
        .on(methods::MODELS, move |call| {
            let evidence = Arc::clone(&models_evidence);
            async move { provider::discover_models(&evidence, call).await }
        })?
        .on(methods::QUOTA, move |call| {
            let evidence = Arc::clone(&quota_evidence);
            async move { provider::quota(&evidence, call).await }
        })?
        .on(methods::IMPORT_CREDENTIALS, move |call| {
            let evidence = Arc::clone(&import_evidence);
            async move { provider::import_credentials(&evidence, call).await }
        })?
        .on(methods::ROTATE_CREDENTIALS, move |call| {
            let evidence = Arc::clone(&rotate_evidence);
            async move { provider::rotate_credentials(&evidence, call).await }
        })?
        .on(methods::REFRESH_CREDENTIALS, move |call| {
            let evidence = Arc::clone(&refresh_evidence);
            async move { provider::refresh_credentials(&evidence, call).await }
        })?
        .on(methods::ACCOUNT_CONFIGURATION, move |call| {
            let evidence = Arc::clone(&configuration_evidence);
            async move { provider::account_configuration(&evidence, call).await }
        })?
        .on(methods::ACCOUNT_CHANGED, move |call| {
            let evidence = Arc::clone(&changed_evidence);
            async move { provider::account_changed(&evidence, call).await }
        })?
        .on(methods::PROFILE, move |call| {
            let evidence = Arc::clone(&profile_evidence);
            async move { provider::profile(&evidence, call).await }
        })?
        .on(methods::SUBSCRIPTION, move |call| {
            let evidence = Arc::clone(&subscription_evidence);
            async move { provider::subscription(&evidence, call).await }
        })?
        .on(methods::REFRESH_REQUEST_PROFILES, move |call| {
            let evidence = Arc::clone(&profiles_evidence);
            async move { provider::refresh_request_profiles(&evidence, call).await }
        })?
        .build()
}
