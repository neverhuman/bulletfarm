use crate::{
    WireError,
    v1alpha1::{
        ReleaseEvidenceKindV1, ReleaseRegistryObjectKindV1, ReleaseRepositoryNameV1,
        ReleaseSignerRoleV1,
    },
};

pub(super) const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
pub(super) const REQUIRED_REPOSITORIES: [&str; 4] = [
    "bullet-farm",
    "bullet-git",
    "bullet-kernel",
    "bullet-portal",
];
pub(super) const REQUIRED_COMMON_EVIDENCE: [&str; 4] =
    ["environment", "policy", "schema", "toolchain"];
pub(super) const REQUIRED_SIGNER_ROLES: [&str; 5] = [
    "artifact-release",
    "gate-attestor",
    "registry-curator",
    "source-tag",
    "trusted-time",
];

pub(super) fn schema(value: &str) -> Result<(), WireError> {
    if value == "v1alpha1" {
        Ok(())
    } else {
        Err(invalid("release record schema is unsupported"))
    }
}

pub(super) fn family(value: &str) -> Result<(), WireError> {
    if value == "bullet-farm" {
        Ok(())
    } else {
        Err(invalid("release record names the wrong family"))
    }
}

pub(super) fn positive(value: u64, label: &str) -> Result<(), WireError> {
    if value == 0 || value > MAX_SAFE_INTEGER {
        Err(invalid(format!(
            "{label} is outside the exact integer range"
        )))
    } else {
        Ok(())
    }
}

pub(super) fn ordered_pair(first: u64, second: u64, label: &str) -> Result<(), WireError> {
    positive(first, label)?;
    positive(second, label)?;
    if first >= second {
        Err(invalid(format!("{label} is empty or reversed")))
    } else {
        Ok(())
    }
}

pub(super) fn ordered_times(
    first: u64,
    second: u64,
    third: u64,
    label: &str,
) -> Result<(), WireError> {
    ordered_pair(first, second, label)?;
    positive(third, label)?;
    if second >= third {
        Err(invalid(format!("{label} expiry is not after completion")))
    } else {
        Ok(())
    }
}

pub(super) fn raw_digest(value: &str, label: &str) -> Result<(), WireError> {
    lower_hex(value, 64, label)
}

pub(super) fn tagged_digest(value: &str, label: &str) -> Result<(), WireError> {
    value
        .strip_prefix("blake3:")
        .ok_or_else(|| invalid(format!("{label} digest is not algorithm tagged")))
        .and_then(|hex| lower_hex(hex, 64, label))
}

fn lower_hex(value: &str, length: usize, label: &str) -> Result<(), WireError> {
    if value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(invalid(format!(
            "{label} is not full lowercase hexadecimal"
        )))
    }
}

pub(super) fn typed_id(value: &str, prefix: &str) -> Result<(), WireError> {
    value
        .strip_prefix(&format!("{prefix}_"))
        .ok_or_else(|| invalid(format!("identifier does not use {prefix}_")))
        .and_then(|hex| lower_hex(hex, 64, "identifier"))
}

pub(super) fn key_id(value: &str) -> Result<(), WireError> {
    if !value.is_empty()
        && value.len() <= 128
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric()
                || (index != 0 && matches!(byte, b'.' | b'_' | b':' | b'/' | b'-'))
        })
    {
        Ok(())
    } else {
        Err(invalid("release key ID is malformed"))
    }
}

pub(super) fn gate_id(value: &str) -> Result<(), WireError> {
    let Some(rest) = value.strip_prefix("release.") else {
        return Err(invalid("release gate ID lacks its namespace"));
    };
    if rest.is_empty()
        || value.len() > 128
        || !rest.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
    {
        Err(invalid("release gate ID is malformed"))
    } else {
        Ok(())
    }
}

pub(super) fn profile_id(value: &str) -> Result<(), WireError> {
    let first = value.as_bytes().first().copied();
    let last = value.as_bytes().last().copied();
    if value.len() < 2
        || value.len() > 64
        || first.is_none_or(|byte| !byte.is_ascii_lowercase())
        || last.is_none_or(|byte| !byte.is_ascii_alphanumeric())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        Err(invalid("release profile ID is malformed"))
    } else {
        Ok(())
    }
}

