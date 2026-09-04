use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::message::{
    IPC_PROTOCOL, IPC_PROTOCOL_VERSION, IpcHello, IpcHelloAck, IpcService, JsonRpcNotification,
    JsonRpcRequest, MAX_IPC_FRAME_BYTES, MAX_IPC_IN_FLIGHT, MAX_IPC_MESSAGES_PER_CONNECTION,
    REQUIRED_FEATURES, RpcCallParams, RpcCancelParams, RpcResponse, decode_message,
    decode_response, encode_jsonl, strict_value, validate_deadline, validate_features,
    validate_frame_limit, validate_method,
};
use crate::{RpcRequestId, WireError};

#[derive(Clone, Debug, Eq, PartialEq)]
enum ClientState {
    New,
    HelloPending {
        request_id: RpcRequestId,
        offered_frame_bytes: usize,
    },
    Ready {
        negotiated_frame_bytes: usize,
    },
    Closed,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RpcClientSession {
    local_service: IpcService,
    expected_peer: IpcService,
    state: ClientState,
    messages: u16,
    seen_request_ids: BTreeSet<RpcRequestId>,
    active_requests: BTreeMap<RpcRequestId, bool>,
}

impl RpcClientSession {
    pub fn new(local_service: IpcService, expected_peer: IpcService) -> Self {
        Self {
            local_service,
            expected_peer,
            state: ClientState::New,
            messages: 0,
            seen_request_ids: BTreeSet::new(),
            active_requests: BTreeMap::new(),
        }
    }

    pub fn start_hello(
        &mut self,
        request_id: RpcRequestId,
        offered_frame_bytes: usize,
    ) -> Result<Vec<u8>, WireError> {
        self.apply(|session| {
            if session.state != ClientState::New {
                return Err(WireError::new(
                    "IPC_HELLO_STATE_INVALID",
                    "hello may be sent exactly once as the first message",
                ));
            }
            validate_frame_limit(offered_frame_bytes)?;
            count_message(&mut session.messages)?;
            session.remember_request(&request_id)?;
            let hello = IpcHello {
                protocol: IPC_PROTOCOL.to_owned(),
                service: session.local_service,
                versions: vec![IPC_PROTOCOL_VERSION],
                max_frame_bytes: u32::try_from(offered_frame_bytes)
                    .expect("the fixed IPC frame bound fits u32"),
                features: REQUIRED_FEATURES.iter().map(ToString::to_string).collect(),
            };
            let encoded = encode_jsonl(
                &JsonRpcRequest::new(request_id.clone(), "bullet.hello", hello),
                MAX_IPC_FRAME_BYTES,
            )?;
            session.state = ClientState::HelloPending {
                request_id,
                offered_frame_bytes,
            };
            Ok(encoded)
        })
    }

    pub fn start_call(
        &mut self,
        request_id: RpcRequestId,
        method: impl Into<String>,
        deadline_unix_ms: u64,
        now_unix_ms: u64,
        body: Value,
    ) -> Result<Vec<u8>, WireError> {
        let method = method.into();
        self.apply(|session| {
            let frame_limit = session.ready_frame_limit()?;
            validate_method(&method)?;
            if matches!(method.as_str(), "bullet.hello" | "bullet.cancel") {
                return Err(WireError::new(
                    "IPC_METHOD_INVALID",
                    "hello and cancellation have dedicated constructors",
                ));
            }
            validate_deadline(deadline_unix_ms, now_unix_ms)?;
            if session.active_requests.len() >= MAX_IPC_IN_FLIGHT {
                return Err(WireError::new(
                    "IPC_IN_FLIGHT_LIMIT",
                    "connection has too many active requests",
                ));
            }
            count_message(&mut session.messages)?;
            session.remember_request(&request_id)?;
            let encoded = encode_jsonl(
                &JsonRpcRequest::new(
                    request_id.clone(),
                    method,
                    RpcCallParams {
                        deadline_unix_ms,
                        body,
                    },
                ),
                frame_limit,
            )?;
            session.active_requests.insert(request_id, false);
            Ok(encoded)
        })
    }

    pub fn cancel(&mut self, request_id: &RpcRequestId) -> Result<Vec<u8>, WireError> {
        self.apply(|session| {
            let frame_limit = session.ready_frame_limit()?;
            let cancelled = session.active_requests.get_mut(request_id).ok_or_else(|| {
                WireError::new(
                    "IPC_CANCEL_UNKNOWN",
                    "cancellation must reference an active request",
                )
            })?;
            if *cancelled {
                return Err(WireError::new(
                    "IPC_CANCEL_DUPLICATE",
                    "an active request may be cancelled only once",
                ));
            }
            count_message(&mut session.messages)?;
            let encoded = encode_jsonl(
                &JsonRpcNotification::new(
                    "bullet.cancel",
                    RpcCancelParams {
                        request_id: request_id.clone(),
                    },
                ),
                frame_limit,
            )?;
            *cancelled = true;
            Ok(encoded)
        })
    }

