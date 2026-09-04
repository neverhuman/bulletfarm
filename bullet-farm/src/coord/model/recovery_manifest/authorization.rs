use serde::{Deserialize, Serialize};

use crate::coord::{CoordError, validate_field};

use super::{invalid, validate_prefixed};

const AUTHORIZATION_KIND: &str = "bullet.coord.recovery-authorization.v1";
const SIGNATURE_KIND: &str = "bullet.coord.recovery-authorization-signature.v1";
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
pub(crate) const MAX_AUTHORIZATION_WINDOW_MS: u64 = 24 * 60 * 60 * 1_000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum RecoveryAuthorizationDecisionV1 {
    Approve,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryAuthorizationV1 {
    pub(crate) kind: String,
    pub(crate) schema_version: u32,
    pub(crate) decision: RecoveryAuthorizationDecisionV1,
    pub(crate) inspection_id: String,
    pub(crate) inspection_sha256: String,
    pub(crate) recovery_operator: String,
    pub(crate) recovery_operator_uid: u32,
    pub(crate) reviewer_principal: String,
    pub(crate) reviewer_fingerprint: String,
    pub(crate) policy_namespace: String,
    pub(crate) bootstrap_provenance_sha256: String,
    pub(crate) decision_at_unix_ms: u64,
    pub(crate) authorized_at_unix_ms: u64,
    pub(crate) expires_at_unix_ms: u64,
    pub(crate) authority_boot_id: String,
    pub(crate) authority_time_namespace_device: u64,
    pub(crate) authority_time_namespace_inode: u64,
    pub(crate) authorized_at_boottime_ms: u64,
    pub(crate) expires_at_boottime_ms: u64,
}

impl RecoveryAuthorizationV1 {
    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        if self.kind != AUTHORIZATION_KIND || self.schema_version != 1 {
            return Err(invalid(
                "recovery authorization kind or schema version is unsupported",
            ));
        }
        validate_prefixed(&self.inspection_id, "rci_", 64, "inspection ID")?;
        validate_prefixed(&self.inspection_sha256, "sha256:", 64, "inspection SHA-256")?;
        validate_prefixed(
            &self.bootstrap_provenance_sha256,
            "sha256:",
            64,
            "bootstrap provenance SHA-256",
        )?;
        validate_field("recovery_operator", &self.recovery_operator)?;
        validate_field("reviewer_principal", &self.reviewer_principal)?;
        validate_field("policy_namespace", &self.policy_namespace)?;
        validate_prefixed(
            &self.reviewer_fingerprint,
            "sha256:",
            64,
            "reviewer fingerprint",
        )?;
        validate_linux_boot_id(&self.authority_boot_id)?;
        let unix_window = self
            .expires_at_unix_ms
            .checked_sub(self.authorized_at_unix_ms)
            .ok_or_else(|| invalid("authorization expiry precedes its issue time"))?;
        let boottime_window = self
            .expires_at_boottime_ms
            .checked_sub(self.authorized_at_boottime_ms)
            .ok_or_else(|| invalid("authorization boot-time expiry precedes its issue time"))?;
        if self.decision_at_unix_ms == 0
            || self.decision_at_unix_ms > self.authorized_at_unix_ms
            || self.authorized_at_unix_ms == 0
            || self.expires_at_unix_ms > MAX_SAFE_INTEGER
            || self.expires_at_boottime_ms > MAX_SAFE_INTEGER
            || self.authority_time_namespace_device == 0
            || self.authority_time_namespace_device > MAX_SAFE_INTEGER
            || self.authority_time_namespace_inode == 0
            || self.authority_time_namespace_inode > MAX_SAFE_INTEGER
            || unix_window == 0
            || unix_window > MAX_AUTHORIZATION_WINDOW_MS
            || boottime_window != unix_window
        {
            return Err(invalid(
                "recovery authorization must have one equal positive maximum-24-hour Unix/boot window after its decision",
            ));
        }
        Ok(())
    }
}

pub(crate) fn validate_linux_boot_id(value: &str) -> Result<(), CoordError> {
    let bytes = value.as_bytes();
    let valid = bytes.len() == 36
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => *byte == b'-',
            _ => byte.is_ascii_digit() || (b'a'..=b'f').contains(byte),
        });
    if !valid {
        return Err(invalid(
            "authorization boot ID must be a lowercase Linux UUID",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryAuthorizationSignatureV1 {
    pub(crate) kind: String,
    pub(crate) schema_version: u32,
    pub(crate) namespace: String,
    pub(crate) reviewer_principal: String,
    pub(crate) reviewer_fingerprint: String,
    pub(crate) authorization_sha256: String,
    pub(crate) signature_ed25519: String,
}

impl RecoveryAuthorizationSignatureV1 {
    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        if self.kind != SIGNATURE_KIND || self.schema_version != 1 {
            return Err(invalid(
                "authorization signature kind or schema version is unsupported",
            ));
        }
        validate_field("signature namespace", &self.namespace)?;
        validate_field("signature reviewer", &self.reviewer_principal)?;
        validate_prefixed(
            &self.reviewer_fingerprint,
            "sha256:",
            64,
            "signature reviewer fingerprint",
        )?;
        validate_prefixed(
            &self.authorization_sha256,
            "sha256:",
            64,
            "authorization SHA-256",
        )?;
        validate_prefixed(
            &self.signature_ed25519,
            "ed25519:",
            128,
            "authorization Ed25519 signature",
        )
    }
}
