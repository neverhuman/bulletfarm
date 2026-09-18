use std::{fs::File, io::Read, os::unix::fs::MetadataExt};

use ed25519_compact::{PublicKey, Signature};
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::coord::{
    CoordError,
    model::{
        RecoveryAuthorizationDecisionV1, RecoveryAuthorizationSignatureV1, RecoveryAuthorizationV1,
        RecoveryBootstrapProvenanceV1, RecoveryInspectionV1,
    },
};

const SIGNATURE_DOMAIN: &[u8] = b"bullet-family.coord.recovery-authorization-signature.v1\0";
const DECISION_DOMAIN: &[u8] = b"bullet-family.coord.recovery-operator-decision.v1\0";
const REPLAY_DOMAIN: &[u8] = b"bullet-family.coord.recovery-replay-contract.v1\0";
const MAX_EXECUTABLE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct OperatorDecisionSubject<'a> {
    kind: &'static str,
    schema_version: u32,
    decision: &'a RecoveryAuthorizationDecisionV1,
    inspection_id: &'a str,
    inspection_sha256: &'a str,
    recovery_operator: &'a str,
    recovery_operator_uid: u32,
    reviewer_principal: &'a str,
    reviewer_fingerprint: &'a str,
    policy_namespace: &'a str,
    bootstrap_provenance_sha256: &'a str,
    decision_at_unix_ms: u64,
}

#[path = "trust/policy.rs"]
mod policy;
pub(super) use policy::{InstalledPolicy, installed_policy, policy_sha256};
#[cfg(test)]
use policy::{
    MAX_AUTHORIZATION_WINDOW_MS, OPERATOR_IDENTITY, POLICY_NAMESPACE, REVIEWER_PRINCIPAL,
    fingerprint,
};
use policy::{REPLAY_CONTRACT_IDENTITY, REPLAY_CONTRACT_VERSION};
#[cfg(test)]
pub(in crate::coord) use policy::{TestPolicyGuard, install_test_policy};
#[path = "trust/window.rs"]
mod window;
pub(super) use window::{ClockObservation, VerifiedAuthorization};

#[cfg(test)]
pub(super) fn verify(
    inspection: &RecoveryInspectionV1,
    authorization: &RecoveryAuthorizationV1,
    signature: &RecoveryAuthorizationSignatureV1,
    provenance: &RecoveryBootstrapProvenanceV1,
    clock: ClockObservation,
) -> Result<VerifiedAuthorization, CoordError> {
    let policy = installed_policy()?;
    verify_with_policy(
        &policy,
        inspection,
        authorization,
        signature,
        provenance,
        clock,
    )
}

pub(super) fn verify_observed(
    inspection: &RecoveryInspectionV1,
    authorization: &RecoveryAuthorizationV1,
    signature: &RecoveryAuthorizationSignatureV1,
    provenance: &RecoveryBootstrapProvenanceV1,
    observe_clock: impl FnOnce() -> Result<ClockObservation, CoordError>,
) -> Result<VerifiedAuthorization, CoordError> {
    let policy = installed_policy()?;
    let clock = observe_clock()?;
    verify_with_policy(
        &policy,
        inspection,
        authorization,
        signature,
        provenance,
        clock,
    )
}

