use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use super::message::{
    IPC_PROTOCOL, IPC_PROTOCOL_VERSION, IpcHello, IpcHelloAck, IpcService, MAX_IPC_FRAME_BYTES,
    MAX_IPC_IN_FLIGHT, MAX_IPC_MESSAGES_PER_CONNECTION, REQUIRED_FEATURES, RawNotification,
    RawRequest, RpcCallParams, RpcCancelParams, decode_message, strict_value, validate_deadline,
    validate_hello, validate_jsonrpc, validate_method,
};
use crate::{RpcRequestId, WireError};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RpcInbound {
    Hello {
        request_id: RpcRequestId,
        peer: IpcService,
        acknowledgement: IpcHelloAck,
    },
    Call {
        request_id: RpcRequestId,
        method: String,
        deadline_unix_ms: u64,
        body: Value,
    },
    Cancel {
        request_id: RpcRequestId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ActiveRequest {
    cancelled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SessionState {
    AwaitingHello,
    Ready { negotiated_frame_bytes: usize },
    Closed,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RpcServerSession {
    local_service: IpcService,
    state: SessionState,
    messages: u16,
    seen_request_ids: BTreeSet<RpcRequestId>,
    active_requests: BTreeMap<RpcRequestId, ActiveRequest>,
}

impl RpcServerSession {
    pub fn new(local_service: IpcService) -> Self {
        Self {
            local_service,
            state: SessionState::AwaitingHello,
            messages: 0,
            seen_request_ids: BTreeSet::new(),
            active_requests: BTreeMap::new(),
        }
    }

    pub fn accept_frame(
        &mut self,
        frame: &[u8],
        now_unix_ms: u64,
    ) -> Result<RpcInbound, WireError> {
        if self.state == SessionState::Closed {
            return Err(closed());
        }
        let result = self.accept_open_frame(frame, now_unix_ms);
        if result.is_err() {
            self.state = SessionState::Closed;
        }
        result
    }

    pub fn finish_request(&mut self, request_id: &RpcRequestId) -> Result<(), WireError> {
        if self.state == SessionState::Closed {
            return Err(closed());
        }
        self.active_requests.remove(request_id).map_or_else(
            || {
                Err(WireError::new(
                    "IPC_REQUEST_UNKNOWN",
                    "cannot finish an inactive request",
                ))
            },
            |_| Ok(()),
        )
    }

    pub fn request_is_cancelled(&self, request_id: &RpcRequestId) -> Result<bool, WireError> {
        self.active_requests.get(request_id).map_or_else(
            || {
                Err(WireError::new(
                    "IPC_REQUEST_UNKNOWN",
                    "request is not active",
                ))
            },
            |request| Ok(request.cancelled),
        )
    }

    pub fn negotiated_frame_bytes(&self) -> Option<usize> {
        match self.state {
            SessionState::Ready {
                negotiated_frame_bytes,
            } => Some(negotiated_frame_bytes),
            SessionState::AwaitingHello | SessionState::Closed => None,
        }
    }

    fn accept_open_frame(
        &mut self,
        frame: &[u8],
        now_unix_ms: u64,
    ) -> Result<RpcInbound, WireError> {
        count_message(&mut self.messages)?;
        let frame_limit = self.negotiated_frame_bytes().unwrap_or(MAX_IPC_FRAME_BYTES);
        let value = decode_message(frame, frame_limit)?;
        let object = value.as_object().ok_or_else(|| {
            WireError::new("IPC_MESSAGE_INVALID", "JSON-RPC frame must be an object")
        })?;
        object
            .get("method")
            .and_then(Value::as_str)
            .ok_or_else(|| WireError::new("IPC_MESSAGE_INVALID", "JSON-RPC method is required"))?;
        if object.contains_key("id") {
            let request: RawRequest = strict_value(value)?;
            validate_jsonrpc(&request.jsonrpc)?;
            self.accept_request(request, now_unix_ms)
        } else {
            let notification: RawNotification = strict_value(value)?;
            validate_jsonrpc(&notification.jsonrpc)?;
            self.accept_notification(notification)
        }
    }

    fn accept_request(
        &mut self,
        request: RawRequest,
        now_unix_ms: u64,
    ) -> Result<RpcInbound, WireError> {
        match self.state {
            SessionState::AwaitingHello => self.accept_hello(request),
            SessionState::Ready { .. } => self.accept_call(request, now_unix_ms),
            SessionState::Closed => Err(closed()),
        }
    }

    fn accept_hello(&mut self, request: RawRequest) -> Result<RpcInbound, WireError> {
        if request.method != "bullet.hello" {
            return Err(WireError::new(
                "IPC_HELLO_REQUIRED",
                "the first frame must be a bullet.hello request",
            ));
        }
        let hello: IpcHello = strict_value(request.params)?;
        validate_hello(&hello)?;
        self.remember_request(&request.id)?;
        let negotiated_frame_bytes = usize::try_from(hello.max_frame_bytes)
            .unwrap_or(MAX_IPC_FRAME_BYTES)
            .min(MAX_IPC_FRAME_BYTES);
        let acknowledgement = IpcHelloAck {
            protocol: IPC_PROTOCOL.to_owned(),
            service: self.local_service,
            selected_version: IPC_PROTOCOL_VERSION,
            max_frame_bytes: u32::try_from(negotiated_frame_bytes)
                .expect("the fixed IPC frame bound fits u32"),
            features: REQUIRED_FEATURES.iter().map(ToString::to_string).collect(),
        };
        self.state = SessionState::Ready {
            negotiated_frame_bytes,
        };
        Ok(RpcInbound::Hello {
            request_id: request.id,
            peer: hello.service,
            acknowledgement,
        })
    }

    fn accept_call(
        &mut self,
        request: RawRequest,
        now_unix_ms: u64,
    ) -> Result<RpcInbound, WireError> {
        validate_method(&request.method)?;
        if matches!(request.method.as_str(), "bullet.hello" | "bullet.cancel") {
            return Err(WireError::new(
                "IPC_METHOD_INVALID",
                "hello cannot repeat and cancellation must be a notification",
            ));
        }
        if self.active_requests.len() >= MAX_IPC_IN_FLIGHT {
            return Err(WireError::new(
                "IPC_IN_FLIGHT_LIMIT",
                "connection has too many active requests",
            ));
        }
        let params: RpcCallParams = strict_value(request.params)?;
        validate_deadline(params.deadline_unix_ms, now_unix_ms)?;
        self.remember_request(&request.id)?;
        self.active_requests
            .insert(request.id.clone(), ActiveRequest { cancelled: false });
        Ok(RpcInbound::Call {
            request_id: request.id,
            method: request.method,
            deadline_unix_ms: params.deadline_unix_ms,
            body: params.body,
        })
    }

    fn accept_notification(
        &mut self,
        notification: RawNotification,
    ) -> Result<RpcInbound, WireError> {
        if self.state == SessionState::AwaitingHello {
            return Err(WireError::new(
                "IPC_HELLO_REQUIRED",
                "the first frame must be a bullet.hello request with an ID",
            ));
        }
        if notification.method != "bullet.cancel" {
            return Err(WireError::new(
                "IPC_NOTIFICATION_INVALID",
                "bullet.cancel is the only admitted notification",
            ));
        }
        let cancel: RpcCancelParams = strict_value(notification.params)?;
        let active = self
            .active_requests
            .get_mut(&cancel.request_id)
            .ok_or_else(|| {
                WireError::new(
                    "IPC_CANCEL_UNKNOWN",
                    "cancellation must reference an active request",
                )
            })?;
        if active.cancelled {
            return Err(WireError::new(
                "IPC_CANCEL_DUPLICATE",
                "an active request may be cancelled only once",
            ));
        }
        active.cancelled = true;
        Ok(RpcInbound::Cancel {
            request_id: cancel.request_id,
        })
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
