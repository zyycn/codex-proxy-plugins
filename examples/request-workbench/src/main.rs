use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use gateway_plugin_sdk::{
    CallContext, Capability, ContributionDeclaration, Contributions, ErrorCode, PluginFault, Stage,
    call::{
        management::{
            ManagementPage, ManagementRegistration, ManagementRequest, ManagementResource,
            ManagementResponse, ManagementRoute,
        },
        provider::Registration,
    },
    client::{
        CallFuture, CallReply, MiddlewareCall, MiddlewarePlugin, PluginCall, PluginHandler,
        PluginSession, SessionConfig, SessionError,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const PLUGIN_ID: &str = "codex-proxy.request-workbench";
const HEADER_NAME: &str = "x-cpr-example";
const DEFAULT_HEADER_VALUE: &str = "official-example";
const JSON_CONTENT_TYPE: &str = "application/json";
const ICON_PATH: &str = "assets/icon.png";
const MAXIMUM_ECHO_BYTES: usize = 256;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Configuration {
    #[serde(default = "default_header_value")]
    header_value: String,
}

impl Configuration {
    fn from_value(value: &Value) -> Result<Self, SessionError> {
        let configuration: Self =
            serde_json::from_value(value.clone()).map_err(|_| SessionError::Configuration)?;
        if configuration.header_value.is_empty()
            || configuration.header_value.len() > 128
            || configuration.header_value.trim() != configuration.header_value
            || !configuration
                .header_value
                .bytes()
                .all(|byte| (b' '..=b'~').contains(&byte))
        {
            return Err(SessionError::Configuration);
        }
        Ok(configuration)
    }
}

fn default_header_value() -> String {
    DEFAULT_HEADER_VALUE.to_owned()
}

struct WorkbenchState {
    header_value: Arc<str>,
    handled_requests: AtomicU64,
}

impl WorkbenchState {
    fn new(header_value: String) -> Self {
        Self {
            header_value: header_value.into(),
            handled_requests: AtomicU64::new(0),
        }
    }
}

struct WorkbenchPlugin<M> {
    middleware: M,
    contributes: Contributions,
    state: Arc<WorkbenchState>,
}

impl<M> WorkbenchPlugin<M> {
    fn new(middleware: M, contributes: Contributions, state: Arc<WorkbenchState>) -> Self {
        Self {
            middleware,
            contributes,
            state,
        }
    }

    fn register(&self, call: PluginCall) -> Result<CallReply, PluginFault> {
        require_registration_call(&call)?;
        let registration = Registration {
            contributes: self.contributes.clone(),
            provider: None,
        };
        Ok(CallReply::unary(to_value(registration)?, Vec::new()))
    }

    fn register_management(&self, call: PluginCall) -> Result<CallReply, PluginFault> {
        require_registration_call(&call)?;
        let registration = management_registration();
        Ok(CallReply::unary(json!({}), to_json_bytes(&registration)?))
    }

    fn handle_management(&self, call: PluginCall) -> Result<CallReply, PluginFault> {
        if call.context.stage != Stage::Management {
            return Err(invalid_input("management call has an invalid stage"));
        }
        let request: ManagementRequest =
            serde_json::from_value(call.params).map_err(|_| invalid_management_request())?;
        if !request.query.is_empty() {
            return json_reply(
                400,
                &ErrorResponse {
                    error: "query parameters are not supported",
                },
            );
        }
        match (request.method.as_str(), request.path.as_str()) {
            ("GET", "status") => {
                if request.content_type.is_some() || !call.payload.is_empty() {
                    return json_reply(
                        400,
                        &ErrorResponse {
                            error: "status does not accept a request body",
                        },
                    );
                }
                json_reply(
                    200,
                    &StatusResponse {
                        header_value: &self.state.header_value,
                        handled_requests: self.state.handled_requests.load(Ordering::Relaxed),
                    },
                )
            }
            ("POST", "echo") => {
                if request.content_type.as_deref() != Some(JSON_CONTENT_TYPE) {
                    return json_reply(
                        400,
                        &ErrorResponse {
                            error: "echo requires application/json",
                        },
                    );
                }
                match decode_echo(&call.payload) {
                    Ok(message) => json_reply(200, &message),
                    Err(error) => json_reply(400, &error),
                }
            }
            _ => Err(PluginFault::new(
                ErrorCode::Unsupported,
                "management route is not supported",
            )),
        }
    }
}

impl<M> PluginHandler for WorkbenchPlugin<M>
where
    M: PluginHandler,
{
    fn call(&self, call: PluginCall) -> CallFuture<'_> {
        if call.method == "middleware.handle" {
            return self.middleware.call(call);
        }
        Box::pin(async move {
            match call.method.as_str() {
                "plugin.register" => self.register(call),
                "management.register" => self.register_management(call),
                "management.handle" => self.handle_management(call),
                _ => Err(PluginFault::new(
                    ErrorCode::Unsupported,
                    "plugin method is not supported",
                )),
            }
        })
    }

    fn cancel(&self, context: &CallContext) {
        self.middleware.cancel(context);
    }

    fn quiesce(&self) {
        self.middleware.quiesce();
    }

    fn shutdown(&self) {
        self.middleware.shutdown();
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusResponse<'a> {
    header_value: &'a str,
    handled_requests: u64,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct EchoMessage {
    message: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct ErrorResponse {
    error: &'static str,
}

fn decode_echo(payload: &[u8]) -> Result<EchoMessage, ErrorResponse> {
    let message: EchoMessage = serde_json::from_slice(payload).map_err(|_| ErrorResponse {
        error: "body must be a JSON object containing only message",
    })?;
    if message.message.is_empty()
        || message.message.len() > MAXIMUM_ECHO_BYTES
        || message.message.chars().any(char::is_control)
    {
        return Err(ErrorResponse {
            error: "message must be 1 to 256 UTF-8 bytes without control characters",
        });
    }
    Ok(message)
}

fn json_reply(body_status: u16, body: &impl Serialize) -> Result<CallReply, PluginFault> {
    let response = ManagementResponse {
        status: body_status,
        content_type: JSON_CONTENT_TYPE.to_owned(),
    };
    Ok(CallReply::unary(to_value(response)?, to_json_bytes(body)?))
}

fn require_registration_call(call: &PluginCall) -> Result<(), PluginFault> {
    if call.context.stage != Stage::Registration
        || !call
            .params
            .as_object()
            .is_some_and(serde_json::Map::is_empty)
        || !call.payload.is_empty()
    {
        return Err(invalid_input("registration call is invalid"));
    }
    Ok(())
}

fn to_value(value: impl Serialize) -> Result<Value, PluginFault> {
    serde_json::to_value(value).map_err(|_| encoding_fault())
}

fn to_json_bytes(value: &impl Serialize) -> Result<Vec<u8>, PluginFault> {
    serde_json::to_vec(value).map_err(|_| encoding_fault())
}

fn invalid_management_request() -> PluginFault {
    invalid_input("management request is invalid")
}

fn invalid_input(message: &'static str) -> PluginFault {
    PluginFault::new(ErrorCode::InvalidInput, message)
}

fn encoding_fault() -> PluginFault {
    PluginFault::new(ErrorCode::Fault, "plugin response could not be encoded")
}

fn contributions() -> Contributions {
    Contributions::from([
        (
            Capability::Middleware,
            ContributionDeclaration {
                id: format!("{PLUGIN_ID}.middleware"),
                version: 1,
                stages: vec![Stage::Request],
                input_formats: vec!["openai".to_owned()],
                output_formats: vec!["openai".to_owned()],
            },
        ),
        (
            Capability::Management,
            ContributionDeclaration {
                id: format!("{PLUGIN_ID}.management"),
                version: 1,
                stages: vec![Stage::Management],
                input_formats: Vec::new(),
                output_formats: Vec::new(),
            },
        ),
    ])
}

fn management_registration() -> ManagementRegistration {
    ManagementRegistration {
        routes: vec![
            ManagementRoute {
                method: "GET".to_owned(),
                path: "status".to_owned(),
                provider_id: None,
                request_content_types: Vec::new(),
                response_content_types: vec![JSON_CONTENT_TYPE.to_owned()],
            },
            ManagementRoute {
                method: "POST".to_owned(),
                path: "echo".to_owned(),
                provider_id: None,
                request_content_types: vec![JSON_CONTENT_TYPE.to_owned()],
                response_content_types: vec![JSON_CONTENT_TYPE.to_owned()],
            },
        ],
        resources: ["web/index.html", "web/app.js", "web/app.css", ICON_PATH]
            .into_iter()
            .map(|path| ManagementResource {
                path: path.to_owned(),
                public: false,
            })
            .collect(),
        pages: vec![ManagementPage {
            id: "request-workbench".to_owned(),
            title: "请求工作台".to_owned(),
            description: Some("查看请求处理状态与响应标记，发送消息验证插件调用".to_owned()),
            entry: "web/index.html".to_owned(),
            icon: Some(ICON_PATH.to_owned()),
        }],
        callbacks: Vec::new(),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = PluginSession::accept(
        tokio::io::stdin(),
        tokio::io::stdout(),
        SessionConfig::default(),
    )
    .await?;
    let contributes = contributions();
    if session.handshake().plugin_id != PLUGIN_ID || session.handshake().contributes != contributes
    {
        return Err(SessionError::Handshake.into());
    }
    let configuration = Configuration::from_value(&session.handshake().configuration)?;
    let state = Arc::new(WorkbenchState::new(configuration.header_value));
    let middleware_contributes = Contributions::from([(
        Capability::Middleware,
        contributes
            .get(&Capability::Middleware)
            .cloned()
            .ok_or(SessionError::Configuration)?,
    )]);
    let middleware_state = Arc::clone(&state);
    let middleware =
        MiddlewarePlugin::new(&middleware_contributes, move |call: MiddlewareCall| {
            let state = Arc::clone(&middleware_state);
            async move {
                state.handled_requests.fetch_add(1, Ordering::Relaxed);
                let mut response = call.next.run(call.request).await?;
                response.append_header(HEADER_NAME, state.header_value.as_bytes().to_vec());
                Ok(response)
            }
        })?;
    session
        .run(WorkbenchPlugin::new(middleware, contributes, state))
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gateway_plugin_sdk::{Manifest, Permission, PluginIcon};

    #[test]
    fn configuration_defaults_and_rejects_unsafe_header_values() {
        let configuration = Configuration::from_value(&json!({})).unwrap();
        assert_eq!(configuration.header_value, DEFAULT_HEADER_VALUE);
        assert!(Configuration::from_value(&json!({"headerValue":"line\nbreak"})).is_err());
        assert!(Configuration::from_value(&json!({"headerValue":" value"})).is_err());
    }

    #[test]
    fn echo_is_bounded_and_does_not_accept_extra_fields() {
        assert_eq!(
            decode_echo(br#"{"message":"hello"}"#).unwrap(),
            EchoMessage {
                message: "hello".to_owned()
            }
        );
        assert!(decode_echo(br#"{"message":"hello","secret":"no"}"#).is_err());
        assert!(decode_echo(br#"{"message":""}"#).is_err());
        let oversized = serde_json::to_vec(&json!({"message":"x".repeat(257)})).unwrap();
        assert!(decode_echo(&oversized).is_err());
    }

    #[test]
    fn registrations_match_the_source_manifest_contract() {
        let contributes = contributions();
        assert_eq!(contributes.len(), 2);
        assert_eq!(
            contributes[&Capability::Middleware].id,
            "codex-proxy.request-workbench.middleware"
        );
        let management = management_registration();
        assert_eq!(management.routes.len(), 2);
        assert_eq!(management.resources.len(), 4);
        assert_eq!(management.pages[0].entry, "web/index.html");
        assert_eq!(management.pages[0].icon.as_deref(), Some(ICON_PATH));

        let manifest: Manifest = serde_json::from_str(include_str!("../plugin.json")).unwrap();
        manifest.validate().unwrap();
        assert_eq!(manifest.plugin_id().unwrap(), PLUGIN_ID);
        assert_eq!(manifest.contributes, contributes);
        assert_eq!(manifest.icon, Some(PluginIcon::Path(ICON_PATH.to_owned())));
        assert_eq!(
            manifest.permissions,
            [Permission::ResponseHeadersWrite].into_iter().collect()
        );
        assert!(manifest.package.is_none());
        let mut registered_resources = management
            .resources
            .into_iter()
            .map(|resource| resource.path)
            .collect::<Vec<_>>();
        registered_resources.sort();
        assert_eq!(
            registered_resources,
            manifest
                .resources
                .keys()
                .filter(|path| path.as_str() != "LICENSE")
                .cloned()
                .collect::<Vec<_>>()
        );
        assert_eq!(manifest.resources["LICENSE"], "text/plain");
    }
}
