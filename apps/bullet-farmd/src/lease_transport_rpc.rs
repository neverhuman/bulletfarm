//! farmd-internal signed lease transport over a Unix JSON-RPC socket.
//!
//! This is not `/v1` and is not a public admission path. Farmd holds only
//! the verification key. The runner signs each request.

use bullet_adapters::SqliteLedger;
use bullet_application::{
    wire_grant, HeartbeatRequest, LeaseTransportRpcError, LeaseTransportRpcRequest,
    LeaseTransportRpcResponse, ReleaseRequest, SignedAcquireBody, SignedLeaseError,
    SignedLeaseService,
};
use bullet_domain::{RunnerId, WorkPackageId};
use serde::de::DeserializeOwned;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

/// Shared farmd-internal lease-transport state.
pub struct LeaseTransportState {
    ledger: Mutex<SqliteLedger>,
    service: Mutex<SignedLeaseService>,
}

impl LeaseTransportState {
    /// Open the ledger and bind the verification key only.
    ///
    /// # Errors
    ///
    /// Ledger open or key parse failure.
    pub fn open(
        db: &Path,
        issuer: &str,
        key_id: &str,
        public_hex: &str,
    ) -> Result<Arc<Self>, String> {
        let ledger = SqliteLedger::open(db).map_err(|err| err.to_string())?;
        let service = SignedLeaseService::from_hex(issuer, key_id, public_hex)
            .map_err(|err| format!("{}: {err}", err.reason_code()))?;
        Ok(Arc::new(Self {
            ledger: Mutex::new(ledger),
            service: Mutex::new(service),
        }))
    }
}

/// Listen on `socket` and serve signed lease operations until the task is
/// cancelled.
///
/// # Errors
///
/// Bind or accept failure.
pub async fn serve(socket: PathBuf, state: Arc<LeaseTransportState>) -> Result<(), String> {
    if socket.exists() {
        std::fs::remove_file(&socket).map_err(|err| err.to_string())?;
    }
    if let Some(parent) = socket.parent() {
        std::fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let listener = UnixListener::bind(&socket).map_err(|err| err.to_string())?;
    tracing::info!("lease-transport listening on {}", socket.display());
    loop {
        let (stream, _) = listener.accept().await.map_err(|err| err.to_string())?;
        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(err) = handle_connection(stream, state).await {
                tracing::warn!("lease-transport connection: {err}");
            }
        });
    }
}

async fn handle_connection(
    stream: UnixStream,
    state: Arc<LeaseTransportState>,
) -> Result<(), String> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .map_err(|err| err.to_string())?;
    if line.is_empty() {
        return Ok(());
    }
    let request: LeaseTransportRpcRequest =
        serde_json::from_str(line.trim()).map_err(|err| err.to_string())?;
    let response = dispatch(&state, request).await;
    let mut encoded = serde_json::to_string(&response).map_err(|err| err.to_string())?;
    encoded.push('\n');
    reader
        .get_mut()
        .write_all(encoded.as_bytes())
        .await
        .map_err(|err| err.to_string())?;
    Ok(())
}

async fn dispatch(
    state: &LeaseTransportState,
    request: LeaseTransportRpcRequest,
) -> LeaseTransportRpcResponse {
    let id = request.id.clone();
    match dispatch_inner(state, request).await {
        Ok(ok) => LeaseTransportRpcResponse {
            id,
            ok: Some(ok),
            err: None,
        },
        Err(err) => LeaseTransportRpcResponse {
            id,
            ok: None,
            err: Some(err),
        },
    }
}

