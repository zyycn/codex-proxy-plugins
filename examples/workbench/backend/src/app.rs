use crate::{
    authentication, command,
    evidence::EvidenceLog,
    management,
    request::{self, ScopeTracker},
};
use gateway_plugin_sdk::client::{AuthorError, ComposedPlugin, PluginBuilder, methods};
use std::sync::Arc;

/// 通过公开的类型化 SDK 组装工作台处理器。
///
/// # 错误
///
/// 清单与处理器声明不一致时返回错误。
pub fn plugin() -> Result<ComposedPlugin, AuthorError> {
    let evidence = Arc::new(EvidenceLog::default());
    let scope = Arc::new(ScopeTracker::default());
    let middleware_evidence = Arc::clone(&evidence);
    let middleware_scope = Arc::clone(&scope);
    let route_evidence = Arc::clone(&evidence);
    let route_scope = Arc::clone(&scope);
    let schedule_evidence = Arc::clone(&evidence);
    let schedule_scope = Arc::clone(&scope);
    let request_evidence = Arc::clone(&evidence);
    let request_scope = Arc::clone(&scope);
    let websocket_evidence = Arc::clone(&evidence);
    let websocket_scope = Arc::clone(&scope);
    let command_evidence = Arc::clone(&evidence);
    let frontend_evidence = Arc::clone(&evidence);
    let management_evidence = Arc::clone(&evidence);

    PluginBuilder::from_json(include_bytes!("../../plugin.json"))?
        .middleware(move |call| {
            let evidence = Arc::clone(&middleware_evidence);
            let scope = Arc::clone(&middleware_scope);
            async move { request::middleware(&evidence, &scope, call).await }
        })?
        .on(methods::ROUTE_MODEL, move |call| {
            let evidence = Arc::clone(&route_evidence);
            let scope = Arc::clone(&route_scope);
            async move { request::route_model(&evidence, &scope, call).await }
        })?
        .on(methods::SCHEDULE_ACCOUNT, move |call| {
            let evidence = Arc::clone(&schedule_evidence);
            let scope = Arc::clone(&schedule_scope);
            async move { request::schedule_account(&evidence, &scope, call).await }
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
            async move { management::handle(&evidence, call).await }
        })?
        .command_line(command::command_registration(), move |call| {
            let evidence = Arc::clone(&command_evidence);
            async move { command::command_line(&evidence, call).await }
        })?
        .build()
}
