//! Purpose separation for dogfood launch keys. This suite proves key-policy
//! shape and lookup only; admission still changes in the next bounded slice.

use std::{fs, path::PathBuf};

use bullet_wire::{
    AuthorityAudience, DogfoodAudienceV1, DogfoodBindingV1, KeyAlgorithmV1, KeyPurposeV1,
    PolicySnapshotV1, decode_canonical,
};

const FIXTURE: &str = "crates/bullet-wire/tests/fixtures/policy-v1alpha2-live-enabled.json";
const DOGFOOD_ISSUER: &str = "dogfood-operator";
const DOGFOOD_KEY_ID: &str = "dogfood-launch-1";
const DOGFOOD_PUBLIC_KEY: &str = "1111111111111111111111111111111111111111111111111111111111111111";

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

fn dogfood_index(policy: &PolicySnapshotV1) -> usize {
    policy
        .issuer_keys
        .iter()
        .position(|key| key.key_purpose == KeyPurposeV1::DogfoodLaunchSigning)
        .unwrap()
}

fn dogfood_policy() -> PolicySnapshotV1 {
    let mut policy = fixture();
    policy.sandbox_policy.live_admission_enabled = false;
    let mut key = policy.issuer_keys[authority_index(&policy)].clone();
    key.issuer = DOGFOOD_ISSUER.into();
    key.key_id = DOGFOOD_KEY_ID.into();
    key.key_purpose = KeyPurposeV1::DogfoodLaunchSigning;
    key.public_key = DOGFOOD_PUBLIC_KEY.into();
    key.audiences.clear();
    policy.issuer_keys.push(key);
    policy
}

fn code(policy: &PolicySnapshotV1) -> &'static str {
    policy.validate().unwrap_err().code()
}

fn lookup_code(policy: &PolicySnapshotV1, issuer: &str, key_id: &str, now: u64) -> &'static str {
    policy
        .dogfood_signer_key_at(issuer, key_id, &DogfoodBindingV1::read_only_propose(), now)
        .unwrap_err()
        .code()
}