async fn dispatch_inner(
    state: &LeaseTransportState,
    request: LeaseTransportRpcRequest,
) -> Result<serde_json::Value, LeaseTransportRpcError> {
    let now = now_unix_ms();
    let mut ledger = state.ledger.lock().await;
    let mut service = state.service.lock().await;
    match request.method.as_str() {
        "acquire" => {
            let body: SignedAcquireBody = decode_body(&request.body)?;
            let grant = service
                .acquire(&mut *ledger, &request.permit, &body, now)
                .map_err(map_signed)?;
            let wire = wire_grant(&*ledger, &grant, &body.work_package_id).map_err(map_signed)?;
            serde_json::to_value(wire).map_err(map_json)
        }
        "readback" => {
            let body: SignedAcquireBody = decode_body(&request.body)?;
            let grant = service
                .readback(&*ledger, &request.permit, &body, now)
                .map_err(map_signed)?;
            let wire = wire_grant(&*ledger, &grant, &body.work_package_id).map_err(map_signed)?;
            serde_json::to_value(wire).map_err(map_json)
        }
        "heartbeat" => {
            let (work_package_id, idempotency_key) = subject(&request)?;
            let call: HeartbeatRequest = decode_body(&request.body)?;
            service
                .heartbeat(
                    &mut *ledger,
                    &request.permit,
                    &work_package_id,
                    &idempotency_key,
                    &call,
                    now,
                )
                .map_err(map_signed)?;
            Ok(serde_json::json!({}))
        }
        "release" => {
            let (work_package_id, idempotency_key) = subject(&request)?;
            let (runner_id, runner_epoch) = release_runner(&request)?;
            let call: ReleaseRequest = decode_body(&request.body)?;
            service
                .release(
                    &mut *ledger,
                    &request.permit,
                    &runner_id,
                    runner_epoch,
                    &work_package_id,
                    &idempotency_key,
                    &call,
                    now,
                )
                .map_err(map_signed)?;
            Ok(serde_json::json!({}))
        }
        _ => Err(LeaseTransportRpcError {
            code: "LEASE_TRANSPORT_UNSUPPORTED".into(),
            message: format!("method {} is not a signed lease operation", request.method),
        }),
    }
}

fn subject(
    request: &LeaseTransportRpcRequest,
) -> Result<(WorkPackageId, String), LeaseTransportRpcError> {
    let raw = request
        .work_package_id
        .as_deref()
        .ok_or_else(|| LeaseTransportRpcError {
            code: "LEASE_TRANSPORT_INVALID".into(),
            message: "work_package_id is required".into(),
        })?;
    let work_package_id = WorkPackageId::parse(raw).map_err(|err| LeaseTransportRpcError {
        code: "LEASE_TRANSPORT_INVALID".into(),
        message: err.to_string(),
    })?;
    let idempotency_key =
        request
            .idempotency_key
            .clone()
            .ok_or_else(|| LeaseTransportRpcError {
                code: "LEASE_TRANSPORT_INVALID".into(),
                message: "idempotency_key is required".into(),
            })?;
    Ok((work_package_id, idempotency_key))
}

fn release_runner(
    request: &LeaseTransportRpcRequest,
) -> Result<(RunnerId, u64), LeaseTransportRpcError> {
    let raw = request
        .runner_id
        .as_deref()
        .ok_or_else(|| LeaseTransportRpcError {
            code: "LEASE_TRANSPORT_INVALID".into(),
            message: "runner_id is required".into(),
        })?;
    let runner_id = RunnerId::parse(raw).map_err(|err| LeaseTransportRpcError {
        code: "LEASE_TRANSPORT_INVALID".into(),
        message: err.to_string(),
    })?;
    let runner_epoch = request.runner_epoch.ok_or_else(|| LeaseTransportRpcError {
        code: "LEASE_TRANSPORT_INVALID".into(),
        message: "runner_epoch is required".into(),
    })?;
    Ok((runner_id, runner_epoch))
}

fn decode_body<T: DeserializeOwned>(body: &serde_json::Value) -> Result<T, LeaseTransportRpcError> {
    serde_json::from_value(body.clone()).map_err(|err| LeaseTransportRpcError {
        code: "LEASE_TRANSPORT_INVALID".into(),
        message: err.to_string(),
    })
}

fn map_signed(err: SignedLeaseError) -> LeaseTransportRpcError {
    LeaseTransportRpcError {
        code: err.reason_code().to_string(),
        message: err.to_string(),
    }
}

fn map_json(err: serde_json::Error) -> LeaseTransportRpcError {
    LeaseTransportRpcError {
        code: "LEASE_TRANSPORT_INVALID".into(),
        message: err.to_string(),
    }
}

fn now_unix_ms() -> u64 {
    u64::try_from(chrono::Utc::now().timestamp_millis()).unwrap_or(0)
}
