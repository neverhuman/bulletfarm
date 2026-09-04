//! Canonical Candidate-preparation and execution-envelope subjects.
//!
//! These records freeze Kernel-owned Candidate inputs before execution. They
//! grant no integration authority and do not replace the one-use repository
//! mutation permit.

use std::collections::BTreeSet;

use crate::{
    Blake3Digest, WireError, decode_canonical, hash_canonical,
    v1alpha1::{
        CandidatePreparationGrantV1, ExecutionEnvelopeV1, ExecutionToolV1,
        SignedCandidatePreparationGrantV1,
    },
};

pub const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
pub const CANDIDATE_PREPARATION_SIGNING_PURPOSE: &str = "candidate-preparation-grant-signing";
pub const CANDIDATE_PREPARATION_CLAIMS_DOMAIN: &str = "candidate-preparation.grant.v1alpha1";
pub const CANDIDATE_PREPARATION_ENVELOPE_DOMAIN: &str = "candidate-preparation.envelope.v1alpha1";
pub const CANDIDATE_PREPARATION_DIGEST_DOMAIN: &str = "candidate-preparation.grant";
pub const EXECUTION_ENVELOPE_SIGNING_PURPOSE: &str = "execution-envelope-signing";
pub const EXECUTION_ENVELOPE_CLAIMS_DOMAIN: &str = "execution.envelope.v1alpha1";
pub const EXECUTION_ENVELOPE_DIGEST_DOMAIN: &str = "execution.envelope";
pub const EXECUTION_TOOLCHAIN_DIGEST_DOMAIN: &str = "execution.toolchain";

pub fn decode_candidate_preparation_grant(
    bytes: &[u8],
) -> Result<CandidatePreparationGrantV1, WireError> {
    let grant = decode_canonical(bytes)?;
    validate_candidate_preparation_grant(&grant)?;
    Ok(grant)
}

pub fn decode_execution_envelope(bytes: &[u8]) -> Result<ExecutionEnvelopeV1, WireError> {
    let envelope = decode_canonical(bytes)?;
    validate_execution_envelope(&envelope)?;
    Ok(envelope)
}

pub fn decode_signed_candidate_preparation_grant(
    bytes: &[u8],
) -> Result<SignedCandidatePreparationGrantV1, WireError> {
    let signed: SignedCandidatePreparationGrantV1 = decode_canonical(bytes)?;
    require_schema(&signed.schema_version)?;
    require_nonempty("signed grant issuer", &signed.issuer)?;
    require_key_id(&signed.key_id)?;
    if !signed.paseto.starts_with("v4.public.") || signed.paseto.len() > 32_768 {
        return Err(invalid(
            "signed grant is not a bounded PASETO v4.public envelope",
        ));
    }
    Ok(signed)
}

pub fn candidate_preparation_digest(
    grant: &CandidatePreparationGrantV1,
) -> Result<Blake3Digest, WireError> {
    validate_candidate_preparation_grant(grant)?;
    hash_canonical(CANDIDATE_PREPARATION_DIGEST_DOMAIN, grant)
}

pub fn execution_envelope_digest(
    envelope: &ExecutionEnvelopeV1,
) -> Result<Blake3Digest, WireError> {
    validate_execution_envelope(envelope)?;
    hash_canonical(EXECUTION_ENVELOPE_DIGEST_DOMAIN, envelope)
}

pub fn execution_toolchain_digest(tools: &[ExecutionToolV1]) -> Result<Blake3Digest, WireError> {
    validate_tools(tools)?;
    hash_canonical(EXECUTION_TOOLCHAIN_DIGEST_DOMAIN, &tools.to_vec())
}

