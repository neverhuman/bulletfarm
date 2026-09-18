//! Read-only receipt verification against one explicit admitted policy.

use std::{
    io::{Seek, SeekFrom, Write},
    path::Path,
};

use super::{
    MAX_POLICY_BYTES, MAX_RECEIPT_BYTES, MAX_SIGNATURE_BYTES, ReleaseReceipt, ReleaseReceiptPolicy,
    SIGNATURE_NAMESPACE, identity_parts, policy_digest, validate_identity,
};
use crate::{
    coord::CoordError,
    release::{signature, verify as input},
};

#[derive(Debug)]
pub(in crate::release) struct VerifiedReleaseReceipt {
    pub(in crate::release) receipt: ReleaseReceipt,
    pub(in crate::release) policy_digest: String,
}

pub(crate) struct VerifiedDetached {
    pub(crate) bytes: Vec<u8>,
    pub(crate) digest: String,
}

/// Verify one bounded payload against paths already selected by a higher-level
/// operator policy. This helper does not choose trust roots or interpret results.
#[allow(clippy::too_many_arguments)]
pub(crate) fn verify_detached(
    payload_path: &Path,
    signature_path: &Path,
    allowed_signers_path: &Path,
    signing_identity: &str,
    namespace: &str,
    label: &str,
    maximum: u64,
) -> Result<VerifiedDetached, CoordError> {
    validate_identity(signing_identity, |reason| {
        CoordError::new("INVALID_RELEASE_SIGNING_IDENTITY", reason)
    })?;
    signature::admit_verifier()?;
    let (mut allowed_signers, _) = snapshot(
        allowed_signers_path,
        "externally admitted allowed signers",
        MAX_POLICY_BYTES,
    )?;
    let (mut payload, bytes) = snapshot(payload_path, label, maximum)?;
    let (mut signature_input, _) = snapshot(
        signature_path,
        "detached release evidence signature",
        MAX_SIGNATURE_BYTES,
    )?;
    let (principal, fingerprint) = identity_parts(signing_identity).ok_or_else(|| {
        CoordError::new(
            "INVALID_RELEASE_SIGNING_IDENTITY",
            "release signing identity is malformed",
        )
    })?;
    let streamed = signature::verify(
        &signature_input.file,
        &allowed_signers.file,
        payload.file.try_clone().map_err(CoordError::io)?,
        principal,
        fingerprint,
        namespace,
        label,
    )?;
    let digest = blake3::hash(&bytes);
    if streamed.byte_count != bytes.len() as u64 || streamed.digest != *digest.as_bytes() {
        return Err(CoordError::new(
            "RELEASE_EVIDENCE_SUBJECT_MISMATCH",
            "streamed signature subject differs from the immutable evidence snapshot",
        ));
    }
    let _ = input::read_open_bounded(&mut allowed_signers.file, MAX_POLICY_BYTES)?;
    let _ = input::read_open_bounded(&mut payload.file, maximum)?;
    let _ = input::read_open_bounded(&mut signature_input.file, MAX_SIGNATURE_BYTES)?;
    Ok(VerifiedDetached {
        bytes,
        digest: format!("blake3:{}", digest.to_hex()),
    })
}

pub(in crate::release) fn verify(
    receipt_path: &Path,
    signature_path: &Path,
    policy_path: &Path,
) -> Result<VerifiedReleaseReceipt, CoordError> {
    signature::admit_verifier()?;
    let (mut policy_input, policy_bytes) =
        snapshot(policy_path, "receipt policy", MAX_POLICY_BYTES)?;
    let policy = ReleaseReceiptPolicy::parse(&policy_bytes)?;
    let policy_digest = policy_digest(&policy_bytes);
    let (mut receipt_input, receipt_bytes) =
        snapshot(receipt_path, "release receipt", MAX_RECEIPT_BYTES)?;
    let receipt = ReleaseReceipt::parse(&receipt_bytes)?;
    if receipt.family != policy.family || receipt.policy_digest != policy_digest {
        return Err(CoordError::new(
            "RELEASE_RECEIPT_POLICY_MISMATCH",
            "release receipt does not bind the exact admitted policy",
        ));
    }
    let signer = policy
        .signer
        .iter()
        .find(|signer| signer.release_signing_identity == receipt.release_signing_identity)
        .ok_or_else(|| {
            CoordError::new(
                "RELEASE_RECEIPT_SIGNER_NOT_ALLOWED",
                "release receipt signer is absent from the admitted policy",
            )
        })?;
    if signer
        .receipt_kind
        .binary_search_by(|kind| kind.as_str().cmp(receipt.receipt_kind.as_str()))
        .is_err()
        || receipt.started_at_unix_ms < signer.valid_from_unix_ms
        || receipt.expires_at_unix_ms > signer.valid_until_unix_ms
    {
        return Err(CoordError::new(
            "RELEASE_RECEIPT_NOT_AUTHORIZED",
            "signer policy does not authorize this receipt kind and full validity interval",
        ));
    }
    let (mut signature_input, _) = snapshot(
        signature_path,
        "release receipt signature",
        MAX_SIGNATURE_BYTES,
    )?;
    let mut allowed_signers = tempfile::tempfile().map_err(CoordError::io)?;
    let (principal, fingerprint) = signer.identity_parts();
    writeln!(
        allowed_signers,
        "{principal} namespaces=\"{SIGNATURE_NAMESPACE}\" {}",
        signer.public_key
    )
    .map_err(CoordError::io)?;
    allowed_signers.flush().map_err(CoordError::io)?;
    allowed_signers
        .seek(SeekFrom::Start(0))
        .map_err(CoordError::io)?;
    let allowed_signers = input::immutable_snapshot(
        &mut allowed_signers,
        "generated receipt allowed signers",
        MAX_POLICY_BYTES,
    )?;
    let streamed = signature::verify(
        &signature_input.file,
        &allowed_signers.file,
        receipt_input.file.try_clone().map_err(CoordError::io)?,
        principal,
        fingerprint,
        SIGNATURE_NAMESPACE,
        "release receipt signature verification",
    )?;
    if streamed.byte_count != receipt_input.byte_count
        || streamed.digest != *receipt_input.digest.as_bytes()
    {
        return Err(CoordError::new(
            "RELEASE_RECEIPT_SUBJECT_MISMATCH",
            "streamed signature subject differs from the canonical receipt",
        ));
    }
    let _ = input::read_open_bounded(&mut policy_input.file, MAX_POLICY_BYTES)?;
    let _ = input::read_open_bounded(&mut receipt_input.file, MAX_RECEIPT_BYTES)?;
    let _ = input::read_open_bounded(&mut signature_input.file, MAX_SIGNATURE_BYTES)?;
    Ok(VerifiedReleaseReceipt {
        receipt,
        policy_digest,
    })
}

fn snapshot(
    path: &Path,
    label: &str,
    maximum: u64,
) -> Result<(input::ImmutableSnapshot, Vec<u8>), CoordError> {
    let mut admitted = input::admitted_external_file(path, label, maximum)?;
    let mut snapshot = input::immutable_snapshot(&mut admitted, label, maximum)?;
    let bytes = input::read_open_bounded(&mut snapshot.file, maximum)?;
    Ok((snapshot, bytes))
}