    pub fn accept_response(&mut self, frame: &[u8]) -> Result<RpcResponse, WireError> {
        self.apply(|session| {
            count_message(&mut session.messages)?;
            let frame_limit = match session.state {
                ClientState::HelloPending {
                    offered_frame_bytes,
                    ..
                }
                | ClientState::Ready {
                    negotiated_frame_bytes: offered_frame_bytes,
                } => offered_frame_bytes,
                ClientState::New => {
                    return Err(WireError::new(
                        "IPC_HELLO_REQUIRED",
                        "cannot receive a response before sending hello",
                    ));
                }
                ClientState::Closed => return Err(closed()),
            };
            let response = decode_response(decode_message(frame, frame_limit)?)?;
            match session.state.clone() {
                ClientState::HelloPending {
                    request_id,
                    offered_frame_bytes,
                } => session.accept_hello_response(response, &request_id, offered_frame_bytes),
                ClientState::Ready { .. } => session.accept_call_response(response),
                ClientState::New | ClientState::Closed => Err(closed()),
            }
        })
    }

    pub fn negotiated_frame_bytes(&self) -> Option<usize> {
        match self.state {
            ClientState::Ready {
                negotiated_frame_bytes,
            } => Some(negotiated_frame_bytes),
            ClientState::New | ClientState::HelloPending { .. } | ClientState::Closed => None,
        }
    }

    fn accept_hello_response(
        &mut self,
        response: RpcResponse,
        hello_id: &RpcRequestId,
        offered_frame_bytes: usize,
    ) -> Result<RpcResponse, WireError> {
        if response.request_id() != hello_id {
            return Err(WireError::new(
                "IPC_RESPONSE_ID_MISMATCH",
                "hello response does not match the pending request",
            ));
        }
        let RpcResponse::Success { ref result, .. } = response else {
            return Err(WireError::new(
                "IPC_HELLO_REFUSED",
                "peer returned an error for protocol negotiation",
            ));
        };
        let acknowledgement: IpcHelloAck = strict_value(result.clone())?;
        self.validate_acknowledgement(&acknowledgement, offered_frame_bytes)?;
        self.state = ClientState::Ready {
            negotiated_frame_bytes: usize::try_from(acknowledgement.max_frame_bytes)
                .expect("u32 fits usize on supported platforms"),
        };
        Ok(response)
    }

    fn accept_call_response(&mut self, response: RpcResponse) -> Result<RpcResponse, WireError> {
        if self.active_requests.remove(response.request_id()).is_none() {
            return Err(WireError::new(
                "IPC_RESPONSE_ID_UNKNOWN",
                "response does not match an active request",
            ));
        }
        Ok(response)
    }

    fn validate_acknowledgement(
        &self,
        acknowledgement: &IpcHelloAck,
        offered_frame_bytes: usize,
    ) -> Result<(), WireError> {
        if acknowledgement.protocol != IPC_PROTOCOL
            || acknowledgement.service != self.expected_peer
            || acknowledgement.selected_version != IPC_PROTOCOL_VERSION
        {
            return Err(WireError::new(
                "IPC_HELLO_ACK_INVALID",
                "peer identity, protocol, or selected version is not the offered contract",
            ));
        }
        let selected = usize::try_from(acknowledgement.max_frame_bytes).unwrap_or(usize::MAX);
        validate_frame_limit(selected)?;
        if selected > offered_frame_bytes {
            return Err(WireError::new(
                "IPC_HELLO_ACK_INVALID",
                "peer selected a frame maximum above the offered bound",
            ));
        }
        validate_features(&acknowledgement.features)
    }

    fn ready_frame_limit(&self) -> Result<usize, WireError> {
        match self.state {
            ClientState::Ready {
                negotiated_frame_bytes,
            } => Ok(negotiated_frame_bytes),
            ClientState::New | ClientState::HelloPending { .. } => Err(WireError::new(
                "IPC_HELLO_REQUIRED",
                "protocol negotiation must complete before calls",
            )),
            ClientState::Closed => Err(closed()),
        }
    }

    fn remember_request(&mut self, request_id: &RpcRequestId) -> Result<(), WireError> {
        if !self.seen_request_ids.insert(request_id.clone()) {
            return Err(WireError::new(
                "IPC_REQUEST_ID_REUSED",
                "request IDs are unique for the full connection lifetime",
            ));
        }
        Ok(())
    }

    fn apply<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, WireError>,
    ) -> Result<T, WireError> {
        if self.state == ClientState::Closed {
            return Err(closed());
        }
        let result = operation(self);
        if result.is_err() {
            self.state = ClientState::Closed;
        }
        result
    }
}

fn count_message(messages: &mut u16) -> Result<(), WireError> {
    *messages = messages.checked_add(1).ok_or_else(|| {
        WireError::new("IPC_MESSAGE_LIMIT", "connection message counter overflowed")
    })?;
    if *messages > MAX_IPC_MESSAGES_PER_CONNECTION {
        return Err(WireError::new(
            "IPC_MESSAGE_LIMIT",
            "connection must be replaced after its bounded message budget",
        ));
    }
    Ok(())
}

fn closed() -> WireError {
    WireError::new(
        "IPC_SESSION_CLOSED",
        "the protocol session is already closed",
    )
}