#[test]
fn wire_values_are_closed_and_dogfood_is_not_an_authority_audience() {
    assert_eq!(
        serde_json::to_string(&KeyPurposeV1::DogfoodLaunchSigning).unwrap(),
        r#""dogfood-launch-signing""#
    );
    assert!(decode_canonical::<KeyPurposeV1>(br#""dogfood-signing""#).is_err());
    assert!(decode_canonical::<KeyPurposeV1>(br#""DOGFOOD-LAUNCH-SIGNING""#).is_err());
    assert_eq!(
        serde_json::to_string(&DogfoodAudienceV1::DogfoodRunner).unwrap(),
        r#""dogfood-runner""#
    );
    assert!(decode_canonical::<AuthorityAudience>(br#""dogfood-runner""#).is_err());
}

#[test]
fn dedicated_key_resolves_but_general_authority_refuses_it() {
    let policy = dogfood_policy();
    policy.validate().unwrap();
    let now = policy.activation_at_unix_ms;
    let key = policy
        .dogfood_signer_key_at(
            DOGFOOD_ISSUER,
            DOGFOOD_KEY_ID,
            &DogfoodBindingV1::read_only_propose(),
            now,
        )
        .unwrap();
    assert_eq!(key.public_key, DOGFOOD_PUBLIC_KEY);
    assert!(key.audiences.is_empty());
    assert_eq!(
        policy
            .authority_key_at(
                DOGFOOD_ISSUER,
                DOGFOOD_KEY_ID,
                AuthorityAudience::ProviderRunner,
                now,
            )
            .unwrap_err()
            .code(),
        "AUTHORITY_KEY_WRONG_PURPOSE"
    );

    let mut dogfood_only = policy.clone();
    dogfood_only
        .issuer_keys
        .remove(authority_index(&dogfood_only));
    dogfood_only.validate().unwrap();
    dogfood_only
        .dogfood_signer_key_at(
            DOGFOOD_ISSUER,
            DOGFOOD_KEY_ID,
            &DogfoodBindingV1::read_only_propose(),
            now,
        )
        .unwrap();
    dogfood_only.sandbox_policy.live_admission_enabled = true;
    assert_eq!(
        dogfood_only.validate().unwrap_err().code(),
        "LIVE_ADMISSION_REQUIRES_RUNNER_KEY"
    );
}

#[test]
fn identity_purpose_scope_and_policy_context_are_exact() {
    let policy = dogfood_policy();
    let now = policy.activation_at_unix_ms;
    assert_eq!(
        lookup_code(&policy, "unknown", DOGFOOD_KEY_ID, now),
        "DOGFOOD_SIGNER_KEY_UNKNOWN"
    );
    assert_eq!(
        lookup_code(&policy, DOGFOOD_ISSUER, "unknown", now),
        "DOGFOOD_SIGNER_KEY_UNKNOWN"
    );
    assert_eq!(
        lookup_code(
            &policy,
            &policy.issuer_keys[authority_index(&policy)].issuer,
            &policy.issuer_keys[authority_index(&policy)].key_id,
            now,
        ),
        "DOGFOOD_SIGNER_KEY_WRONG_PURPOSE"
    );
    assert_eq!(
        lookup_code(
            &policy,
            &policy.issuer_keys[release_index(&policy)].issuer,
            &policy.issuer_keys[release_index(&policy)].key_id,
            now,
        ),
        "DOGFOOD_SIGNER_KEY_WRONG_PURPOSE"
    );

    let mut binding = DogfoodBindingV1::read_only_propose();
    binding.schema_version = "v2".into();
    assert_eq!(
        policy
            .dogfood_signer_key_at(DOGFOOD_ISSUER, DOGFOOD_KEY_ID, &binding, now)
            .unwrap_err()
            .code(),
        "INVALID_DOGFOOD_BINDING"
    );

    let mut live = policy.clone();
    live.sandbox_policy.live_admission_enabled = true;
    assert_eq!(
        lookup_code(&live, DOGFOOD_ISSUER, DOGFOOD_KEY_ID, now),
        "DOGFOOD_REFUSES_LIVE_ADMISSION"
    );
    let mut first_generation = policy.clone();
    first_generation.policy_generation = 1;
    assert_eq!(
        lookup_code(&first_generation, DOGFOOD_ISSUER, DOGFOOD_KEY_ID, now,),
        "LIVE_ADMISSION_REQUIRES_GENERATION"
    );
    let mut old_schema = policy.clone();
    old_schema.schema_version = "v1alpha1".into();
    assert_eq!(
        lookup_code(&old_schema, DOGFOOD_ISSUER, DOGFOOD_KEY_ID, now),
        "UNSUPPORTED_POLICY_SCHEMA"
    );
}

#[test]
fn key_shape_identity_and_material_reuse_refuse() {
    let policy = dogfood_policy();
    let index = dogfood_index(&policy);

    let mut audience = policy.clone();
    audience.issuer_keys[index].audiences = vec![AuthorityAudience::ProviderRunner];
    assert_eq!(code(&audience), "INVALID_DOGFOOD_PUBLIC_KEY");

    let mut algorithm = policy.clone();
    algorithm.issuer_keys[index].algorithm = KeyAlgorithmV1::SshEd25519;
    assert_eq!(code(&algorithm), "INVALID_DOGFOOD_PUBLIC_KEY");

    let mut encoding = policy.clone();
    encoding.issuer_keys[index].public_key = "AA".repeat(32);
    assert_eq!(code(&encoding), "INVALID_DOGFOOD_PUBLIC_KEY");

    let mut authority_scope = policy.clone();
    let authority = authority_index(&authority_scope);
    authority_scope.issuer_keys[authority].audiences.clear();
    assert_eq!(code(&authority_scope), "INVALID_AUTHORITY_PUBLIC_KEY");

    let mut duplicate_identity = policy.clone();
    duplicate_identity
        .issuer_keys
        .push(duplicate_identity.issuer_keys[index].clone());
    assert_eq!(code(&duplicate_identity), "INVALID_ISSUER_KEY_LIFECYCLE");

    let mut duplicate_material = policy.clone();
    let mut alias = duplicate_material.issuer_keys[index].clone();
    alias.issuer = "dogfood-alias".into();
    alias.key_id = "dogfood-alias-1".into();
    duplicate_material.issuer_keys.push(alias);
    assert_eq!(code(&duplicate_material), "SIGNER_KEY_MATERIAL_REUSED");

    let mut cross_purpose = policy;
    let authority = authority_index(&cross_purpose);
    cross_purpose.issuer_keys[index].public_key =
        cross_purpose.issuer_keys[authority].public_key.clone();
    assert_eq!(code(&cross_purpose), "SIGNER_KEY_MATERIAL_REUSED");
}

#[test]
fn policy_key_and_revocation_windows_use_inclusive_exclusive_boundaries() {
    let policy = dogfood_policy();
    let activation = policy.activation_at_unix_ms;
    let expiry = policy.expires_at_unix_ms;
    assert_eq!(
        lookup_code(&policy, DOGFOOD_ISSUER, DOGFOOD_KEY_ID, activation - 1),
        "POLICY_NOT_ACTIVE"
    );
    policy
        .dogfood_signer_key_at(
            DOGFOOD_ISSUER,
            DOGFOOD_KEY_ID,
            &DogfoodBindingV1::read_only_propose(),
            activation,
        )
        .unwrap();
    policy
        .dogfood_signer_key_at(
            DOGFOOD_ISSUER,
            DOGFOOD_KEY_ID,
            &DogfoodBindingV1::read_only_propose(),
            expiry - 1,
        )
        .unwrap();
    assert_eq!(
        lookup_code(&policy, DOGFOOD_ISSUER, DOGFOOD_KEY_ID, expiry),
        "POLICY_NOT_ACTIVE"
    );

    let mut delayed = policy.clone();
    let index = dogfood_index(&delayed);
    delayed.issuer_keys[index].activates_at_unix_ms = activation + 100;
    delayed.issuer_keys[index].expires_at_unix_ms = activation + 200;
    assert_eq!(
        lookup_code(&delayed, DOGFOOD_ISSUER, DOGFOOD_KEY_ID, activation + 99,),
        "DOGFOOD_SIGNER_KEY_INACTIVE"
    );
    delayed
        .dogfood_signer_key_at(
            DOGFOOD_ISSUER,
            DOGFOOD_KEY_ID,
            &DogfoodBindingV1::read_only_propose(),
            activation + 100,
        )
        .unwrap();
    delayed
        .dogfood_signer_key_at(
            DOGFOOD_ISSUER,
            DOGFOOD_KEY_ID,
            &DogfoodBindingV1::read_only_propose(),
            activation + 199,
        )
        .unwrap();
    assert_eq!(
        lookup_code(&delayed, DOGFOOD_ISSUER, DOGFOOD_KEY_ID, activation + 200,),
        "DOGFOOD_SIGNER_KEY_INACTIVE"
    );

    let mut revoked = policy.clone();
    let index = dogfood_index(&revoked);
    revoked.issuer_keys[index].revoked_at_unix_ms = Some(activation + 100);
    revoked
        .dogfood_signer_key_at(
            DOGFOOD_ISSUER,
            DOGFOOD_KEY_ID,
            &DogfoodBindingV1::read_only_propose(),
            activation + 99,
        )
        .unwrap();
    assert_eq!(
        lookup_code(&revoked, DOGFOOD_ISSUER, DOGFOOD_KEY_ID, activation + 100,),
        "DOGFOOD_SIGNER_KEY_INACTIVE"
    );

    let mut nonoverlap = policy;
    let index = dogfood_index(&nonoverlap);
    nonoverlap.issuer_keys[index].activates_at_unix_ms = expiry + 100;
    nonoverlap.issuer_keys[index].expires_at_unix_ms = expiry + 200;
    nonoverlap.issuer_keys[index].retain_until_unix_ms = expiry + 15_200;
    assert_eq!(
        lookup_code(&nonoverlap, DOGFOOD_ISSUER, DOGFOOD_KEY_ID, activation,),
        "DOGFOOD_SIGNER_KEY_INACTIVE"
    );
}
