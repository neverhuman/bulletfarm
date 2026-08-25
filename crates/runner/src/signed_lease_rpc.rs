//! Production Runner client for farmd-internal signed lease transport.
//!
//! The Runner holds the signing key. Farmd holds only the verification
//! key. Public `/v1/leases/*` stays unmounted. Ready-queue reads stay the
//! unsigned `GET /v1/ready` projection.

use crate::error::RunnerError;
use crate::http_lease::HttpLeaseClient;
use crate::lease::{
    AcquireGrant, AcquireRequest, HeartbeatCall, LeaseClient, ReadyView, ReleaseCall,
};
use async_trait::async_trait;
use bullet_application::{
    sign_runner_permit, HeartbeatRequest, LeaseTransportRpcRequest, LeaseTransportRpcResponse,
    ReleaseRequest, SignedAcquireBody, SignedLeaseWireGrant,
};
use bullet_domain::{AttemptId, AttemptState, RunnerId, VariantId, WorkPackageId};
use bullet_harness_core::lease_transport::{LeaseTransportOperation, LeaseTransportSigningKey};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

struct AcquireMeta {
    work_package_id: WorkPackageId,
    idempotency_key: String,
    runner_id: RunnerId,
    runner_epoch: u64,
    variant_id: VariantId,
}

/// Signed Unix JSON-RPC lease client. Ready reads stay on farmd HTTP.
pub struct SignedLeaseRpcClient {
    socket: PathBuf,
    key: LeaseTransportSigningKey,
    ready: HttpLeaseClient,
    last: Mutex<BTreeMap<String, AcquireMeta>>,
}

impl SignedLeaseRpcClient {
    /// Bind the runner signing key, the farmd-internal socket, and the
    /// unsigned ready projection.
    ///
    /// # Errors
    ///
    /// `PROTOCOL_ERROR` for a bad farmd base URL.
    pub fn new(
        socket: PathBuf,
        key: LeaseTransportSigningKey,
        farmd: &str,
    ) -> Result<Self, RunnerError> {
        Ok(Self {
            socket,
            key,
            ready: HttpLeaseClient::new(farmd)?,
            last: Mutex::new(BTreeMap::new()),
        })
    }

    /// Load a 64-byte secret key file.
    ///
    /// # Errors
    ///
    /// IO or `LEASE_TRANSPORT_INVALID`.
    pub fn load_key(
        path: &Path,
        issuer: &str,
        key_id: &str,
    ) -> Result<LeaseTransportSigningKey, RunnerError> {
        let bytes = std::fs::read(path).map_err(|err| RunnerError::Io {
            context: "lease-transport signing key".into(),
            reason: err.to_string(),
        })?;
        LeaseTransportSigningKey::from_bytes(issuer, key_id, &bytes).map_err(map_transport)
    }

