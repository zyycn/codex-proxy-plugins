use gateway_plugin_sdk::{
    ErrorCode, PluginFault,
    call::host::{
        AffinityLookupRequest, AffinityLookupResult, AuthListRequest, AuthListResult,
        AuthSaveRequest, AuthSaveResult, HttpRequest, HttpResponse, KeyListRequest, KeyListResult,
        LogRequest, LogResult, ModelListRequest, ModelListResult, StateGetRequest, StateGetResult,
        StatePutRequest, StatePutResult, StreamClose, StreamRead,
    },
    client::{HostClient, SessionError},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};

pub(crate) async fn list_keys(
    host: &HostClient,
    cursor: Option<String>,
    limit: u16,
) -> Result<KeyListResult, PluginFault> {
    metadata(
        host,
        "host.keys.list",
        &KeyListRequest { cursor, limit },
        Vec::new(),
    )
    .await
    .map(|(result, _)| result)
}

pub(crate) async fn list_models(
    host: &HostClient,
    client_key_id: String,
) -> Result<ModelListResult, PluginFault> {
    metadata(
        host,
        "host.models.list",
        &ModelListRequest {
            client_key_id,
            protocol: "openai".to_owned(),
            client_version: "capability-workbench/0.1.0".to_owned(),
        },
        Vec::new(),
    )
    .await
    .map(|(result, _)| result)
}

pub(crate) async fn list_accounts(
    host: &HostClient,
    provider_id: Option<String>,
    cursor: Option<String>,
    limit: u16,
) -> Result<AuthListResult, PluginFault> {
    payload(
        host,
        "host.auth.list",
        &AuthListRequest {
            provider_id,
            cursor,
            limit,
        },
    )
    .await
}

pub(crate) async fn save_account(
    host: &HostClient,
    request: &AuthSaveRequest,
) -> Result<AuthSaveResult, PluginFault> {
    payload(host, "host.auth.save", request).await
}

pub(crate) async fn get_state(
    host: &HostClient,
    namespace: &str,
    key: &str,
) -> Result<StateGetResult, PluginFault> {
    metadata(
        host,
        "host.state.get",
        &StateGetRequest {
            namespace: namespace.to_owned(),
            key: key.to_owned(),
        },
        Vec::new(),
    )
    .await
    .map(|(result, _)| result)
}

pub(crate) async fn put_state(
    host: &HostClient,
    request: &StatePutRequest,
) -> Result<StatePutResult, PluginFault> {
    metadata(host, "host.state.put", request, Vec::new())
        .await
        .map(|(result, _)| result)
}

pub(crate) async fn log(host: &HostClient, request: &LogRequest) -> Result<LogResult, PluginFault> {
    metadata(host, "host.log", request, Vec::new())
        .await
        .map(|(result, _)| result)
}

pub(crate) async fn affinity(
    host: &HostClient,
    provider: String,
    key: String,
) -> Result<AffinityLookupResult, PluginFault> {
    metadata(
        host,
        "host.affinity.lookup",
        &AffinityLookupRequest { provider, key },
        Vec::new(),
    )
    .await
    .map(|(result, _)| result)
}

pub(crate) async fn open_http(
    host: &HostClient,
    request: &HttpRequest,
) -> Result<HttpResponse, PluginFault> {
    let (response, payload): (HttpResponse, Vec<u8>) =
        metadata(host, "host.http.do_stream", request, Vec::new()).await?;
    if !payload.is_empty() || response.stream.is_none() {
        return Err(invalid_callback());
    }
    Ok(response)
}

pub(crate) async fn read_http(
    host: &HostClient,
    stream: String,
    maximum_bytes: u32,
) -> Result<(bool, Vec<u8>), PluginFault> {
    let (result, payload): (Value, Vec<u8>) = metadata(
        host,
        "host.http.stream_read",
        &StreamRead {
            stream,
            maximum_bytes,
        },
        Vec::new(),
    )
    .await?;
    let eof = result
        .as_object()
        .filter(|value| value.len() == 1)
        .and_then(|value| value.get("eof"))
        .and_then(Value::as_bool)
        .ok_or_else(invalid_callback)?;
    if eof && !payload.is_empty() {
        return Err(invalid_callback());
    }
    Ok((eof, payload))
}

pub(crate) async fn close_http(host: &HostClient, stream: String) -> Result<(), PluginFault> {
    let (result, payload): (Map<String, Value>, Vec<u8>) = metadata(
        host,
        "host.http.stream_close",
        &StreamClose { stream },
        Vec::new(),
    )
    .await?;
    if !result.is_empty() || !payload.is_empty() {
        return Err(invalid_callback());
    }
    Ok(())
}

async fn payload<I, O>(host: &HostClient, method: &str, input: &I) -> Result<O, PluginFault>
where
    I: Serialize,
    O: DeserializeOwned,
{
    let reply = host
        .call(
            method,
            serde_json::json!({}),
            serde_json::to_vec(input).map_err(|_| invalid_callback())?,
        )
        .await
        .map_err(SessionError::into_plugin_fault)?;
    if !reply
        .result
        .as_object()
        .is_some_and(serde_json::Map::is_empty)
    {
        return Err(invalid_callback());
    }
    serde_json::from_slice(&reply.payload).map_err(|_| invalid_callback())
}

async fn metadata<I, O>(
    host: &HostClient,
    method: &str,
    input: &I,
    payload: Vec<u8>,
) -> Result<(O, Vec<u8>), PluginFault>
where
    I: Serialize,
    O: DeserializeOwned,
{
    let reply = host
        .call(
            method,
            serde_json::to_value(input).map_err(|_| invalid_callback())?,
            payload,
        )
        .await
        .map_err(SessionError::into_plugin_fault)?;
    let result = serde_json::from_value(reply.result).map_err(|_| invalid_callback())?;
    Ok((result, reply.payload))
}

fn invalid_callback() -> PluginFault {
    PluginFault::new(ErrorCode::Fault, "宿主回调返回了无效响应")
}
