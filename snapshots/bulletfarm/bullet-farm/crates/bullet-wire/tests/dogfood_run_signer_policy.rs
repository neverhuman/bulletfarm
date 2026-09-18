//! Purpose separation for terminal dogfood-run attestation keys. This suite
//! proves current-policy shape and lookup only; it creates no signed run.

use std::{fs, path::PathBuf};

use bullet_wire::{
    AuthorityAudience, Blake3Digest, DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE, DogfoodBindingV1,
    KeyAlgorithmV1, KeyPurposeV1, PolicySnapshotV1, PrincipalId, decode_canonical,
};

const FIXTURE: &str = "crates/bullet-wire/tests/fixtures/policy-v1alpha2-live-enabled.json";
const KEY_ID: &str = "dogfood-run-attestor-1";
const ATTESTOR_PUBLIC_KEY: &str =
    "5555555555555555555555555555555555555555555555555555555555555555";
const LAUNCH_PUBLIC_KEY: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const ENROLLMENT_PUBLIC_KEY: &str =
    "4444444444444444444444444444444444444444444444444444444444444444";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf()
}

fn fixture() -> PolicySnapshotV1 {
    decode_canonical(&fs::read(root().join(FIXTURE)).unwrap()).unwrap()
}

fn principal(seed: u8) -> PrincipalId {
    PrincipalId::from_digest(Blake3Digest::from_bytes([seed; 32]))
}

fn index(policy: &PolicySnapshotV1, purpose: KeyPurposeV1) -> usize {
    policy
        .issuer_keys
        .iter()
        .position(|key| key.key_purpose == purpose)
        .unwrap()
}

fn attestor_policy() -> PolicySnapshotV1 {
    let mut policy = fixture();
    policy.sandbox_policy.live_admission_enabled = false;
    let authority = policy.issuer_keys[index(&policy, KeyPurposeV1::AuthoritySigning)].clone();

    let mut launch = authority.clone();
    launch.issuer = "dogfood-operator".into();
    launch.key_id = "dogfood-launch-1".into();
    launch.key_purpose = KeyPurposeV1::DogfoodLaunchSigning;
    launch.public_key = LAUNCH_PUBLIC_KEY.into();
    launch.audiences.clear();

    let mut enrollment = authority.clone();
    enrollment.issuer = "provider-enrollment-operator".into();
    enrollment.key_id = "provider-enrollment-1".into();
    enrollment.key_purpose = KeyPurposeV1::ProviderEnrollmentSigning;
    enrollment.public_key = ENROLLMENT_PUBLIC_KEY.into();
    enrollment.audiences.clear();

    let mut attestor = authority;
    attestor.issuer = principal(9).to_string();
    attestor.key_id = KEY_ID.into();
    attestor.key_purpose = KeyPurposeV1::DogfoodRunAttestationSigning;
    attestor.public_key = ATTESTOR_PUBLIC_KEY.into();
    attestor.audiences.clear();

    policy.issuer_keys.extend([launch, enrollment, attestor]);
    policy
}

fn policy_code(policy: &PolicySnapshotV1) -> &'static str {
    policy.validate().unwrap_err().code()
}

fn lookup_code(
    policy: &PolicySnapshotV1,
    attestor: &PrincipalId,
    key_id: &str,
    now_unix_ms: u64,
) -> &'static str {
    policy
        .dogfood_run_attestor_key_at(attestor, key_id, now_unix_ms)
        .unwrap_err()
        .code()
}