    fn lock_last(&self) -> Result<MutexGuard<'_, BTreeMap<String, AcquireMeta>>, RunnerError> {
        self.last.lock().map_err(|_| RunnerError::Io {
            context: "lease-transport meta lock".into(),
            reason: "poisoned".into(),
        })
    }

    fn meta_for(&self, attempt_id: &AttemptId) -> Result<AcquireMeta, RunnerError> {
        self.lock_last()?
            .get(attempt_id.as_str())
            .map(|meta| AcquireMeta {
                work_package_id: meta.work_package_id.clone(),
                idempotency_key: meta.idempotency_key.clone(),
                runner_id: meta.runner_id.clone(),
                runner_epoch: meta.runner_epoch,
                variant_id: meta.variant_id.clone(),
            })
            .ok_or_else(|| RunnerError::Lease {
                code: "LEASE_TRANSPORT_UNKNOWN".into(),
                message: format!("no signed acquire recorded for {attempt_id}"),
            })
    }

    fn remember(&self, grant: &AcquireGrant, request: &AcquireRequest) -> Result<(), RunnerError> {
        self.lock_last()?.insert(
            grant.attempt.id.to_string(),
            AcquireMeta {
                work_package_id: request.work_package_id.clone(),
                idempotency_key: request.idempotency_key.clone(),
                runner_id: request.runner_id.clone(),
                runner_epoch: request.runner_epoch,
                variant_id: grant.lease.variant_id.clone(),
            },
        );
        Ok(())
    }

    async fn rpc(
        &self,
        request: &LeaseTransportRpcRequest,
    ) -> Result<serde_json::Value, RunnerError> {
        let mut stream =
            UnixStream::connect(&self.socket)
                .await
                .map_err(|err| RunnerError::Io {
                    context: "lease-transport connect".into(),
                    reason: err.to_string(),
                })?;
        let mut encoded = serde_json::to_string(request).map_err(|err| {
            RunnerError::Protocol(format!("encode lease-transport request: {err}"))
        })?;
        encoded.push('\n');
        stream
            .write_all(encoded.as_bytes())
            .await
            .map_err(|err| RunnerError::Io {
                context: "lease-transport write".into(),
                reason: err.to_string(),
            })?;
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .map_err(|err| RunnerError::Io {
                context: "lease-transport read".into(),
                reason: err.to_string(),
            })?;
        let response: LeaseTransportRpcResponse =
            serde_json::from_str(line.trim()).map_err(|err| {
                RunnerError::Protocol(format!("decode lease-transport response: {err}"))
            })?;
        if response.id != request.id {
            return Err(RunnerError::Protocol(
                "lease-transport response id mismatch".into(),
            ));
        }
        if let Some(err) = response.err {
            if err.code == "STALE_AUTHORITY" {
                return Err(RunnerError::StaleAuthority(err.message));
            }
            return Err(RunnerError::Lease {
                code: err.code,
                message: err.message,
            });
        }
        response.ok.ok_or_else(|| {
            RunnerError::Protocol("lease-transport response missing ok and err".into())
        })
    }

    async fn acquire_or_readback(
        &self,
        body: &SignedAcquireBody,
    ) -> Result<SignedLeaseWireGrant, RunnerError> {
        let now = now_unix_ms();
        let permit = sign_runner_permit(
            &self.key,
            LeaseTransportOperation::Acquire,
            &body.runner_id,
            body.runner_epoch,
            body.work_package_id.as_str(),
            &body.idempotency_key,
            body,
            now,
        )
        .map_err(map_signed)?;
        match self
            .rpc(&rpc_request(
                "acquire",
                permit,
                serde_json::to_value(body)
                    .map_err(|err| RunnerError::Protocol(format!("encode acquire body: {err}")))?,
                None,
            ))
            .await
        {
            Ok(value) => decode_wire(value),
            Err(err) if is_lost_response(&err) => self.readback(body).await.map_err(|readback| {
                if matches!(&readback, RunnerError::Lease { code, .. } if code == "LEASE_TRANSPORT_UNKNOWN")
                {
                    err
                } else {
                    readback
                }
            }),
            Err(err) => Err(err),
        }
    }

    async fn readback(
        &self,
        body: &SignedAcquireBody,
    ) -> Result<SignedLeaseWireGrant, RunnerError> {
        let now = now_unix_ms();
        let permit = sign_runner_permit(
            &self.key,
            LeaseTransportOperation::Readback,
            &body.runner_id,
            body.runner_epoch,
            body.work_package_id.as_str(),
            &body.idempotency_key,
            body,
            now,
        )
        .map_err(map_signed)?;
        let value = self
            .rpc(&rpc_request(
                "readback",
                permit,
                serde_json::to_value(body)
                    .map_err(|err| RunnerError::Protocol(format!("encode readback body: {err}")))?,
                None,
            ))
            .await?;
        decode_wire(value)
    }
}

#[async_trait]
impl LeaseClient for SignedLeaseRpcClient {
    async fn acquire(&self, request: &AcquireRequest) -> Result<AcquireGrant, RunnerError> {
        let body = SignedAcquireBody {
            work_package_id: request.work_package_id.clone(),
            runner_id: request.runner_id.clone(),
            runner_epoch: request.runner_epoch,
            idempotency_key: request.idempotency_key.clone(),
            ttl_seconds: request.ttl_seconds,
        };
        let wire = self.acquire_or_readback(&body).await?;
        let grant = AcquireGrant {
            attempt: wire.grant.attempt,
            authority_token: wire.authority_token,
            lease: wire.grant.lease,
        };
        self.remember(&grant, request)?;
        Ok(grant)
    }

