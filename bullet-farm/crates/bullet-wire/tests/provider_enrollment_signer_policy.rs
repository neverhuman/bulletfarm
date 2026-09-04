//! Purpose separation for provider-enrollment verification keys. These tests
//! prove policy shape and lookup only; no signed enrollment is created.

use std::{fs, path::PathBuf};

use bullet_wire::{
    AuthorityAudience, KeyAlgorithmV1, KeyPurposeV1, PolicySnapshotV1, decode_canonical,
};

const FIXTURE: &str = "crates/bullet-wire/tests/fixtures/policy-v1alpha2-live-enabled.json";
const ISSUER: &str = "provider-enrollment-operator";
const KEY_ID: &str = "provider-enrollment-1";
const PUBLIC_KEY: &str = "4444444444444444444444444444444444444444444444444444444444444444";
const DOGFOOD_PUBLIC_KEY: &str = "5555555555555555555555555555555555555555555555555555555555555555";

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

fn authority_index(policy: &PolicySnapshotV1) -> usize {
    policy
        .issuer_keys
        .iter()
        .position(|key| key.key_purpose == KeyPurposeV1::AuthoritySigning)
        .unwrap()
}

fn release_index(policy: &PolicySnapshotV1) -> usize {
    policy
        .issuer_keys
        .iter()
        .position(|key| key.key_purpose == KeyPurposeV1::ReleaseSigning)
        .unwrap()
}

fn enrollment_index(policy: &PolicySnapshotV1) -> usize {
    policy
        .issuer_keys
        .iter()
        .position(|key| key.key_purpose == KeyPurposeV1::ProviderEnrollmentSigning)
        .unwrap()
}

fn dogfood_index(policy: &PolicySnapshotV1) -> usize {
    policy
        .issuer_keys
        .iter()
        .position(|key| key.key_purpose == KeyPurposeV1::DogfoodLaunchSigning)
        .unwrap()
}

fn enrollment_policy() -> PolicySnapshotV1 {
    let mut policy = fixture();
    let mut key = policy.issuer_keys[authority_index(&policy)].clone();
    key.issuer = ISSUER.into();
    key.key_id = KEY_ID.into();
    key.key_purpose = KeyPurposeV1::ProviderEnrollmentSigning;
    key.public_key = PUBLIC_KEY.into();
    key.audiences.clear();
    let mut dogfood = key.clone();
    dogfood.issuer = "dogfood-operator".into();
    dogfood.key_id = "dogfood-launch-1".into();
    dogfood.key_purpose = KeyPurposeV1::DogfoodLaunchSigning;
    dogfood.public_key = DOGFOOD_PUBLIC_KEY.into();
    policy.issuer_keys.push(key);
    policy.issuer_keys.push(dogfood);
    policy
}

fn policy_code(policy: &PolicySnapshotV1) -> &'static str {
    policy.validate().unwrap_err().code()
}

fn lookup_code(
    policy: &PolicySnapshotV1,
    issuer: &str,
    key_id: &str,
    now_unix_ms: u64,
) -> &'static str {
    policy
        .provider_enrollment_signer_key_at(issuer, key_id, now_unix_ms)
        .unwrap_err()
        .code()
}

