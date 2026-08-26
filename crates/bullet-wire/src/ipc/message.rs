use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{RpcRequestId, WireError, canonical_json, decode_canonical_value};

pub const JSON_RPC_VERSION: &str = "2.0";
pub const IPC_PROTOCOL: &str = "bullet.ipc";
pub const IPC_PROTOCOL_VERSION: u16 = 1;
pub const MAX_IPC_FRAME_BYTES: usize = 65_536;
pub const MIN_IPC_FRAME_BYTES: usize = 1_024;
pub const MAX_IPC_DEADLINE_MS: u64 = 60_000;
pub const MAX_IPC_IN_FLIGHT: usize = 64;
pub const MAX_IPC_MESSAGES_PER_CONNECTION: u16 = 4_096;

pub(super) const REQUIRED_FEATURES: [&str; 2] = ["cancellation", "deadlines"];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IpcService {
    Runner,
    Verifier,
    Effects,
    BulletGitd,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IpcHello {
    pub protocol: String,
    pub service: IpcService,
    pub versions: Vec<u16>,
    pub max_frame_bytes: u32,
    pub features: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IpcHelloAck {
    pub protocol: String,
    pub service: IpcService,
    pub selected_version: u16,
    pub max_frame_bytes: u32,
    pub features: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RpcCallParams {
    pub deadline_unix_ms: u64,
    pub body: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RpcCancelParams {
    pub request_id: RpcRequestId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcRequest<T> {
    pub jsonrpc: String,
    pub id: RpcRequestId,
    pub method: String,
    pub params: T,
}

impl<T> JsonRpcRequest<T> {
    pub fn new(id: RpcRequestId, method: impl Into<String>, params: T) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            id,
            method: method.into(),
            params,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcNotification<T> {
    pub jsonrpc: String,
    pub method: String,
    pub params: T,
}

impl<T> JsonRpcNotification<T> {
    pub fn new(method: impl Into<String>, params: T) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            method: method.into(),
            params,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcSuccess<T> {
    pub jsonrpc: String,
    pub id: RpcRequestId,
    pub result: T,
}

impl<T> JsonRpcSuccess<T> {
    pub fn new(id: RpcRequestId, result: T) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            id,
            result,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcErrorBody {
    pub code: i32,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonRpcFailure {
    pub jsonrpc: String,
    pub id: RpcRequestId,
    pub error: JsonRpcErrorBody,
}

impl JsonRpcFailure {
    pub fn new(id: RpcRequestId, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: JSON_RPC_VERSION.to_owned(),
            id,
            error: JsonRpcErrorBody {
                code,
                message: message.into(),
                data: None,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RpcResponse {
    Success {
        request_id: RpcRequestId,
        result: Value,
    },
    Failure {
        request_id: RpcRequestId,
        error: JsonRpcErrorBody,
    },
}

impl RpcResponse {
    pub fn request_id(&self) -> &RpcRequestId {
        match self {
            Self::Success { request_id, .. } | Self::Failure { request_id, .. } => request_id,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawRequest {
    pub jsonrpc: String,
    pub id: RpcRequestId,
    pub method: String,
    pub params: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
}

pub(super) fn decode_message(frame: &[u8], limit: usize) -> Result<Value, WireError> {
    validate_frame(frame, limit)?;
    decode_canonical_value(frame)
}

pub(super) fn decode_response(value: Value) -> Result<RpcResponse, WireError> {
    let object = value
        .as_object()
        .ok_or_else(|| WireError::new("IPC_MESSAGE_INVALID", "JSON-RPC frame must be an object"))?;
    match (object.contains_key("result"), object.contains_key("error")) {
        (true, false) => {
            let response: JsonRpcSuccess<Value> = strict_value(value)?;
            validate_jsonrpc(&response.jsonrpc)?;
            Ok(RpcResponse::Success {
                request_id: response.id,
                result: response.result,
            })
        }
        (false, true) => {
            let response: JsonRpcFailure = strict_value(value)?;
            validate_jsonrpc(&response.jsonrpc)?;
            Ok(RpcResponse::Failure {
                request_id: response.id,
                error: response.error,
            })
        }
        _ => Err(WireError::new(
            "IPC_RESPONSE_INVALID",
            "response must contain exactly one of result or error",
        )),
    }
}

pub(super) fn validate_frame(frame: &[u8], limit: usize) -> Result<(), WireError> {
    if frame.len() > limit {
        return Err(WireError::new(
            "IPC_FRAME_TOO_LARGE",
            format!(
                "frame is {} bytes; negotiated maximum is {limit}",
                frame.len()
            ),
        ));
    }
    if frame.contains(&b'\n') || frame.contains(&b'\r') {
        return Err(WireError::new(
            "IPC_FRAME_BOUNDARY_INVALID",
            "the decoder accepts exactly one JSONL frame without its line terminator",
        ));
    }
    Ok(())
}

pub(super) fn validate_jsonrpc(version: &str) -> Result<(), WireError> {
    if version != JSON_RPC_VERSION {
        return Err(WireError::new(
            "IPC_JSONRPC_VERSION_UNSUPPORTED",
            "jsonrpc must be exactly 2.0",
        ));
    }
    Ok(())
}

pub(super) fn validate_hello(hello: &IpcHello) -> Result<(), WireError> {
    if hello.protocol != IPC_PROTOCOL {
        return Err(WireError::new(
            "IPC_PROTOCOL_UNSUPPORTED",
            "hello protocol must be bullet.ipc",
        ));
    }
    if hello.versions.is_empty() || hello.versions.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(WireError::new(
            "IPC_PROTOCOL_VERSIONS_INVALID",
            "offered protocol versions must be nonempty, sorted, and unique",
        ));
    }
    if !hello.versions.contains(&IPC_PROTOCOL_VERSION) {
        return Err(WireError::new(
            "IPC_PROTOCOL_VERSION_UNSUPPORTED",
            "peer does not offer IPC protocol version 1",
        ));
    }
    validate_frame_limit(usize::try_from(hello.max_frame_bytes).unwrap_or(usize::MAX))?;
    validate_features(&hello.features)
}

pub(super) fn validate_features(features: &[String]) -> Result<(), WireError> {
    if features.is_empty()
        || features.windows(2).any(|pair| pair[0] >= pair[1])
        || features.iter().any(|feature| {
            feature.is_empty()
                || !feature
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        })
    {
        return Err(WireError::new(
            "IPC_FEATURES_INVALID",
            "features must be nonempty, sorted, unique lowercase ASCII labels",
        ));
    }
    if REQUIRED_FEATURES
        .iter()
        .any(|required| !features.iter().any(|feature| feature == required))
    {
        return Err(WireError::new(
            "IPC_REQUIRED_FEATURE_MISSING",
            "deadlines and cancellation are mandatory protocol features",
        ));
    }
    Ok(())
}

pub(super) fn validate_method(method: &str) -> Result<(), WireError> {
    if method.len() > 128
        || !method.starts_with("bullet.")
        || !method.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        })
    {
        return Err(WireError::new(
            "IPC_METHOD_INVALID",
            "method must be a bounded lowercase bullet.* label",
        ));
    }
    Ok(())
}

pub(super) fn validate_deadline(deadline_unix_ms: u64, now_unix_ms: u64) -> Result<(), WireError> {
    let remaining = deadline_unix_ms.checked_sub(now_unix_ms).ok_or_else(|| {
        WireError::new(
            "IPC_DEADLINE_EXPIRED",
            "request deadline has already passed",
        )
    })?;
    if remaining == 0 {
        return Err(WireError::new(
            "IPC_DEADLINE_EXPIRED",
            "request deadline has already passed",
        ));
    }
    if remaining > MAX_IPC_DEADLINE_MS {
        return Err(WireError::new(
            "IPC_DEADLINE_TOO_FAR",
            "request deadline exceeds the admitted call horizon",
        ));
    }
    Ok(())
}

pub(super) fn validate_frame_limit(limit: usize) -> Result<(), WireError> {
    if !(MIN_IPC_FRAME_BYTES..=MAX_IPC_FRAME_BYTES).contains(&limit) {
        return Err(WireError::new(
            "IPC_FRAME_LIMIT_INVALID",
            format!(
                "frame maximum must be between {MIN_IPC_FRAME_BYTES} and {MAX_IPC_FRAME_BYTES}"
            ),
        ));
    }
    Ok(())
}

pub(super) fn strict_value<T>(value: Value) -> Result<T, WireError>
where
    T: for<'de> Deserialize<'de>,
{
    serde_json::from_value(value).map_err(|error| {
        WireError::new(
            "IPC_MESSAGE_INVALID",
            format!("JSON-RPC message does not match its strict schema: {error}"),
        )
    })
}

pub fn encode_jsonl<T>(message: &T, negotiated_frame_bytes: usize) -> Result<Vec<u8>, WireError>
where
    T: Serialize,
{
    validate_frame_limit(negotiated_frame_bytes)?;
    let mut frame = canonical_json(message)?;
    validate_frame(&frame, negotiated_frame_bytes)?;
    frame.push(b'\n');
    Ok(frame)
}
