use ed25519_compact::{PublicKey, Signature};

use crate::coord::{
    CoordError,
    model::{
        RecoveryAuthorizationDecisionV1, RecoveryAuthorizationSignatureV1, RecoveryAuthorizationV1,
        RecoveryBootstrapProvenanceV1, RecoveryInspectionV1,
    },
};

use super::trust::{self, InstalledPolicy};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TrustedRecoveryClockObservation {
    pub(crate) unix_ms: u64,
    pub(crate) boottime_ms: u64,
    pub(crate) boot_id: String,
    pub(crate) time_namespace_device: u64,
    pub(crate) time_namespace_inode: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecoveryAuthorizationDraftInput {
    pub(crate) decision: RecoveryAuthorizationDecisionV1,
    pub(crate) recovery_operator: String,
    pub(crate) recovery_operator_uid: u32,
    pub(crate) reviewer_principal: String,
    pub(crate) reviewer_fingerprint: String,
    pub(crate) policy_namespace: String,
    pub(crate) decision_at_unix_ms: u64,
    pub(crate) authorized_at_unix_ms: u64,
    pub(crate) expires_at_unix_ms: u64,
    pub(crate) authority_boot_id: String,
    pub(crate) authority_time_namespace_device: u64,
    pub(crate) authority_time_namespace_inode: u64,
    pub(crate) authorized_at_boottime_ms: u64,
    pub(crate) expires_at_boottime_ms: u64,
    pub(crate) trusted_clock: TrustedRecoveryClockObservation,
}

/// Operator/reviewer facts accepted by the future CLI boundary. Clock facts
/// are intentionally absent: they come only from the root-custodied observer.
#[cfg(target_os = "linux")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObservedRecoveryAuthorizationDraftInput {
    pub(crate) decision: RecoveryAuthorizationDecisionV1,
    pub(crate) recovery_operator: String,
    pub(crate) recovery_operator_uid: u32,
    pub(crate) reviewer_principal: String,
    pub(crate) reviewer_fingerprint: String,
    pub(crate) policy_namespace: String,
    pub(crate) validity_window_ms: u64,
}

/// Draft from one admitted Linux clock observation, deriving every time and
/// namespace field rather than accepting it from argv or a document.
#[cfg(target_os = "linux")]
pub(crate) fn draft_observed(
    inspection: &RecoveryInspectionV1,
    provenance: &RecoveryBootstrapProvenanceV1,
    input: ObservedRecoveryAuthorizationDraftInput,
) -> Result<RecoveryAuthorizationV1, CoordError> {
    // Production policy is deliberately checked before even the root-runtime
    // clock path is observed.
    let _ = trust::installed_policy()?;
    let observed = trusted_observation(super::clock::observe()?);
    let expires_at_unix_ms = observed
        .unix_ms
        .checked_add(input.validity_window_ms)
        .ok_or_else(|| invalid("authorization Unix expiry overflows"))?;
    let expires_at_boottime_ms = observed
        .boottime_ms
        .checked_add(input.validity_window_ms)
        .ok_or_else(|| invalid("authorization boot-time expiry overflows"))?;
    draft(
        inspection,
        provenance,
        RecoveryAuthorizationDraftInput {
            decision: input.decision,
            recovery_operator: input.recovery_operator,
            recovery_operator_uid: input.recovery_operator_uid,
            reviewer_principal: input.reviewer_principal,
            reviewer_fingerprint: input.reviewer_fingerprint,
            policy_namespace: input.policy_namespace,
            decision_at_unix_ms: observed.unix_ms,
            authorized_at_unix_ms: observed.unix_ms,
            expires_at_unix_ms,
            authority_boot_id: observed.boot_id.clone(),
            authority_time_namespace_device: observed.time_namespace_device,
            authority_time_namespace_inode: observed.time_namespace_inode,
            authorized_at_boottime_ms: observed.boottime_ms,
            expires_at_boottime_ms,
            trusted_clock: observed,
        },
    )
}

/// Re-observe the accepted clock immediately before create-once publication.
#[cfg(target_os = "linux")]
pub(crate) fn require_observed_current(
    authorization: &RecoveryAuthorizationV1,
) -> Result<(), CoordError> {
    let policy = trust::installed_policy()?;
    authorization.validate()?;
    policy.require_authorization_identity(authorization)?;
    let observed = trusted_observation(super::clock::observe()?);
    require_observation_inside(authorization, &observed)
}

#[cfg(target_os = "linux")]
fn trusted_observation(value: super::trust::ClockObservation) -> TrustedRecoveryClockObservation {
    TrustedRecoveryClockObservation {
        unix_ms: value.unix_ms,
        boottime_ms: value.boottime_ms,
        boot_id: value.boot_id,
        time_namespace_device: value.time_namespace_device,
        time_namespace_inode: value.time_namespace_inode,
    }
}

