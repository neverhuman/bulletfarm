//! Kernel-side signed lease-transport service.
//!
//! Public farmd `/v1/leases/*` routes stay absent. `HttpLeaseClient` is not
//! an admission path. This service verifies a runner-issued `lease-runner`
//! permit, then applies one ledger operation or returns the durable grant
//! for a lost acquire response. Farmd holds only the verification key.
//! The in-process issuer below exists only under `test-seams`.

use crate::launch_grant::KERNEL_AUTHORITY_EPOCH;
use crate::leases::LeaseService;
use crate::records::{HeartbeatRequest, LeaseGrant, LeaseRequest, ReleaseRequest, StoredGraph};
use crate::store::{Ledger, LedgerError};
use bullet_domain::{AuthorityToken, Digest, RunnerId, VariantId, WorkPackageId, WorkspaceId};
use bullet_harness_core::launch_grant::MemoryNonceLedger;
use bullet_harness_core::lease_transport::LeaseTransportSigningKey;
use bullet_harness_core::lease_transport::{
    request_digest, verify_lease_permit, LeaseTransportError, LeaseTransportExpectation,
    LeaseTransportOperation, LeaseTransportVerificationKey, SignedLeasePermit,
};
use serde::{Deserialize, Serialize};

/// Request body covered by an `acquire` or `readback` permit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedAcquireBody {
    /// Package to lease.
    pub work_package_id: WorkPackageId,
    /// Runner identity.
    pub runner_id: RunnerId,
    /// Runner generation.
    pub runner_epoch: u64,
    /// Idempotency key; also seeds the attempt.
    pub idempotency_key: String,
    /// Requested TTL in seconds (`1..=15`).
    pub ttl_seconds: i64,
}

/// Acquire reply returned over the internal Unix transport.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedLeaseWireGrant {
    /// Durable grant (attempt + lease).
    pub grant: LeaseGrant,
    /// Rebuilt authority token for the incarnation.
    pub authority_token: AuthorityToken,
}

/// One Kernel-owned signed lease-transport endpoint.
///
/// The grant index is durable on `Ledger`. Nonces stay process-local:
/// a restart forgets spent nonces, but a new permit plus idempotent
/// `acquire_lease` returns the same grant.
pub struct SignedLeaseService {
    verification: LeaseTransportVerificationKey,
    nonces: MemoryNonceLedger,
}

impl SignedLeaseService {
    /// Bind the service to one verification key. The matching signing key
    /// stays with the runner, never this service.
    #[must_use]
    pub fn new(verification: LeaseTransportVerificationKey) -> Self {
        Self {
            verification,
            nonces: MemoryNonceLedger::new(),
        }
    }

    /// Bind from the published 64-hex verification half.
    ///
    /// # Errors
    ///
    /// `LEASE_TRANSPORT_INVALID` for a bad label or key encoding.
    pub fn from_hex(
        issuer: &str,
        key_id: &str,
        public_hex: &str,
    ) -> Result<Self, SignedLeaseError> {
        Ok(Self::new(
            LeaseTransportVerificationKey::from_hex(issuer, key_id, public_hex)
                .map_err(SignedLeaseError::Transport)?,
        ))
    }

    /// Register a freshly minted nonce before verification.
    pub fn register_nonce(&mut self, nonce: &str, binding: &str, expires_at_unix_ms: u64) -> bool {
        self.nonces.register(nonce, binding, expires_at_unix_ms)
    }

    fn admit_permit<T: Serialize>(
        &mut self,
        args: AdmitArgs<'_, T>,
    ) -> Result<(), SignedLeaseError> {
        let claims = self
            .verification
            .authenticate(args.permit)
            .map_err(SignedLeaseError::Transport)?;
        let binding = format!(
            "{}:{}:{}",
            claims.operation.as_str(),
            claims.runner_id,
            claims.idempotency_digest
        );
        let _ = self
            .nonces
            .register(&claims.permit_nonce, &binding, claims.expires_at_unix_ms);
        self.verify(args)
    }

    /// Acquire or replay one writer lease after verifying the permit.
    ///
    /// # Errors
    ///
    /// Typed transport refusal or ledger failure.
    pub fn acquire<L: Ledger>(
        &mut self,
        ledger: &mut L,
        permit: &SignedLeasePermit,
        body: &SignedAcquireBody,
        now_unix_ms: u64,
    ) -> Result<LeaseGrant, SignedLeaseError> {
        self.admit_permit(AdmitArgs {
            permit,
            operation: LeaseTransportOperation::Acquire,
            body,
            runner_id: &body.runner_id,
            runner_epoch: body.runner_epoch,
            work_package_id: body.work_package_id.as_str(),
            idempotency_key: &body.idempotency_key,
            now_unix_ms,
        })?;
        let (graph, variant_id) = graph_for_package(ledger, &body.work_package_id)?;
        let request = lease_request(body, &graph, &variant_id);
        let grant = ledger
            .acquire_lease(&request)
            .map_err(SignedLeaseError::Ledger)?;
        ledger
            .put_lease_transport_grant(&idempotency_digest(&body.idempotency_key)?, &grant)
            .map_err(SignedLeaseError::Ledger)?;
        Ok(grant)
    }