pub(super) fn sorted_unique(
    values: &[String],
    validate: fn(&str) -> Result<(), WireError>,
    label: &str,
) -> Result<(), WireError> {
    if values.is_empty() || values.len() > 64 {
        return Err(invalid(format!("{label} require 1..=64 values")));
    }
    let mut previous = None;
    for value in values {
        validate(value)?;
        if previous.is_some_and(|prior: &str| prior >= value.as_str()) {
            return Err(invalid(format!("{label} must be byte-sorted and unique")));
        }
        previous = Some(value);
    }
    Ok(())
}

pub(super) fn sorted_unique_optional(
    values: &[String],
    validate: fn(&str) -> Result<(), WireError>,
    label: &str,
) -> Result<(), WireError> {
    if values.len() > 64 {
        return Err(invalid(format!("{label} exceed 64 values")));
    }
    let mut previous = None;
    for value in values {
        validate(value)?;
        if previous.is_some_and(|prior: &str| prior >= value.as_str()) {
            return Err(invalid(format!("{label} must be byte-sorted and unique")));
        }
        previous = Some(value);
    }
    Ok(())
}

pub(super) fn native_subject_id(value: &str, kind: ReleaseEvidenceKindV1) -> Result<(), WireError> {
    let (namespace, typed) = value
        .split_once(':')
        .ok_or_else(|| invalid("native subject ID lacks its evidence namespace"))?;
    if namespace != evidence_kind(kind) {
        return Err(invalid(
            "native subject ID namespace does not match its kind",
        ));
    }
    let (prefix, hex) = typed
        .split_once('_')
        .ok_or_else(|| invalid("native subject ID lacks a typed prefix"))?;
    if !(2..=32).contains(&prefix.len())
        || !prefix.as_bytes()[0].is_ascii_lowercase()
        || !prefix
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(invalid("native subject ID prefix is malformed"));
    }
    lower_hex(hex, 64, "native subject ID")
}

pub(super) fn release_tag(value: &str) -> Result<(), WireError> {
    if value.len() < 2
        || value.len() > 128
        || !value.starts_with('v')
        || !value.as_bytes().get(1).is_some_and(u8::is_ascii_digit)
        || value.ends_with(['.', '-'])
        || value.contains("..")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
    {
        Err(invalid("release tag is malformed"))
    } else {
        Ok(())
    }
}

pub(super) fn git_oid(value: &str) -> Result<(), WireError> {
    match value.split_once(':') {
        Some(("sha1", hex)) => lower_hex(hex, 40, "Git OID"),
        Some(("sha256", hex)) => lower_hex(hex, 64, "Git OID"),
        _ => Err(invalid("Git OID is not algorithm tagged")),
    }
}

pub(super) fn signing_identity(value: &str) -> Result<(), WireError> {
    let mut parts = value.split('|');
    let principal = parts.next().unwrap_or_default();
    let algorithm = parts.next().unwrap_or_default();
    let fingerprint = parts.next().unwrap_or_default();
    let fingerprint_body = fingerprint.strip_prefix("SHA256:");
    if parts.next().is_some()
        || principal.is_empty()
        || principal.len() > 128
        || principal
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte == b'|')
        || algorithm != "ed25519"
        || fingerprint_body.is_none_or(|body| {
            !(16..=96).contains(&body.len())
                || !body
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))
        })
    {
        Err(invalid("release signing identity is malformed"))
    } else {
        Ok(())
    }
}

pub(super) fn public_key(value: &str) -> Result<(), WireError> {
    let Some(blob) = value.strip_prefix("ssh-ed25519 ") else {
        return Err(invalid("release public key is not SSH Ed25519"));
    };
    if !(40..=256).contains(&blob.len())
        || !blob
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))
    {
        Err(invalid("release public key is malformed"))
    } else {
        Ok(())
    }
}