fn verify_with_policy(
    policy: &InstalledPolicy,
    inspection: &RecoveryInspectionV1,
    authorization: &RecoveryAuthorizationV1,
    signature: &RecoveryAuthorizationSignatureV1,
    provenance: &RecoveryBootstrapProvenanceV1,
    clock: ClockObservation,
) -> Result<VerifiedAuthorization, CoordError> {
    inspection.validate()?;
    authorization.validate()?;
    signature.validate()?;
    provenance.validate()?;
    if authorization.inspection_id != inspection.inspection_id
        || authorization.inspection_sha256 != inspection.sealed_sha256()?.as_str()
    {
        return Err(invalid(
            "authorization does not bind the exact sealed inspection",
        ));
    }
    if authorization.bootstrap_provenance_sha256 != sealed_sha256(provenance)? {
        return Err(invalid(
            "authorization does not bind the exact sealed bootstrap provenance",
        ));
    }
    policy.require_authorization_identity(authorization)?;
    if rustix::process::geteuid().as_raw() != policy.operator_uid {
        return Err(invalid(
            "recovery operator identity or effective UID is not admitted by policy",
        ));
    }
    policy.require_signature_identity(signature)?;
    let authorization_bytes = bullet_wire::canonical_json(authorization).map_err(wire)?;
    let authorization_sha256 = sha256_line(&authorization_bytes);
    if signature.authorization_sha256 != authorization_sha256 {
        return Err(invalid(
            "detached signature does not bind the exact authorization",
        ));
    }
    let public_key = PublicKey::from_slice(&policy.reviewer_public_key)
        .map_err(|_| invalid("installed recovery reviewer public key is invalid"))?;
    let signature_bytes = decode_hex::<64>(&signature.signature_ed25519, "ed25519:")?;
    let signature_value = Signature::from_slice(&signature_bytes)
        .map_err(|_| invalid("recovery authorization signature encoding is invalid"))?;
    public_key
        .verify(signing_message(&authorization_bytes), &signature_value)
        .map_err(|_| invalid("recovery authorization signature did not verify"))?;
    verify_self_executable(provenance)?;
    if authorization.decision_at_unix_ms <= inspection.subject.incident_at_unix_ms {
        return Err(invalid(
            "authorization decision time must follow the derived incident time",
        ));
    }
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
    let decision = OperatorDecisionSubject {
        kind: "bullet.coord.recovery-operator-decision.v1",
        schema_version: 1,
        decision: &authorization.decision,
        inspection_id: &authorization.inspection_id,
        inspection_sha256: &authorization.inspection_sha256,
        recovery_operator: &authorization.recovery_operator,
        recovery_operator_uid: authorization.recovery_operator_uid,
        reviewer_principal: &authorization.reviewer_principal,
        reviewer_fingerprint: &authorization.reviewer_fingerprint,
        policy_namespace: &authorization.policy_namespace,
        bootstrap_provenance_sha256: &authorization.bootstrap_provenance_sha256,
        decision_at_unix_ms: authorization.decision_at_unix_ms,
    };
    let decision_bytes = bullet_wire::canonical_json(&decision).map_err(wire)?;
    Ok(VerifiedAuthorization {
        recovery_operator: authorization.recovery_operator.clone(),
        policy_sha256: policy_sha256(policy)?,
        operator_decision_sha256: sha256_domain(DECISION_DOMAIN, &decision_bytes),
        replay_contract_version: REPLAY_CONTRACT_VERSION,
        replay_contract_sha256: sha256_domain(REPLAY_DOMAIN, REPLAY_CONTRACT_IDENTITY.as_bytes()),
        bootstrap_commit_oid: provenance.bootstrap_commit_oid.clone(),
        bootstrap_paths: provenance.bootstrap_paths(),
        decision_at_unix_ms: authorization.decision_at_unix_ms,
        authority_boot_id: authorization.authority_boot_id.clone(),
        authority_time_namespace_device: authorization.authority_time_namespace_device,
        authority_time_namespace_inode: authorization.authority_time_namespace_inode,
        authorized_at_unix_ms: authorization.authorized_at_unix_ms,
        expires_at_unix_ms: authorization.expires_at_unix_ms,
        authorized_at_boottime_ms: authorization.authorized_at_boottime_ms,
        expires_at_boottime_ms: authorization.expires_at_boottime_ms,
    })
}

