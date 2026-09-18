//! Signed launch-grant contract: round trip, golden pin, time window, policy
//! key lookup, and binding digest helpers. Field-level and hostile-token
//! refusals live in `negative.rs` and `hostile.rs`.

mod hostile;
mod negative;

use std::{collections::BTreeMap, fs, path::PathBuf, str::FromStr};

use bullet_wire::{
    AttemptId, AuthorityAudience, AuthoritySigningKey, AuthorityVerificationKey, Blake3Digest,
    GateId, GraphRevisionId, KeyAlgorithmV1, KeyPurposeV1, LAUNCH_GRANT_ENVELOPE_DOMAIN,
    LaunchGrantClaims, LaunchGrantExpectation, LaunchOperation, LaunchProvider, MissionId,
    PolicySnapshotV1, ProviderProfileId, RepositoryId, RunnerId, SignedLaunchGrant, VariantId,
    WorkPackageId, WorkspaceId, decode_canonical, decode_canonical_value, environment_digest,
    hash_canonical, hash_framed_bytes, policy_snapshot_digest, workspace_nonce_digest,
};

pub(crate) const SECRET_KEY: [u8; 64] = [
    180, 203, 251, 67, 223, 76, 226, 16, 114, 125, 149, 62, 74, 113, 51, 7, 250, 25, 187, 125, 159,
    133, 4, 20, 56, 217, 225, 27, 148, 42, 55, 116, 30, 185, 219, 187, 188, 4, 124, 3, 253, 112,
    96, 78, 0, 113, 240, 152, 126, 22, 178, 139, 117, 114, 37, 193, 31, 0, 65, 93, 14, 32, 177,
    162,
];
pub(crate) const PUBLIC_KEY: [u8; 32] = [
    30, 185, 219, 187, 188, 4, 124, 3, 253, 112, 96, 78, 0, 113, 240, 152, 126, 22, 178, 139, 117,
    114, 37, 193, 31, 0, 65, 93, 14, 32, 177, 162,
];
pub(crate) const ISSUER: &str = "bullet-kernel-local";
pub(crate) const KEY_ID: &str = "authority-test-1";
pub(crate) const NOT_BEFORE: u64 = 1_800_000_000_000;
pub(crate) const WORKSPACE_NONCE: [u8; 32] = [24; 32];
const GOLDEN_HASH: &str = "5f89dde4a6e9c6d4b19631dd99607418012f1cbe57bfad89e8de4016254df4b0";

pub(crate) fn id<T: FromStr>(prefix: &str, value: char) -> T
where
    T::Err: std::fmt::Debug,
{
    format!("{prefix}{}", value.to_string().repeat(64))
        .parse()
        .unwrap()
}

pub(crate) fn digest(value: u8) -> Blake3Digest {
    Blake3Digest::from_bytes([value; 32])
}

pub(crate) fn environment() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("HOME".to_owned(), "/srv/bullet/runner".to_owned()),
        ("PATH".to_owned(), "/usr/local/bin:/usr/bin:/bin".to_owned()),
        ("TERM".to_owned(), "dumb".to_owned()),
    ])
}