#[test]
fn wire_value_is_exact_and_never_an_authority_audience() {
    assert_eq!(
        serde_json::to_string(&KeyPurposeV1::ProviderEnrollmentSigning).unwrap(),
        r#""provider-enrollment-signing""#
    );
    assert!(decode_canonical::<KeyPurposeV1>(br#""provider-signing""#).is_err());
    assert!(decode_canonical::<KeyPurposeV1>(br#""PROVIDER-ENROLLMENT-SIGNING""#).is_err());
    assert!(decode_canonical::<AuthorityAudience>(br#""provider-enrollment-signing""#).is_err());
}

#[test]
fn dedicated_key_resolves_and_other_key_purposes_refuse() {
    let policy = enrollment_policy();
    policy.validate().unwrap();
    let now = policy.activation_at_unix_ms;
    let key = policy
        .provider_enrollment_signer_key_at(ISSUER, KEY_ID, now)
        .unwrap();
    assert_eq!(key.public_key, PUBLIC_KEY);
    assert!(key.audiences.is_empty());
    assert_eq!(
        policy
            .authority_key_at(ISSUER, KEY_ID, AuthorityAudience::ProviderRunner, now)
            .unwrap_err()
            .code(),
        "AUTHORITY_KEY_WRONG_PURPOSE"
    );

    for index in [
        authority_index(&policy),
        dogfood_index(&policy),
        release_index(&policy),
    ] {
        let key = &policy.issuer_keys[index];
        assert_eq!(
            lookup_code(&policy, &key.issuer, &key.key_id, now),
            "PROVIDER_ENROLLMENT_SIGNER_KEY_WRONG_PURPOSE"
        );
    }
}

#[test]
fn identity_policy_window_and_key_lifecycle_are_exact() {
    let policy = enrollment_policy();
    let now = policy.activation_at_unix_ms;
    assert_eq!(
        lookup_code(&policy, "unknown", KEY_ID, now),
        "PROVIDER_ENROLLMENT_SIGNER_KEY_UNKNOWN"
    );
    assert_eq!(
        lookup_code(&policy, ISSUER, "unknown", now),
        "PROVIDER_ENROLLMENT_SIGNER_KEY_UNKNOWN"
    );
    assert_eq!(
        lookup_code(
            &policy,
            ISSUER,
            KEY_ID,
            policy.activation_at_unix_ms.saturating_sub(1),
        ),
        "POLICY_NOT_ACTIVE"
    );

    let mut revoked = policy.clone();
    let index = enrollment_index(&revoked);
    revoked.issuer_keys[index].revoked_at_unix_ms = Some(now);
    assert_eq!(
        lookup_code(&revoked, ISSUER, KEY_ID, now),
        "PROVIDER_ENROLLMENT_SIGNER_KEY_INACTIVE"
    );

    let mut expired = policy;
    let index = enrollment_index(&expired);
    expired.issuer_keys[index].expires_at_unix_ms -= 1;
    let expiry = expired.issuer_keys[index].expires_at_unix_ms;
    assert_eq!(
        lookup_code(&expired, ISSUER, KEY_ID, expiry),
        "PROVIDER_ENROLLMENT_SIGNER_KEY_INACTIVE"
    );
}

#[test]
fn key_shape_identity_and_cross_purpose_material_reuse_refuse() {
    let policy = enrollment_policy();
    let index = enrollment_index(&policy);

    let mut audience = policy.clone();
    audience.issuer_keys[index].audiences = vec![AuthorityAudience::ProviderRunner];
    assert_eq!(
        policy_code(&audience),
        "INVALID_PROVIDER_ENROLLMENT_PUBLIC_KEY"
    );

    let mut algorithm = policy.clone();
    algorithm.issuer_keys[index].algorithm = KeyAlgorithmV1::SshEd25519;
    assert_eq!(
        policy_code(&algorithm),
        "INVALID_PROVIDER_ENROLLMENT_PUBLIC_KEY"
    );

    let mut encoding = policy.clone();
    encoding.issuer_keys[index].public_key = "AA".repeat(32);
    assert_eq!(
        policy_code(&encoding),
        "INVALID_PROVIDER_ENROLLMENT_PUBLIC_KEY"
    );

    let mut duplicate_identity = policy.clone();
    duplicate_identity
        .issuer_keys
        .push(duplicate_identity.issuer_keys[index].clone());
    assert_eq!(
        policy_code(&duplicate_identity),
        "INVALID_ISSUER_KEY_LIFECYCLE"
    );

    let mut duplicate_material = policy.clone();
    let mut alias = duplicate_material.issuer_keys[index].clone();
    alias.issuer = "provider-enrollment-alias".into();
    alias.key_id = "provider-enrollment-alias-1".into();
    duplicate_material.issuer_keys.push(alias);
    assert_eq!(
        policy_code(&duplicate_material),
        "SIGNER_KEY_MATERIAL_REUSED"
    );

    let mut cross_purpose = policy;
    let authority = authority_index(&cross_purpose);
    cross_purpose.issuer_keys[index].public_key =
        cross_purpose.issuer_keys[authority].public_key.clone();
    assert_eq!(policy_code(&cross_purpose), "SIGNER_KEY_MATERIAL_REUSED");
}