    async fn heartbeat(&self, call: &HeartbeatCall) -> Result<(), RunnerError> {
        let meta = self.meta_for(&call.attempt_id)?;
        let request = HeartbeatRequest {
            variant_id: call.variant_id.clone(),
            attempt_id: call.attempt_id.clone(),
            fence: call.fence,
            runner_id: call.runner_id.clone(),
            runner_epoch: call.runner_epoch,
            workspace_nonce: call.workspace_nonce,
            ttl_seconds: call.ttl_seconds,
        };
        let now = now_unix_ms();
        let permit = sign_runner_permit(
            &self.key,
            LeaseTransportOperation::Heartbeat,
            &request.runner_id,
            request.runner_epoch,
            meta.work_package_id.as_str(),
            &meta.idempotency_key,
            &request,
            now,
        )
        .map_err(map_signed)?;
        let _ = self
            .rpc(&rpc_request(
                "heartbeat",
                permit,
                serde_json::to_value(&request).map_err(|err| {
                    RunnerError::Protocol(format!("encode heartbeat body: {err}"))
                })?,
                Some(&meta),
            ))
            .await?;
        Ok(())
    }

    async fn advance(
        &self,
        _attempt_id: &AttemptId,
        _state: AttemptState,
    ) -> Result<(), RunnerError> {
        Err(RunnerError::Lease {
            code: "LEASE_TRANSPORT_UNSUPPORTED".into(),
            message: "signed advance is not implemented; do not fall back to HttpLeaseClient"
                .into(),
        })
    }

    async fn release(&self, call: &ReleaseCall) -> Result<(), RunnerError> {
        let meta = self.meta_for(&call.attempt_id)?;
        let request = ReleaseRequest {
            variant_id: meta.variant_id.clone(),
            attempt_id: call.attempt_id.clone(),
            final_state: call.outcome,
            requeue: call.requeue,
        };
        let now = now_unix_ms();
        let permit = sign_runner_permit(
            &self.key,
            LeaseTransportOperation::Release,
            &meta.runner_id,
            meta.runner_epoch,
            meta.work_package_id.as_str(),
            &meta.idempotency_key,
            &request,
            now,
        )
        .map_err(map_signed)?;
        let _ = self
            .rpc(&rpc_request(
                "release",
                permit,
                serde_json::to_value(&request)
                    .map_err(|err| RunnerError::Protocol(format!("encode release body: {err}")))?,
                Some(&meta),
            ))
            .await?;
        Ok(())
    }

    async fn next_ready(&self) -> Result<Option<ReadyView>, RunnerError> {
        self.ready.next_ready().await
    }
}

fn rpc_request(
    method: &str,
    permit: bullet_harness_core::lease_transport::SignedLeasePermit,
    body: serde_json::Value,
    meta: Option<&AcquireMeta>,
) -> LeaseTransportRpcRequest {
    LeaseTransportRpcRequest {
        id: request_id(),
        method: method.to_string(),
        permit,
        body,
        work_package_id: meta.map(|item| item.work_package_id.to_string()),
        idempotency_key: meta.map(|item| item.idempotency_key.clone()),
        runner_id: meta.map(|item| item.runner_id.to_string()),
        runner_epoch: meta.map(|item| item.runner_epoch),
    }
}

fn decode_wire(value: serde_json::Value) -> Result<SignedLeaseWireGrant, RunnerError> {
    serde_json::from_value(value)
        .map_err(|err| RunnerError::Protocol(format!("decode lease-transport grant: {err}")))
}

fn is_lost_response(err: &RunnerError) -> bool {
    matches!(err, RunnerError::Io { .. } | RunnerError::Protocol(_))
}

fn request_id() -> String {
    bullet_harness_core::synthetic_uuid("lease-rpc")
}

fn now_unix_ms() -> u64 {
    u64::try_from(chrono::Utc::now().timestamp_millis()).unwrap_or(0)
}

fn map_signed(err: bullet_application::SignedLeaseError) -> RunnerError {
    RunnerError::Lease {
        code: err.reason_code().to_string(),
        message: err.to_string(),
    }
}

fn map_transport(err: bullet_harness_core::lease_transport::LeaseTransportError) -> RunnerError {
    RunnerError::Lease {
        code: err.reason_code().to_string(),
        message: err.to_string(),
    }
}