pub fn validate_candidate_preparation_grant(
    grant: &CandidatePreparationGrantV1,
) -> Result<(), WireError> {
    require_schema(&grant.schema_version)?;
    require_exact(
        "signing purpose",
        &grant.signing_purpose,
        CANDIDATE_PREPARATION_SIGNING_PURPOSE,
    )?;
    require_exact(
        "claims domain",
        &grant.claims_domain,
        CANDIDATE_PREPARATION_CLAIMS_DOMAIN,
    )?;
    require_exact(
        "envelope domain",
        &grant.envelope_domain,
        CANDIDATE_PREPARATION_ENVELOPE_DOMAIN,
    )?;
    require_digest_id(
        "candidate preparation grant id",
        &grant.candidate_preparation_grant_id,
        "cpg",
    )?;
    for (name, value, prefix) in [
        ("repository_id", grant.repository_id.as_str(), "rep"),
        ("mission_id", grant.mission_id.as_str(), "mis"),
        ("plan_revision_id", grant.plan_revision_id.as_str(), "pln"),
        ("work_package_id", grant.work_package_id.as_str(), "wpk"),
        ("variant_id", grant.variant_id.as_str(), "var"),
        ("attempt_id", grant.attempt_id.as_str(), "atm"),
        ("runner_id", grant.runner_id.as_str(), "run"),
        ("workspace_id", grant.workspace_id.as_str(), "wsp"),
        ("change_id", grant.change_id.as_str(), "chg"),
        ("graph_revision_id", grant.graph_revision_id.as_str(), "grf"),
        (
            "context_capsule_id",
            grant.context_capsule_id.as_str(),
            "cnt",
        ),
    ] {
        require_digest_id(name, value, prefix)?;
    }
    for (name, value) in [
        ("request_digest", grant.request_digest.as_str()),
        (
            "authority_token_digest",
            grant.authority_token_digest.as_str(),
        ),
        ("grant_nonce", grant.grant_nonce.as_str()),
        ("scope_grant_digest", grant.scope_grant_digest.as_str()),
        ("environment_digest", grant.environment_digest.as_str()),
        ("toolchain_digest", grant.toolchain_digest.as_str()),
    ] {
        require_digest(name, value)?;
    }
    require_nonempty("grant issuer", &grant.issuer)?;
    require_key_id(&grant.key_id)?;
    require_digest_id("execution envelope id", &grant.execution_envelope_id, "exe")?;
    for (name, value) in [
        ("attempt_fence", grant.attempt_fence),
        ("runner_epoch", grant.runner_epoch),
        ("scope_revision", grant.scope_revision),
        ("context_revision", grant.context_revision),
    ] {
        require_positive_safe(name, value)?;
    }
    for (name, value) in [
        ("authority_epoch", grant.authority_epoch),
        ("freeze_generation", grant.freeze_generation),
        ("issued_at_unix_ms", grant.issued_at_unix_ms),
        ("not_before_unix_ms", grant.not_before_unix_ms),
        ("expires_at_unix_ms", grant.expires_at_unix_ms),
    ] {
        require_safe(name, value)?;
    }
    require_time_window(
        grant.issued_at_unix_ms,
        grant.not_before_unix_ms,
        grant.expires_at_unix_ms,
    )?;
    let mut parents = BTreeSet::new();
    if grant.parent_candidate_ids.len() > 128
        || grant.parent_candidate_ids.iter().any(|parent| {
            require_digest_id("parent_candidate_id", parent, "can").is_err()
                || !parents.insert(parent)
        })
    {
        return Err(invalid(
            "parent Candidate list must be ordered, unique, and bounded",
        ));
    }
    Ok(())
}

pub fn validate_execution_envelope(envelope: &ExecutionEnvelopeV1) -> Result<(), WireError> {
    require_schema(&envelope.schema_version)?;
    require_exact(
        "execution signing purpose",
        &envelope.signing_purpose,
        EXECUTION_ENVELOPE_SIGNING_PURPOSE,
    )?;
    require_exact(
        "execution claims domain",
        &envelope.claims_domain,
        EXECUTION_ENVELOPE_CLAIMS_DOMAIN,
    )?;
    require_digest_id(
        "execution envelope id",
        &envelope.execution_envelope_id,
        "exe",
    )?;
    require_digest_id("runner_id", &envelope.runner_id, "run")?;
    require_digest_id("provider_profile_id", &envelope.provider_profile_id, "prf")?;
    require_digest_id(
        "containment_profile_id",
        &envelope.containment_profile_id,
        "ctp",
    )?;
    for (name, value) in [
        ("environment_digest", envelope.environment_digest.as_str()),
        ("toolchain_digest", envelope.toolchain_digest.as_str()),
        (
            "sandbox_image_digest",
            envelope.sandbox_image_digest.as_str(),
        ),
    ] {
        require_digest(name, value)?;
    }
    for (name, value) in [
        ("execution issuer", envelope.issuer.as_str()),
        ("provider", envelope.provider.as_str()),
        ("model", envelope.model.as_str()),
        ("adapter", envelope.adapter.as_str()),
        ("platform", envelope.platform.as_str()),
    ] {
        require_nonempty(name, value)?;
    }
    require_key_id(&envelope.key_id)?;
    require_positive_safe("runner_epoch", envelope.runner_epoch)?;
    require_safe("authority_epoch", envelope.authority_epoch)?;
    require_safe("freeze_generation", envelope.freeze_generation)?;
    require_safe("issued_at_unix_ms", envelope.issued_at_unix_ms)?;
    require_safe("expires_at_unix_ms", envelope.expires_at_unix_ms)?;
    if envelope.expires_at_unix_ms <= envelope.issued_at_unix_ms {
        return Err(invalid("execution envelope time window is empty"));
    }
    validate_tools(&envelope.tools)?;
    let expected = execution_toolchain_digest(&envelope.tools)?.to_string();
    if envelope.toolchain_digest != expected {
        return Err(invalid(
            "toolchain digest does not bind the ordered tool manifest",
        ));
    }
    Ok(())
}

