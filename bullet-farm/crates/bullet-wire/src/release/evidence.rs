use crate::{
    WireError, hash_framed_bytes,
    v1alpha1::{
        ReleaseEvidenceKindV1, ReleaseEvidenceSubjectV1, ReleaseRegistryObjectKindV1,
        ReleaseRegistryObjectV1,
    },
};

use super::{
    RELEASE_BUNDLE_MANIFEST_V2_DIGEST_DOMAIN, RELEASE_BUNDLE_MANIFEST_V2_NATIVE_SUBJECT_PREFIX,
    fields::invalid, validate::ReleaseWireRecord,
};

const MAX_RELEASE_BUNDLE_MANIFEST_V2_BYTES: usize = 1024 * 1024;

/// Hashes the exact raw release-manifest-v2 bytes. This establishes content
/// addressability only; it does not parse the manifest or establish trust.
pub fn release_bundle_manifest_v2_digest(bytes: &[u8]) -> Result<String, WireError> {
    if bytes.is_empty() || bytes.len() > MAX_RELEASE_BUNDLE_MANIFEST_V2_BYTES {
        return Err(invalid(
            "release bundle manifest v2 bytes must be nonempty and at most 1 MiB",
        ));
    }
    Ok(format!(
        "blake3:{}",
        hash_framed_bytes(RELEASE_BUNDLE_MANIFEST_V2_DIGEST_DOMAIN, bytes)?.to_hex()
    ))
}

/// Binds one convention-named Artifact subject to one manifest-v2 registry
/// object and the exact framed raw bytes. Other evidence and object kinds are
/// outside this helper's authority.
pub fn validate_release_bundle_manifest_v2_binding(
    evidence_subjects: &[ReleaseEvidenceSubjectV1],
    registry_objects: &[ReleaseRegistryObjectV1],
    manifest_bytes: &[u8],
) -> Result<(), WireError> {
    for subject in evidence_subjects {
        subject.validate_release()?;
    }
    for object in registry_objects {
        object.validate_release()?;
    }

    let mut subjects = evidence_subjects
        .iter()
        .filter(|subject| subject.subject_kind == ReleaseEvidenceKindV1::Artifact);
    let subject = subjects.next().ok_or_else(|| {
        invalid("release bundle manifest v2 requires exactly one Artifact subject")
    })?;
    if subjects.next().is_some() {
        return Err(invalid(
            "release bundle manifest v2 requires exactly one Artifact subject",
        ));
    }

    let mut objects = registry_objects.iter().filter(|object| {
        object.object_kind == ReleaseRegistryObjectKindV1::ReleaseBundleManifestV2
    });
    let object = objects.next().ok_or_else(|| {
        invalid("release bundle manifest v2 requires exactly one registry object")
    })?;
    if objects.next().is_some() {
        return Err(invalid(
            "release bundle manifest v2 requires exactly one registry object",
        ));
    }

    let digest = release_bundle_manifest_v2_digest(manifest_bytes)?;
    let digest_hex = digest
        .strip_prefix("blake3:")
        .expect("release bundle manifest digest is algorithm tagged");
    let native_subject_id =
        format!("{RELEASE_BUNDLE_MANIFEST_V2_NATIVE_SUBJECT_PREFIX}{digest_hex}");
    if subject.native_subject_id != native_subject_id
        || subject.subject_digest != digest
        || object.object_digest != digest
    {
        return Err(invalid(
            "release bundle manifest v2 subject, object, and raw bytes do not share one exact digest binding",
        ));
    }
    Ok(())
}