    /// Return the last grant for this idempotency key without minting a sibling.
    ///
    /// # Errors
    ///
    /// Typed transport refusal, or `UNKNOWN` when no grant was stored.
    pub fn readback<L: Ledger>(
        &mut self,
        ledger: &L,
        permit: &SignedLeasePermit,
        body: &SignedAcquireBody,
        now_unix_ms: u64,
    ) -> Result<LeaseGrant, SignedLeaseError> {
        self.admit_permit(AdmitArgs {
            permit,
            operation: LeaseTransportOperation::Readback,
            body,
            runner_id: &body.runner_id,
            runner_epoch: body.runner_epoch,
            work_package_id: body.work_package_id.as_str(),
            idempotency_key: &body.idempotency_key,
            now_unix_ms,
        })?;
        ledger
            .get_lease_transport_grant(&idempotency_digest(&body.idempotency_key)?)
            .map_err(SignedLeaseError::Ledger)?
            .ok_or(SignedLeaseError::Unknown)
    }

    /// Renew one lease. The permit covers `call`, not `SignedAcquireBody`.
    ///
    /// # Errors
    ///
    /// Typed transport or ledger refusal.
    pub fn heartbeat<L: Ledger>(
        &mut self,
        ledger: &mut L,
        permit: &SignedLeasePermit,
        work_package_id: &WorkPackageId,
        idempotency_key: &str,
        call: &HeartbeatRequest,
        now_unix_ms: u64,
    ) -> Result<(), SignedLeaseError> {
        self.admit_permit(AdmitArgs {
            permit,
            operation: LeaseTransportOperation::Heartbeat,
            body: call,
            runner_id: &call.runner_id,
            runner_epoch: call.runner_epoch,
            work_package_id: work_package_id.as_str(),
            idempotency_key,
            now_unix_ms,
        })?;
        ledger.heartbeat(call).map_err(SignedLeaseError::Ledger)
    }

    /// Close one lease. The permit covers `call`, not `SignedAcquireBody`.
    ///
    /// # Errors
    ///
    /// Typed transport or ledger refusal.
    #[allow(clippy::too_many_arguments)]
    pub fn release<L: Ledger>(
        &mut self,
        ledger: &mut L,
        permit: &SignedLeasePermit,
        runner_id: &RunnerId,
        runner_epoch: u64,
        work_package_id: &WorkPackageId,
        idempotency_key: &str,
        call: &ReleaseRequest,
        now_unix_ms: u64,
    ) -> Result<(), SignedLeaseError> {
        self.admit_permit(AdmitArgs {
            permit,
            operation: LeaseTransportOperation::Release,
            body: call,
            runner_id,
            runner_epoch,
            work_package_id: work_package_id.as_str(),
            idempotency_key,
            now_unix_ms,
        })?;
        ledger.release_lease(call).map_err(SignedLeaseError::Ledger)
    }

    fn verify<T: Serialize>(&mut self, args: AdmitArgs<'_, T>) -> Result<(), SignedLeaseError> {
        let digest = request_digest(args.body).map_err(SignedLeaseError::Transport)?;
        let expectation = LeaseTransportExpectation {
            operation: args.operation,
            request_digest: digest,
            runner_id: args.runner_id.as_str().to_string(),
            runner_epoch: args.runner_epoch,
            authority_epoch: KERNEL_AUTHORITY_EPOCH,
            work_package_id: args.work_package_id.to_string(),
            idempotency_digest: idempotency_digest(args.idempotency_key)?,
            now_unix_ms: args.now_unix_ms,
        };
        verify_lease_permit(
            args.permit,
            &self.verification,
            &expectation,
            &mut self.nonces,
        )
        .map(|_| ())
        .map_err(SignedLeaseError::Transport)
    }
}

struct AdmitArgs<'a, T> {
    permit: &'a SignedLeasePermit,
    operation: LeaseTransportOperation,
    body: &'a T,
    runner_id: &'a RunnerId,
    runner_epoch: u64,
    work_package_id: &'a str,
    idempotency_key: &'a str,
    now_unix_ms: u64,
}