pub(super) fn relative_path(value: &str) -> Result<(), WireError> {
    if value.is_empty()
        || value.len() > 4096
        || !value.is_ascii()
        || value.starts_with('/')
        || value.ends_with('/')
        || value.contains('\\')
        || value.split('/').any(|part| {
            part.is_empty() || matches!(part, "." | "..") || part.eq_ignore_ascii_case(".git")
        })
    {
        Err(invalid("release registry path is unsafe"))
    } else {
        Ok(())
    }
}

pub(super) const fn repository_name(value: ReleaseRepositoryNameV1) -> &'static str {
    match value {
        ReleaseRepositoryNameV1::BulletFarm => "bullet-farm",
        ReleaseRepositoryNameV1::BulletGit => "bullet-git",
        ReleaseRepositoryNameV1::BulletKernel => "bullet-kernel",
        ReleaseRepositoryNameV1::BulletPortal => "bullet-portal",
    }
}

pub(super) const fn evidence_kind(value: ReleaseEvidenceKindV1) -> &'static str {
    match value {
        ReleaseEvidenceKindV1::Artifact => "artifact",
        ReleaseEvidenceKindV1::AuditAnchor => "audit-anchor",
        ReleaseEvidenceKindV1::Candidate => "candidate",
        ReleaseEvidenceKindV1::Check => "check",
        ReleaseEvidenceKindV1::Configuration => "configuration",
        ReleaseEvidenceKindV1::Effect => "effect",
        ReleaseEvidenceKindV1::Environment => "environment",
        ReleaseEvidenceKindV1::Evidence => "evidence",
        ReleaseEvidenceKindV1::Integration => "integration",
        ReleaseEvidenceKindV1::Jeryu => "jeryu",
        ReleaseEvidenceKindV1::Observation => "observation",
        ReleaseEvidenceKindV1::Platform => "platform",
        ReleaseEvidenceKindV1::Policy => "policy",
        ReleaseEvidenceKindV1::ProfileGraph => "profile-graph",
        ReleaseEvidenceKindV1::ProofBundle => "proof-bundle",
        ReleaseEvidenceKindV1::Provider => "provider",
        ReleaseEvidenceKindV1::Provenance => "provenance",
        ReleaseEvidenceKindV1::Sandbox => "sandbox",
        ReleaseEvidenceKindV1::Sbom => "sbom",
        ReleaseEvidenceKindV1::Scanner => "scanner",
        ReleaseEvidenceKindV1::Schema => "schema",
        ReleaseEvidenceKindV1::Toolchain => "toolchain",
        ReleaseEvidenceKindV1::Transaction => "transaction",
    }
}

pub(super) const fn registry_object_kind(value: ReleaseRegistryObjectKindV1) -> &'static str {
    match value {
        ReleaseRegistryObjectKindV1::GateReceipt => "gate-receipt",
        ReleaseRegistryObjectKindV1::GateReceiptSignature => "gate-receipt-signature",
        ReleaseRegistryObjectKindV1::GateSpec => "gate-spec",
        ReleaseRegistryObjectKindV1::ProfileGraph => "profile-graph",
        ReleaseRegistryObjectKindV1::ReleaseBundleManifestV2 => "release-bundle-manifest-v2",
        ReleaseRegistryObjectKindV1::SignerPolicy => "signer-policy",
        ReleaseRegistryObjectKindV1::TrustedTimeObservation => "trusted-time-observation",
        ReleaseRegistryObjectKindV1::TrustedTimeSignature => "trusted-time-signature",
        ReleaseRegistryObjectKindV1::VerificationRequest => "verification-request",
    }
}

pub(super) const fn signer_role(value: ReleaseSignerRoleV1) -> &'static str {
    match value {
        ReleaseSignerRoleV1::ArtifactRelease => "artifact-release",
        ReleaseSignerRoleV1::GateAttestor => "gate-attestor",
        ReleaseSignerRoleV1::RegistryCurator => "registry-curator",
        ReleaseSignerRoleV1::SourceTag => "source-tag",
        ReleaseSignerRoleV1::TrustedTime => "trusted-time",
    }
}

pub(super) fn invalid(reason: impl Into<String>) -> WireError {
    WireError::new("INVALID_RELEASE_WIRE_RECORD", reason)
}