pub(crate) fn claims() -> LaunchGrantClaims {
    LaunchGrantClaims {
        schema_version: "v1alpha1".to_owned(),
        grant_id: digest(1),
        audience: AuthorityAudience::ProviderRunner,
        operation: LaunchOperation::LaunchProvider,
        issuer: ISSUER.to_owned(),
        key_id: KEY_ID.to_owned(),
        issued_at_unix_ms: NOT_BEFORE - 500,
        not_before_unix_ms: NOT_BEFORE,
        expires_at_unix_ms: NOT_BEFORE + 15_000,
        grant_nonce: digest(2),
        mission_id: id::<MissionId>("mis_", '5'),
        repository_id: id::<RepositoryId>("rep_", '4'),
        graph_revision_id: id::<GraphRevisionId>("grf_", '8'),
        work_package_id: id::<WorkPackageId>("wpk_", 'a'),
        variant_id: id::<VariantId>("var_", 'c'),
        attempt_id: id::<AttemptId>("atm_", 'd'),
        attempt_fence: 10,
        runner_id: id::<RunnerId>("run_", 'e'),
        runner_epoch: 11,
        workspace_id: id::<WorkspaceId>("wsp_", 'f'),
        workspace_nonce_digest: workspace_nonce_digest(&WORKSPACE_NONCE).unwrap(),
        authority_epoch: 20,
        freeze_generation: 0,
        provider: LaunchProvider::Claude,
        adapter: "claude-stream-json-v1".to_owned(),
        provider_profile_id: id::<ProviderProfileId>("prf_", '4'),
        model: "claude-test".to_owned(),
        credential_generation: 19,
        protocol: "claude_stream_json".to_owned(),
        executable_path: "/usr/local/bin/claude".to_owned(),
        executable_digest: digest(3),
        descriptor_digest: digest(4),
        capability_digest: digest(5),
        policy_snapshot_digest: digest(6),
        policy_generation: 17,
        sandbox_manifest_digest: digest(7),
        environment_digest: environment_digest(&environment()).unwrap(),
        gate_ids: vec![id::<GateId>("gat_", '8'), id::<GateId>("gat_", '9')],
        budget_reservation_id: digest(8),
        max_invocations: 3,
        max_wall_clock_ms: 900_000,
        max_cost_micro_usd: 2_500_000,
    }
}

pub(crate) fn expectation(claims: &LaunchGrantClaims) -> LaunchGrantExpectation {
    LaunchGrantExpectation {
        audience: claims.audience,
        lease: claims.lease_subject(),
        provider: claims.provider_subject(),
        policy_snapshot_digest: claims.policy_snapshot_digest,
    }
}

pub(crate) fn signer() -> AuthoritySigningKey {
    AuthoritySigningKey::from_bytes(ISSUER, KEY_ID, &SECRET_KEY).unwrap()
}

pub(crate) fn verifier() -> AuthorityVerificationKey {
    AuthorityVerificationKey::from_bytes(ISSUER, KEY_ID, &PUBLIC_KEY).unwrap()
}

pub(crate) fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf()
}

#[test]
fn sign_verify_round_trip_is_deterministic_and_generated_shapes_agree() {
    let claims = claims();
    let grant = signer().sign_launch_grant(&claims).unwrap();
    assert!(grant.paseto.starts_with("v4.public."));
    assert_eq!(grant.issuer, ISSUER);
    assert_eq!(grant.key_id, KEY_ID);
    assert_eq!(
        grant
            .verify(&verifier(), &expectation(&claims), NOT_BEFORE)
            .unwrap(),
        claims
    );
    assert_eq!(signer().sign_launch_grant(&claims).unwrap(), grant);
    assert_eq!(
        grant.digest().unwrap(),
        hash_framed_bytes(LAUNCH_GRANT_ENVELOPE_DOMAIN, grant.paseto.as_bytes()).unwrap()
    );
    assert_ne!(
        grant.digest().unwrap(),
        hash_framed_bytes("authority.envelope.v1alpha1", grant.paseto.as_bytes()).unwrap()
    );
    let generated = serde_json::from_value::<bullet_wire::v1alpha1::LaunchGrantClaimsV1>(
        serde_json::to_value(&claims).unwrap(),
    )
    .unwrap();
    assert_eq!(
        generated.audience,
        bullet_wire::v1alpha1::AuthorityAudienceV1::ProviderRunner
    );
    assert_eq!(generated.operation, "launch-provider");
    assert_eq!(generated.provider, "claude");
    serde_json::from_value::<bullet_wire::v1alpha1::SignedLaunchGrantV1>(
        serde_json::to_value(&grant).unwrap(),
    )
    .unwrap();
}