/// Runner-held issuer. Farmd never sees this key.
///
/// A valid signature is issuance: the verifier registers the nonce from
/// authenticated claims, then consumes it. Do not call this from farmd.
///
/// # Errors
///
/// Signing, entropy, or digest failure.
#[allow(clippy::too_many_arguments)]
pub fn sign_runner_permit<T: Serialize>(
    key: &LeaseTransportSigningKey,
    operation: LeaseTransportOperation,
    runner_id: &RunnerId,
    runner_epoch: u64,
    work_package_id: &str,
    idempotency_key: &str,
    body: &T,
    now_unix_ms: u64,
) -> Result<SignedLeasePermit, SignedLeaseError> {
    let digest = request_digest(body).map_err(SignedLeaseError::Transport)?;
    let idem = idempotency_digest(idempotency_key)?;
    let nonce =
        bullet_harness_core::lease_transport::new_hex_64().map_err(SignedLeaseError::Transport)?;
    let permit_id =
        bullet_harness_core::lease_transport::new_hex_64().map_err(SignedLeaseError::Transport)?;
    let claims = bullet_harness_core::lease_transport::LeaseTransportClaims {
        schema_version: bullet_harness_core::lease_transport::LEASE_TRANSPORT_SCHEMA_VERSION
            .to_string(),
        permit_id,
        audience: bullet_harness_core::lease_transport::LEASE_TRANSPORT_AUDIENCE.to_string(),
        operation,
        issuer: key.issuer().to_string(),
        key_id: key.key_id().to_string(),
        issued_at_unix_ms: now_unix_ms,
        not_before_unix_ms: now_unix_ms,
        expires_at_unix_ms: now_unix_ms + 15_000,
        permit_nonce: nonce,
        request_digest: digest,
        runner_id: runner_id.as_str().to_string(),
        runner_epoch,
        authority_epoch: KERNEL_AUTHORITY_EPOCH,
        work_package_id: work_package_id.to_string(),
        idempotency_digest: idem,
    };
    key.sign(&claims).map_err(SignedLeaseError::Transport)
}

/// Simulator-only in-process issuer: register the nonce, then sign.
///
/// This deliberately co-locates issuance with the verifier so tests can
/// exercise the wire contract. It is absent from default production builds.
///
/// # Errors
///
/// Signing, entropy, or nonce-registration failure.
#[cfg(any(test, feature = "test-seams"))]
pub fn issue_permit(
    key: &LeaseTransportSigningKey,
    service: &mut SignedLeaseService,
    operation: LeaseTransportOperation,
    body: &SignedAcquireBody,
    now_unix_ms: u64,
) -> Result<SignedLeasePermit, SignedLeaseError> {
    issue_operation_permit(
        key,
        service,
        operation,
        &body.runner_id,
        body.runner_epoch,
        body.work_package_id.as_str(),
        &body.idempotency_key,
        body,
        now_unix_ms,
    )
}

/// Simulator-only issuer for any request body the permit must digest.
///
/// It is absent from default production builds; a real transport must obtain
/// permits from a separately authenticated Kernel-owned issuer.
///
/// # Errors
///
/// Signing, entropy, or nonce-registration failure.
#[allow(clippy::too_many_arguments)]
#[cfg(any(test, feature = "test-seams"))]
pub fn issue_operation_permit<T: Serialize>(
    key: &LeaseTransportSigningKey,
    service: &mut SignedLeaseService,
    operation: LeaseTransportOperation,
    runner_id: &RunnerId,
    runner_epoch: u64,
    work_package_id: &str,
    idempotency_key: &str,
    body: &T,
    now_unix_ms: u64,
) -> Result<SignedLeasePermit, SignedLeaseError> {
    let digest = request_digest(body).map_err(SignedLeaseError::Transport)?;
    let idem = idempotency_digest(idempotency_key)?;
    let nonce =
        bullet_harness_core::lease_transport::new_hex_64().map_err(SignedLeaseError::Transport)?;
    let permit_id =
        bullet_harness_core::lease_transport::new_hex_64().map_err(SignedLeaseError::Transport)?;
    let claims = bullet_harness_core::lease_transport::LeaseTransportClaims {
        schema_version: bullet_harness_core::lease_transport::LEASE_TRANSPORT_SCHEMA_VERSION
            .to_string(),
        permit_id,
        audience: bullet_harness_core::lease_transport::LEASE_TRANSPORT_AUDIENCE.to_string(),
        operation,
        issuer: key.issuer().to_string(),
        key_id: key.key_id().to_string(),
        issued_at_unix_ms: now_unix_ms,
        not_before_unix_ms: now_unix_ms,
        expires_at_unix_ms: now_unix_ms + 15_000,
        permit_nonce: nonce.clone(),
        request_digest: digest,
        runner_id: runner_id.as_str().to_string(),
        runner_epoch,
        authority_epoch: KERNEL_AUTHORITY_EPOCH,
        work_package_id: work_package_id.to_string(),
        idempotency_digest: idem.clone(),
    };
    let binding = format!("{}:{}:{}", operation.as_str(), claims.runner_id, idem);
    if !service.register_nonce(&nonce, &binding, claims.expires_at_unix_ms) {
        return Err(SignedLeaseError::Transport(LeaseTransportError::Invalid {
            reason: "permit nonce already registered".into(),
        }));
    }
    key.sign(&claims).map_err(SignedLeaseError::Transport)
}