pub fn validate_candidate_preparation_binding(
    grant: &CandidatePreparationGrantV1,
    envelope: &ExecutionEnvelopeV1,
) -> Result<(), WireError> {
    validate_candidate_preparation_grant(grant)?;
    validate_execution_envelope(envelope)?;
    if grant.execution_envelope_id != envelope.execution_envelope_id
        || grant.runner_id != envelope.runner_id
        || grant.runner_epoch != envelope.runner_epoch
        || grant.environment_digest != envelope.environment_digest
        || grant.toolchain_digest != envelope.toolchain_digest
        || grant.authority_epoch != envelope.authority_epoch
        || grant.freeze_generation != envelope.freeze_generation
        || grant.issued_at_unix_ms < envelope.issued_at_unix_ms
        || grant.expires_at_unix_ms > envelope.expires_at_unix_ms
    {
        return Err(invalid(
            "Candidate grant does not bind the exact execution envelope",
        ));
    }
    Ok(())
}

fn validate_tools(tools: &[ExecutionToolV1]) -> Result<(), WireError> {
    if tools.is_empty() || tools.len() > 64 {
        return Err(invalid("execution tool manifest must contain 1..=64 tools"));
    }
    let mut ids = BTreeSet::new();
    let mut roles = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for tool in tools {
        require_schema(&tool.schema_version)?;
        require_digest_id("execution tool id", &tool.tool_id, "etl")?;
        require_digest("tool executable digest", &tool.executable_digest)?;
        require_digest("tool descriptor digest", &tool.descriptor_digest)?;
        require_nonempty("tool role", &tool.role)?;
        require_nonempty("tool version", &tool.version)?;
        if !ids.insert(tool.tool_id.as_str())
            || !roles.insert(tool.role.as_str())
            || !paths.insert(tool.executable_path.as_str())
            || !tool.executable_path.starts_with('/')
            || tool.executable_path.contains('\0')
        {
            return Err(invalid(
                "execution tools require unique ids, roles, and absolute non-NUL paths",
            ));
        }
    }
    Ok(())
}

fn require_schema(value: &str) -> Result<(), WireError> {
    require_exact("schema version", value, "v1alpha1")
}

fn require_exact(name: &str, value: &str, expected: &str) -> Result<(), WireError> {
    if value == expected {
        Ok(())
    } else {
        Err(invalid(&format!("{name} is not the frozen value")))
    }
}

fn require_digest_id(name: &str, value: &str, prefix: &str) -> Result<(), WireError> {
    let Some(hex) = value.strip_prefix(&format!("{prefix}_")) else {
        return Err(invalid(&format!("{name} has the wrong prefix")));
    };
    if hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(invalid(&format!("{name} is not full-width lowercase hex")))
    }
}

fn require_digest(name: &str, value: &str) -> Result<(), WireError> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(invalid(&format!("{name} is not a lowercase BLAKE3 digest")))
    }
}

fn require_nonempty(name: &str, value: &str) -> Result<(), WireError> {
    if value.is_empty() || value.contains('\0') {
        Err(invalid(&format!("{name} is empty or contains NUL")))
    } else {
        Ok(())
    }
}

fn require_key_id(value: &str) -> Result<(), WireError> {
    if !value.is_empty()
        && value.len() <= 128
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric()
                || (index > 0 && matches!(byte, b'.' | b'_' | b':' | b'/' | b'-'))
        })
    {
        Ok(())
    } else {
        Err(invalid("key_id is outside the frozen character set"))
    }
}

fn require_safe(name: &str, value: u64) -> Result<(), WireError> {
    if value <= MAX_SAFE_INTEGER {
        Ok(())
    } else {
        Err(invalid(&format!("{name} exceeds the safe-integer ceiling")))
    }
}

fn require_positive_safe(name: &str, value: u64) -> Result<(), WireError> {
    require_safe(name, value)?;
    if value == 0 {
        Err(invalid(&format!("{name} must be positive")))
    } else {
        Ok(())
    }
}

fn require_time_window(issued: u64, not_before: u64, expires: u64) -> Result<(), WireError> {
    if issued <= not_before && not_before < expires {
        Ok(())
    } else {
        Err(invalid(
            "Candidate preparation grant time window is invalid",
        ))
    }
}

fn invalid(detail: &str) -> WireError {
    WireError::new("INVALID_CANDIDATE_PREPARATION_GRANT", detail)
}