#[test]
fn not_before_is_inclusive_expires_at_is_exclusive_and_ttl_is_bounded() {
    let claims = claims();
    let grant = signer().sign_launch_grant(&claims).unwrap();
    let expected = expectation(&claims);
    let verifier = verifier();
    assert_eq!(
        grant
            .verify(&verifier, &expected, claims.not_before_unix_ms - 1)
            .unwrap_err()
            .code(),
        "LAUNCH_GRANT_NOT_YET_VALID"
    );
    grant
        .verify(&verifier, &expected, claims.not_before_unix_ms)
        .unwrap();
    grant
        .verify(&verifier, &expected, claims.expires_at_unix_ms - 1)
        .unwrap();
    assert_eq!(
        grant
            .verify(&verifier, &expected, claims.expires_at_unix_ms)
            .unwrap_err()
            .code(),
        "LAUNCH_GRANT_EXPIRED"
    );

    let mut wide = claims.clone();
    wide.expires_at_unix_ms = wide.not_before_unix_ms + 15_001;
    assert_eq!(
        signer().sign_launch_grant(&wide).unwrap_err().code(),
        "LAUNCH_GRANT_TTL_EXCEEDED"
    );
    let mut measured_from_not_before = claims;
    measured_from_not_before.issued_at_unix_ms = NOT_BEFORE - 60_000;
    signer()
        .sign_launch_grant(&measured_from_not_before)
        .unwrap();
}