/// Service-level refusal.
#[derive(Debug, thiserror::Error)]
pub enum SignedLeaseError {
    /// Permit verification failed.
    #[error(transparent)]
    Transport(LeaseTransportError),
    /// Ledger refused the operation.
    #[error(transparent)]
    Ledger(LedgerError),
    /// Readback found no stored grant.
    #[error("lease transport unknown")]
    Unknown,
}

impl SignedLeaseError {
    /// Stable reason code.
    #[must_use]
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::Transport(error) => error.reason_code(),
            Self::Ledger(error) => error.reason_code(),
            Self::Unknown => "LEASE_TRANSPORT_UNKNOWN",
        }
    }
}

fn idempotency_digest(key: &str) -> Result<String, SignedLeaseError> {
    request_digest(&key).map_err(SignedLeaseError::Transport)
}

fn lease_request(
    body: &SignedAcquireBody,
    graph: &StoredGraph,
    variant_id: &VariantId,
) -> LeaseRequest {
    LeaseRequest {
        idempotency_key: body.idempotency_key.clone(),
        mission_id: graph.mission.id.clone(),
        variant_id: variant_id.clone(),
        attempt_seed: body.idempotency_key.clone(),
        runner_id: body.runner_id.clone(),
        runner_epoch: body.runner_epoch,
        workspace_id: WorkspaceId::from_seed(&body.idempotency_key),
        workspace_nonce: *Digest::of(body.idempotency_key.as_bytes()).as_bytes(),
        scope_revision: 1,
        context_revision: 1,
        ttl_seconds: body.ttl_seconds,
    }
}

/// Rebuild the Unix-wire grant (durable row + authority token).
///
/// # Errors
///
/// Unknown package or token reconstruction failure.
pub fn wire_grant<L: Ledger>(
    ledger: &L,
    grant: &LeaseGrant,
    work_package_id: &WorkPackageId,
) -> Result<SignedLeaseWireGrant, SignedLeaseError> {
    let (graph, _) = graph_for_package(ledger, work_package_id)?;
    let authority_token =
        LeaseService::token_for(&graph, &grant.attempt).map_err(SignedLeaseError::Ledger)?;
    Ok(SignedLeaseWireGrant {
        grant: grant.clone(),
        authority_token,
    })
}

/// Internal Unix JSON-RPC request. Not a public `/v1` body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeaseTransportRpcRequest {
    /// Caller correlation id.
    pub id: String,
    /// `acquire`, `readback`, `heartbeat`, or `release`.
    pub method: String,
    /// Runner-issued permit.
    pub permit: SignedLeasePermit,
    /// Operation body (acquire body, heartbeat call, or release call).
    pub body: serde_json::Value,
    /// Required for heartbeat and release (not digested).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_package_id: Option<String>,
    /// Required for heartbeat and release (not digested).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    /// Required for release (not digested).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_id: Option<String>,
    /// Required for release (not digested).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner_epoch: Option<u64>,
}

/// Internal Unix JSON-RPC response.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeaseTransportRpcResponse {
    /// Caller correlation id.
    pub id: String,
    /// Success payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ok: Option<serde_json::Value>,
    /// Typed refusal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub err: Option<LeaseTransportRpcError>,
}

/// Typed Unix JSON-RPC refusal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeaseTransportRpcError {
    /// Stable reason code.
    pub code: String,
    /// Non-secret detail.
    pub message: String,
}

fn graph_for_package<L: Ledger>(
    ledger: &L,
    package: &WorkPackageId,
) -> Result<(StoredGraph, VariantId), SignedLeaseError> {
    for mission in ledger.list_missions().map_err(SignedLeaseError::Ledger)? {
        let Some(graph) = ledger
            .get_graph(&mission.id)
            .map_err(SignedLeaseError::Ledger)?
        else {
            continue;
        };
        if let Some(variant) = graph
            .variants
            .iter()
            .find(|variant| variant.work_package_id == *package)
        {
            let variant_id = variant.id.clone();
            return Ok((graph, variant_id));
        }
    }
    Err(SignedLeaseError::Unknown)
}