#[test]
fn wire_value_is_exact_closed_and_not_an_authority_audience() {
    assert_eq!(
        serde_json::to_string(&KeyPurposeV1::DogfoodRunAttestationSigning).unwrap(),
        format!(r#""{DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE}""#)
    );
    assert_eq!(
        DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE,
        "dogfood-run-attestation-signing"
    );
    for alias in [
        br#""dogfood-run-signing""#.as_slice(),
        br#""dogfood-run-attestor-signing""#.as_slice(),
        br#""DOGFOOD-RUN-ATTESTATION-SIGNING""#.as_slice(),
    ] {
        assert!(decode_canonical::<KeyPurposeV1>(alias).is_err());
    }
    assert!(
        decode_canonical::<AuthorityAudience>(br#""dogfood-run-attestation-signing""#,).is_err()
    );
}

#[test]
fn exact_attestor_resolves_and_every_other_purpose_refuses_cross_use() {
    let policy = attestor_policy();
    policy.validate().unwrap();
    let attestor = principal(9);
    let now = policy.activation_at_unix_ms;
    let key = policy
        .dogfood_run_attestor_key_at(&attestor, KEY_ID, now)
        .unwrap();
    assert_eq!(key.issuer, attestor.as_str());
    assert_eq!(key.public_key, ATTESTOR_PUBLIC_KEY);
    assert!(key.audiences.is_empty());

    for (seed, purpose) in [
        (20, KeyPurposeV1::AuthoritySigning),
        (21, KeyPurposeV1::DogfoodLaunchSigning),
        (22, KeyPurposeV1::ProviderEnrollmentSigning),
        (23, KeyPurposeV1::ReleaseSigning),
    ] {
        let mut wrong = policy.clone();
        let wrong_principal = principal(seed);
        let wrong_index = index(&wrong, purpose);
        wrong.issuer_keys[wrong_index].issuer = wrong_principal.to_string();
        assert_eq!(
            lookup_code(
                &wrong,
                &wrong_principal,
                &wrong.issuer_keys[wrong_index].key_id,
                now,
            ),
            "DOGFOOD_RUN_ATTESTOR_KEY_WRONG_PURPOSE"
        );
    }

    assert_eq!(
        policy
            .authority_key_at(
                attestor.as_str(),
                KEY_ID,
                AuthorityAudience::ProviderRunner,
                now,
            )
            .unwrap_err()
            .code(),
        "AUTHORITY_KEY_WRONG_PURPOSE"
    );
    assert_eq!(
        policy
            .dogfood_signer_key_at(
                attestor.as_str(),
                KEY_ID,
                &DogfoodBindingV1::read_only_propose(),
                now,
            )
            .unwrap_err()
            .code(),
        "DOGFOOD_SIGNER_KEY_WRONG_PURPOSE"
    );
    assert_eq!(
        policy
            .provider_enrollment_signer_key_at(attestor.as_str(), KEY_ID, now)
            .unwrap_err()
            .code(),
        "PROVIDER_ENROLLMENT_SIGNER_KEY_WRONG_PURPOSE"
    );

    let mut no_runner = policy;
    let authority = index(&no_runner, KeyPurposeV1::AuthoritySigning);
    no_runner.issuer_keys.remove(authority);
    no_runner.sandbox_policy.live_admission_enabled = true;
    assert_eq!(
        no_runner.validate().unwrap_err().code(),
        "LIVE_ADMISSION_REQUIRES_RUNNER_KEY"
    );
}

#[test]
fn policy_key_and_revocation_windows_are_half_open_with_structural_precedence() {
    let policy = attestor_policy();
    let attestor = principal(9);
    let activation = policy.activation_at_unix_ms;
    let expiry = policy.expires_at_unix_ms;

    assert_eq!(
        lookup_code(&policy, &principal(8), KEY_ID, activation),
        "DOGFOOD_RUN_ATTESTOR_KEY_UNKNOWN"
    );
    assert_eq!(
        lookup_code(&policy, &attestor, "unknown", activation),
        "DOGFOOD_RUN_ATTESTOR_KEY_UNKNOWN"
    );
    assert_eq!(
        lookup_code(&policy, &attestor, KEY_ID, activation - 1),
        "POLICY_NOT_ACTIVE"
    );
    policy
        .dogfood_run_attestor_key_at(&attestor, KEY_ID, activation)
        .unwrap();
    policy
        .dogfood_run_attestor_key_at(&attestor, KEY_ID, expiry - 1)
        .unwrap();
    assert_eq!(
        lookup_code(&policy, &attestor, KEY_ID, expiry),
        "POLICY_NOT_ACTIVE"
    );

    let mut delayed = policy.clone();
    let key = index(&delayed, KeyPurposeV1::DogfoodRunAttestationSigning);
    delayed.issuer_keys[key].activates_at_unix_ms = activation + 100;
    delayed.issuer_keys[key].expires_at_unix_ms = activation + 200;
    delayed.issuer_keys[key].retain_until_unix_ms = activation + 15_200;
    assert_eq!(
        lookup_code(&delayed, &attestor, KEY_ID, activation + 99),
        "DOGFOOD_RUN_ATTESTOR_KEY_INACTIVE"
    );
    delayed
        .dogfood_run_attestor_key_at(&attestor, KEY_ID, activation + 100)
        .unwrap();
    delayed
        .dogfood_run_attestor_key_at(&attestor, KEY_ID, activation + 199)
        .unwrap();
    assert_eq!(
        lookup_code(&delayed, &attestor, KEY_ID, activation + 200),
        "DOGFOOD_RUN_ATTESTOR_KEY_INACTIVE"
    );

    let mut revoked = policy.clone();
    let key = index(&revoked, KeyPurposeV1::DogfoodRunAttestationSigning);
    revoked.issuer_keys[key].revoked_at_unix_ms = Some(activation + 100);
    revoked
        .dogfood_run_attestor_key_at(&attestor, KEY_ID, activation + 99)
        .unwrap();
    assert_eq!(
        lookup_code(&revoked, &attestor, KEY_ID, activation + 100),
        "DOGFOOD_RUN_ATTESTOR_KEY_INACTIVE"
    );

    let mut nonoverlap = policy.clone();
    let key = index(&nonoverlap, KeyPurposeV1::DogfoodRunAttestationSigning);
    nonoverlap.issuer_keys[key].activates_at_unix_ms = expiry + 100;
    nonoverlap.issuer_keys[key].expires_at_unix_ms = expiry + 200;
    nonoverlap.issuer_keys[key].retain_until_unix_ms = expiry + 15_200;
    assert_eq!(
        lookup_code(&nonoverlap, &attestor, KEY_ID, activation),
        "DOGFOOD_RUN_ATTESTOR_KEY_INACTIVE"
    );

    let mut malformed_key = policy.clone();
    let key = index(&malformed_key, KeyPurposeV1::DogfoodRunAttestationSigning);
    malformed_key.issuer_keys[key].public_key = "AA".repeat(32);
    assert_eq!(
        lookup_code(&malformed_key, &principal(8), "unknown", activation - 1),
        "INVALID_DOGFOOD_RUN_ATTESTOR_PUBLIC_KEY"
    );
    let mut malformed_policy = policy;
    malformed_policy.expires_at_unix_ms = malformed_policy.activation_at_unix_ms;
    assert_eq!(
        lookup_code(&malformed_policy, &principal(8), "unknown", activation - 1,),
        "INVALID_POLICY_WINDOW"
    );
}

#[test]
fn key_shape_lifecycle_and_current_policy_material_isolation_refuse() {
    let policy = attestor_policy();
    let attestor = index(&policy, KeyPurposeV1::DogfoodRunAttestationSigning);

    let mut audience = policy.clone();
    audience.issuer_keys[attestor].audiences = vec![AuthorityAudience::ProviderRunner];
    assert_eq!(
        policy_code(&audience),
        "INVALID_DOGFOOD_RUN_ATTESTOR_PUBLIC_KEY"
    );
    let mut algorithm = policy.clone();
    algorithm.issuer_keys[attestor].algorithm = KeyAlgorithmV1::SshEd25519;
    assert_eq!(
        policy_code(&algorithm),
        "INVALID_DOGFOOD_RUN_ATTESTOR_PUBLIC_KEY"
    );
    for invalid in [
        "AA".repeat(32),
        "gg".repeat(32),
        "11".repeat(31),
        "00".repeat(32),
    ] {
        let mut malformed = policy.clone();
        malformed.issuer_keys[attestor].public_key = invalid;
        assert_eq!(
            policy_code(&malformed),
            "INVALID_DOGFOOD_RUN_ATTESTOR_PUBLIC_KEY"
        );
    }
    let mut invalid_label = policy.clone();
    invalid_label.issuer_keys[attestor].issuer = "!".into();
    assert_eq!(
        policy_code(&invalid_label),
        "INVALID_DOGFOOD_RUN_ATTESTOR_PUBLIC_KEY"
    );
    let mut invalid_key_label = policy.clone();
    invalid_key_label.issuer_keys[attestor].key_id = "!".into();
    assert_eq!(
        policy_code(&invalid_key_label),
        "INVALID_DOGFOOD_RUN_ATTESTOR_PUBLIC_KEY"
    );

    let mut duplicate_identity = policy.clone();
    duplicate_identity
        .issuer_keys
        .push(duplicate_identity.issuer_keys[attestor].clone());
    assert_eq!(
        policy_code(&duplicate_identity),
        "INVALID_ISSUER_KEY_LIFECYCLE"
    );
    let mut lifecycle = policy.clone();
    lifecycle.issuer_keys[attestor].expires_at_unix_ms =
        lifecycle.issuer_keys[attestor].activates_at_unix_ms;
    assert_eq!(policy_code(&lifecycle), "INVALID_ISSUER_KEY_LIFECYCLE");
    let mut retention = policy.clone();
    retention.issuer_keys[attestor].retain_until_unix_ms =
        retention.issuer_keys[attestor].expires_at_unix_ms + 14_999;
    assert_eq!(policy_code(&retention), "INVALID_ISSUER_KEY_LIFECYCLE");

    let mut alias = policy.clone();
    let mut duplicate = alias.issuer_keys[attestor].clone();
    duplicate.issuer = principal(30).to_string();
    duplicate.key_id = "dogfood-run-attestor-alias".into();
    alias.issuer_keys.push(duplicate);
    assert_eq!(policy_code(&alias), "SIGNER_KEY_MATERIAL_REUSED");

    for purpose in [
        KeyPurposeV1::AuthoritySigning,
        KeyPurposeV1::DogfoodLaunchSigning,
        KeyPurposeV1::ProviderEnrollmentSigning,
    ] {
        let mut reused = policy.clone();
        let other = index(&reused, purpose);
        reused.issuer_keys[attestor].public_key = reused.issuer_keys[other].public_key.clone();
        assert_eq!(policy_code(&reused), "SIGNER_KEY_MATERIAL_REUSED");
    }
}
