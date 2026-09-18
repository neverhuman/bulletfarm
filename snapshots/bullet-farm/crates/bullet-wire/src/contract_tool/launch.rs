use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::authority::{PUBLIC_KEY_HEX, SECRET_KEY, parse_id};
use crate::{
    AuthorityAudience, AuthoritySigningKey, Blake3Digest, LAUNCH_GRANT_CLAIMS_DOMAIN,
    LAUNCH_GRANT_ENVELOPE_DOMAIN, LAUNCH_GRANT_ENVIRONMENT_DOMAIN, LAUNCH_GRANT_IMPLICIT_ASSERTION,
    LAUNCH_GRANT_SIGNING_PURPOSE, LAUNCH_GRANT_WORKSPACE_NONCE_DOMAIN, LaunchGrantClaims,
    LaunchOperation, LaunchProvider, WireError, canonical_json, environment_digest, hash_canonical,
    workspace_nonce_digest,
};

const WORKSPACE_NONCE: [u8; 32] = [24; 32];

/// Deterministic launch-grant golden signed with the same fixture-only key as
/// the authority golden. `issued_at` precedes `not_before` by 500 ms and the
/// window is exactly the 15 s maximum measured from `not_before`.
pub(super) fn launch_grant_golden() -> Result<(Value, Blake3Digest), WireError> {
    let environment = golden_environment();
    let claims = LaunchGrantClaims {
        schema_version: "v1alpha1".to_owned(),
        grant_id: Blake3Digest::from_bytes([1; 32]),
        audience: AuthorityAudience::ProviderRunner,
        operation: LaunchOperation::LaunchProvider,
        issuer: "bullet-kernel-local".to_owned(),
        key_id: "authority-test-1".to_owned(),
        issued_at_unix_ms: 1_799_999_999_500,
        not_before_unix_ms: 1_800_000_000_000,
        expires_at_unix_ms: 1_800_000_015_000,
        grant_nonce: Blake3Digest::from_bytes([2; 32]),
        mission_id: parse_id("mis_", '5')?,
        repository_id: parse_id("rep_", '4')?,
        graph_revision_id: parse_id("grf_", '8')?,
        work_package_id: parse_id("wpk_", 'a')?,
        variant_id: parse_id("var_", 'c')?,
        attempt_id: parse_id("atm_", 'd')?,
        attempt_fence: 10,
        runner_id: parse_id("run_", 'e')?,
        runner_epoch: 11,
        workspace_id: parse_id("wsp_", 'f')?,
        workspace_nonce_digest: workspace_nonce_digest(&WORKSPACE_NONCE)?,
        authority_epoch: 20,
        freeze_generation: 0,
        provider: LaunchProvider::Claude,
        adapter: "claude-stream-json-v1".to_owned(),
        provider_profile_id: parse_id("prf_", '4')?,
        model: "claude-test".to_owned(),
        credential_generation: 19,
        protocol: "claude_stream_json".to_owned(),
        executable_path: "/usr/local/bin/claude".to_owned(),
        executable_digest: Blake3Digest::from_bytes([3; 32]),
        descriptor_digest: Blake3Digest::from_bytes([4; 32]),
        capability_digest: Blake3Digest::from_bytes([5; 32]),
        policy_snapshot_digest: Blake3Digest::from_bytes([6; 32]),
        policy_generation: 17,
        sandbox_manifest_digest: Blake3Digest::from_bytes([7; 32]),
        environment_digest: environment_digest(&environment)?,
        gate_ids: vec![parse_id("gat_", '8')?, parse_id("gat_", '9')?],
        budget_reservation_id: Blake3Digest::from_bytes([8; 32]),
        max_invocations: 3,
        max_wall_clock_ms: 900_000,
        max_cost_micro_usd: 2_500_000,
    };
    let signer =
        AuthoritySigningKey::from_bytes("bullet-kernel-local", "authority-test-1", &SECRET_KEY)?;
    let grant = signer.sign_launch_grant(&claims)?;
    let envelope_digest = grant.digest()?;
    let footer = canonical_json(&json!({
        "issuer": "bullet-kernel-local",
        "key_id": "authority-test-1",
        "purpose": LAUNCH_GRANT_SIGNING_PURPOSE,
        "schema_version": "v1alpha1",
    }))?;
    let implicit_assertion = std::str::from_utf8(LAUNCH_GRANT_IMPLICIT_ASSERTION)
        .map_err(|error| WireError::new("GOLDEN_ENCODING_FAILED", error.to_string()))?;
    let value = json!({
        "audience": claims.audience,
        "claims_canonical_json": utf8(canonical_json(&claims)?)?,
        "claims_digest": claims.digest()?,
        "claims_domain": LAUNCH_GRANT_CLAIMS_DOMAIN,
        "envelope": grant,
        "envelope_digest": envelope_digest,
        "envelope_domain": LAUNCH_GRANT_ENVELOPE_DOMAIN,
        "environment": environment,
        "environment_domain": LAUNCH_GRANT_ENVIRONMENT_DOMAIN,
        "footer_canonical_json": utf8(footer)?,
        "implicit_assertion_utf8": implicit_assertion,
        "operation": claims.operation,
        "public_key_hex": PUBLIC_KEY_HEX,
        "purpose": LAUNCH_GRANT_SIGNING_PURPOSE,
        "schema_version": "v1alpha1",
        "verify_at_unix_ms": claims.not_before_unix_ms,
        "workspace_nonce_domain": LAUNCH_GRANT_WORKSPACE_NONCE_DOMAIN,
        "workspace_nonce_hex": lower_hex(&WORKSPACE_NONCE),
    });
    let digest = hash_canonical("authority.launch-grant-golden.v1alpha1", &value)?;
    Ok((value, digest))
}

fn golden_environment() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("HOME".to_owned(), "/srv/bullet/runner".to_owned()),
        ("PATH".to_owned(), "/usr/local/bin:/usr/bin:/bin".to_owned()),
        ("TERM".to_owned(), "dumb".to_owned()),
    ])
}

fn utf8(bytes: Vec<u8>) -> Result<String, WireError> {
    String::from_utf8(bytes)
        .map_err(|error| WireError::new("GOLDEN_ENCODING_FAILED", error.to_string()))
}

fn lower_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