pub(crate) fn draft(
    inspection: &RecoveryInspectionV1,
    provenance: &RecoveryBootstrapProvenanceV1,
    input: RecoveryAuthorizationDraftInput,
) -> Result<RecoveryAuthorizationV1, CoordError> {
    let policy = trust::installed_policy()?;
    inspection.validate()?;
    provenance.validate()?;
    let authorization = RecoveryAuthorizationV1 {
        kind: "bullet.coord.recovery-authorization.v1".to_owned(),
        schema_version: 1,
        decision: input.decision,
        inspection_id: inspection.inspection_id.clone(),
        inspection_sha256: inspection.sealed_sha256()?.as_str().to_owned(),
        recovery_operator: input.recovery_operator,
        recovery_operator_uid: input.recovery_operator_uid,
        reviewer_principal: input.reviewer_principal,
        reviewer_fingerprint: input.reviewer_fingerprint,
        policy_namespace: input.policy_namespace,
        bootstrap_provenance_sha256: trust::sealed_sha256(provenance)?,
        decision_at_unix_ms: input.decision_at_unix_ms,
        authorized_at_unix_ms: input.authorized_at_unix_ms,
        expires_at_unix_ms: input.expires_at_unix_ms,
        authority_boot_id: input.authority_boot_id,
        authority_time_namespace_device: input.authority_time_namespace_device,
        authority_time_namespace_inode: input.authority_time_namespace_inode,
        authorized_at_boottime_ms: input.authorized_at_boottime_ms,
        expires_at_boottime_ms: input.expires_at_boottime_ms,
    };
    authorization.validate()?;
    policy.require_authorization_identity(&authorization)?;
    if inspection.subject.incident_at_unix_ms == 0
        || authorization.decision_at_unix_ms <= inspection.subject.incident_at_unix_ms
    {
        return Err(invalid(
            "authorization decision time must follow the derived incident time",
        ));
    }
    require_observation_inside(&authorization, &input.trusted_clock)?;
    Ok(authorization)
}

pub(crate) fn signing_message(
    authorization: &RecoveryAuthorizationV1,
) -> Result<Vec<u8>, CoordError> {
    let policy = trust::installed_policy()?;
    let canonical = canonical_authorization(&policy, authorization)?;
    Ok(trust::signing_message(&canonical))
}

pub(crate) fn import_signature(
    authorization: &RecoveryAuthorizationV1,
    raw_signature: &[u8],
) -> Result<RecoveryAuthorizationSignatureV1, CoordError> {
    let policy = trust::installed_policy()?;
    let canonical = canonical_authorization(&policy, authorization)?;
    let signature_bytes: &[u8; 64] = raw_signature
        .try_into()
        .map_err(|_| invalid("raw Ed25519 signature must contain exactly 64 bytes"))?;
    let signature_value = Signature::from_slice(signature_bytes)
        .map_err(|_| invalid("recovery authorization signature encoding is invalid"))?;
    let public_key = PublicKey::from_slice(&policy.reviewer_public_key)
        .map_err(|_| invalid("installed recovery reviewer public key is invalid"))?;
    public_key
        .verify(trust::signing_message(&canonical), &signature_value)
        .map_err(|_| invalid("recovery authorization signature did not verify"))?;
    let signature = RecoveryAuthorizationSignatureV1 {
        kind: "bullet.coord.recovery-authorization-signature.v1".to_owned(),
        schema_version: 1,
        namespace: policy.namespace.clone(),
        reviewer_principal: policy.reviewer_principal.clone(),
        reviewer_fingerprint: policy.reviewer_fingerprint.clone(),
        authorization_sha256: trust::sha256_line(&canonical),
        signature_ed25519: format!("ed25519:{}", trust::encode_hex(signature_bytes)),
    };
    signature.validate()?;
    policy.require_signature_identity(&signature)?;
    Ok(signature)
}

fn canonical_authorization(
    policy: &InstalledPolicy,
    authorization: &RecoveryAuthorizationV1,
) -> Result<Vec<u8>, CoordError> {
    authorization.validate()?;
    policy.require_authorization_identity(authorization)?;
    bullet_wire::canonical_json(authorization)
        .map_err(|error| invalid(format!("cannot canonicalize recovery authority: {error}")))
}

fn require_observation_inside(
    authorization: &RecoveryAuthorizationV1,
    clock: &TrustedRecoveryClockObservation,
) -> Result<(), CoordError> {
    if authorization.authority_boot_id != clock.boot_id {
        return Err(CoordError::new(
            "RECOVERY_AUTHORIZATION_BOOT_CHANGED",
            "recovery authorization names a different Linux boot epoch",
        ));
    }
    if (
        authorization.authority_time_namespace_device,
        authorization.authority_time_namespace_inode,
    ) != (clock.time_namespace_device, clock.time_namespace_inode)
    {
        return Err(CoordError::new(
            "RECOVERY_TIME_NAMESPACE_CHANGED",
            "recovery authorization names a different Linux time namespace",
        ));
    }
    if clock.unix_ms < authorization.authorized_at_unix_ms
        || clock.boottime_ms < authorization.authorized_at_boottime_ms
    {
        return Err(CoordError::new(
            "RECOVERY_AUTHORIZATION_NOT_YET_VALID",
            "trusted clock observation precedes the recovery authorization window",
        ));
    }
    if clock.unix_ms >= authorization.expires_at_unix_ms
        || clock.boottime_ms >= authorization.expires_at_boottime_ms
    {
        return Err(CoordError::new(
            "RECOVERY_AUTHORIZATION_EXPIRED",
            "trusted clock observation is outside the recovery authorization window",
        ));
    }
    Ok(())
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_AUTHORIZATION", reason)
}

#[cfg(test)]
#[path = "authoring/tests.rs"]
mod tests;