fn verify_self_executable(provenance: &RecoveryBootstrapProvenanceV1) -> Result<(), CoordError> {
    let (first, first_identity) = read_self_executable()?;
    let (second, second_identity) = read_self_executable()?;
    if first_identity != second_identity || first != second {
        return Err(CoordError::new(
            "RECOVERY_BOOTSTRAP_MISMATCH",
            "/proc/self/exe changed across independent reads",
        ));
    }
    if first.len() as u64 != provenance.executable_byte_length
        || format!("sha256:{:x}", Sha256::digest(&first)) != provenance.executable_sha256
    {
        return Err(CoordError::new(
            "RECOVERY_BOOTSTRAP_MISMATCH",
            "running /proc/self/exe differs from sealed bootstrap provenance",
        ));
    }
    Ok(())
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct ExecutableIdentity {
    device: u64,
    inode: u64,
    mode: u32,
    owner_uid: u32,
    owner_gid: u32,
    links: u64,
    length: u64,
    mtime_seconds: i64,
    mtime_nanoseconds: i64,
    ctime_seconds: i64,
    ctime_nanoseconds: i64,
}

fn read_self_executable() -> Result<(Vec<u8>, ExecutableIdentity), CoordError> {
    let mut file = File::open("/proc/self/exe").map_err(CoordError::io)?;
    let before = executable_identity(&file)?;
    if before.length == 0 || before.length > MAX_EXECUTABLE_BYTES {
        return Err(CoordError::new(
            "RECOVERY_BOOTSTRAP_MISMATCH",
            "running executable length is outside the admitted bound",
        ));
    }
    let mut bytes = Vec::new();
    (&mut file)
        .take(MAX_EXECUTABLE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    let after = executable_identity(&file)?;
    if before != after || bytes.len() as u64 != before.length {
        return Err(CoordError::new(
            "RECOVERY_BOOTSTRAP_MISMATCH",
            "running executable changed while being read",
        ));
    }
    Ok((bytes, before))
}

fn executable_identity(file: &File) -> Result<ExecutableIdentity, CoordError> {
    let value = file.metadata().map_err(CoordError::io)?;
    if !value.is_file() {
        return Err(CoordError::new(
            "RECOVERY_BOOTSTRAP_MISMATCH",
            "/proc/self/exe is not a regular file",
        ));
    }
    Ok(ExecutableIdentity {
        device: value.dev(),
        inode: value.ino(),
        mode: value.mode(),
        owner_uid: value.uid(),
        owner_gid: value.gid(),
        links: value.nlink(),
        length: value.len(),
        mtime_seconds: value.mtime(),
        mtime_nanoseconds: value.mtime_nsec(),
        ctime_seconds: value.ctime(),
        ctime_nanoseconds: value.ctime_nsec(),
    })
}

pub(super) fn sealed_sha256(value: &impl Serialize) -> Result<String, CoordError> {
    let bytes = bullet_wire::canonical_json(value).map_err(wire)?;
    Ok(sha256_line(&bytes))
}

pub(super) fn sha256_line(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.update(b"\n");
    format!("sha256:{:x}", hasher.finalize())
}

fn sha256_domain(domain: &[u8], bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

pub(super) fn signing_message(authorization_bytes: &[u8]) -> Vec<u8> {
    let mut message = Vec::with_capacity(SIGNATURE_DOMAIN.len() + authorization_bytes.len());
    message.extend_from_slice(SIGNATURE_DOMAIN);
    message.extend_from_slice(authorization_bytes);
    message
}

fn decode_hex<const N: usize>(value: &str, prefix: &str) -> Result<[u8; N], CoordError> {
    let hex = value
        .strip_prefix(prefix)
        .filter(|hex| hex.len() == N * 2)
        .ok_or_else(|| invalid("cryptographic value has the wrong domain or length"))?;
    let mut output = [0_u8; N];
    for (index, pair) in hex.as_bytes().chunks_exact(2).enumerate() {
        output[index] = (nibble(pair[0])? << 4) | nibble(pair[1])?;
    }
    Ok(output)
}

fn nibble(value: u8) -> Result<u8, CoordError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(invalid(
            "cryptographic value must use lowercase hexadecimal",
        )),
    }
}

pub(super) fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_AUTHORIZATION", reason)
}

fn wire(error: impl std::fmt::Display) -> CoordError {
    invalid(format!("cannot canonicalize recovery authority: {error}"))
}

#[cfg(test)]
#[path = "trust/tests.rs"]
mod tests;
#[cfg(test)]
pub(in crate::coord) use tests::{TestAuthority, test_authority, test_authority_with_decision};