#[test]
fn committed_launch_grant_golden_is_byte_exact_and_independently_verifiable() {
    let bytes = fs::read(root().join("fixtures/canonical/launch-grant-golden.json")).unwrap();
    let value = decode_canonical_value(&bytes).unwrap();
    assert_eq!(
        hash_canonical("authority.launch-grant-golden.v1alpha1", &value)
            .unwrap()
            .to_string(),
        GOLDEN_HASH
    );
    assert_eq!(bullet_wire::v1alpha1::LAUNCH_GRANT_GOLDEN_HASH, GOLDEN_HASH);
    let manifest = decode_canonical_value(
        &fs::read(root().join("contracts/v1alpha1/bundle-manifest.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["launch_grant_golden_hash"], GOLDEN_HASH);

    let golden_claims = decode_canonical::<LaunchGrantClaims>(
        value["claims_canonical_json"].as_str().unwrap().as_bytes(),
    )
    .unwrap();
    assert_eq!(golden_claims, claims());
    assert_eq!(
        golden_claims.digest().unwrap().to_string(),
        value["claims_digest"].as_str().unwrap()
    );
    let grant = serde_json::from_value::<SignedLaunchGrant>(value["envelope"].clone()).unwrap();
    assert_eq!(
        grant.digest().unwrap().to_string(),
        value["envelope_digest"].as_str().unwrap()
    );
    let public_key = hex::decode(value["public_key_hex"].as_str().unwrap()).unwrap();
    let verifier = AuthorityVerificationKey::from_bytes(ISSUER, KEY_ID, &public_key).unwrap();
    let now = value["verify_at_unix_ms"].as_u64().unwrap();
    assert_eq!(
        grant
            .verify(&verifier, &expectation(&golden_claims), now)
            .unwrap(),
        golden_claims
    );
    assert_eq!(
        value["implicit_assertion_utf8"],
        "bullet-farm.launch-grant.v1alpha1"
    );
    assert_eq!(value["purpose"], "launch-grant-signing");
    assert_eq!(value["audience"], "provider-runner");
    assert_eq!(value["operation"], "launch-provider");
    let footer_segment = grant.paseto.rsplit('.').next().unwrap();
    assert_eq!(
        hostile::b64url_decode(footer_segment),
        value["footer_canonical_json"].as_str().unwrap().as_bytes()
    );
    let nonce = hex::decode(value["workspace_nonce_hex"].as_str().unwrap()).unwrap();
    let nonce: [u8; 32] = nonce.try_into().unwrap();
    assert_eq!(
        workspace_nonce_digest(&nonce).unwrap(),
        golden_claims.workspace_nonce_digest
    );
    let environment =
        serde_json::from_value::<BTreeMap<String, String>>(value["environment"].clone()).unwrap();
    assert_eq!(
        environment_digest(&environment).unwrap(),
        golden_claims.environment_digest
    );
}

#[test]
fn policy_key_lookup_admits_the_provider_runner_audience() {
    let bytes = fs::read(root().join("policy/v1alpha1/policy.json")).unwrap();
    let mut policy = decode_canonical::<PolicySnapshotV1>(&bytes).unwrap();
    assert_eq!(
        policy_snapshot_digest(&bytes).unwrap().to_string(),
        bullet_wire::v1alpha1::POLICY_SNAPSHOT_HASH
    );
    assert!(
        policy
            .issuer_keys
            .iter()
            .all(|key| key.audiences.is_empty())
    );

    let mut grant_key = policy.issuer_keys[0].clone();
    grant_key.issuer = ISSUER.to_owned();
    grant_key.key_id = KEY_ID.to_owned();
    grant_key.key_purpose = KeyPurposeV1::AuthoritySigning;
    grant_key.algorithm = KeyAlgorithmV1::PasetoV4Public;
    grant_key.public_key = hex::encode(PUBLIC_KEY);
    grant_key.audiences = vec![AuthorityAudience::ProviderRunner];
    policy.issuer_keys.push(grant_key);
    policy.validate().unwrap();
    let key = policy
        .authority_key_at(
            ISSUER,
            KEY_ID,
            AuthorityAudience::ProviderRunner,
            policy.activation_at_unix_ms,
        )
        .unwrap();
    let material = hex::decode(&key.public_key).unwrap();
    let verifier =
        AuthorityVerificationKey::from_bytes(&key.issuer, &key.key_id, &material).unwrap();
    let claims = claims();
    let grant = signer().sign_launch_grant(&claims).unwrap();
    grant
        .verify(&verifier, &expectation(&claims), NOT_BEFORE)
        .unwrap();
    assert_eq!(
        policy
            .authority_key_at(
                ISSUER,
                KEY_ID,
                AuthorityAudience::BulletGitd,
                policy.activation_at_unix_ms
            )
            .unwrap_err()
            .code(),
        "AUTHORITY_KEY_AUDIENCE_MISMATCH"
    );
    assert_eq!(
        serde_json::to_value(AuthorityAudience::ProviderRunner).unwrap(),
        serde_json::json!("provider-runner")
    );
}

#[test]
fn binding_digest_helpers_are_domain_separated_and_fail_closed() {
    assert_ne!(
        workspace_nonce_digest(&WORKSPACE_NONCE).unwrap(),
        hash_framed_bytes("authority.envelope.v1alpha1", &WORKSPACE_NONCE).unwrap()
    );
    assert_eq!(
        workspace_nonce_digest(&[0; 32]).unwrap_err().code(),
        "LAUNCH_GRANT_INVALID"
    );
    let mut reordered = environment();
    let path = reordered.remove("PATH").unwrap();
    reordered.insert("PATH".to_owned(), path);
    assert_eq!(
        environment_digest(&reordered).unwrap(),
        environment_digest(&environment()).unwrap()
    );
    let mut changed = environment();
    changed.insert("PATH".to_owned(), "/bin".to_owned());
    assert_ne!(
        environment_digest(&changed).unwrap(),
        environment_digest(&environment()).unwrap()
    );
    let mut hostile = environment();
    hostile.insert("A=B".to_owned(), "x".to_owned());
    assert_eq!(
        environment_digest(&hostile).unwrap_err().code(),
        "LAUNCH_GRANT_INVALID"
    );
    assert_eq!(
        policy_snapshot_digest(b"").unwrap_err().code(),
        "LAUNCH_GRANT_INVALID"
    );
}
