//! v1alpha2 policy snapshots (ADR 0012): live provider admission is
//! representable only behind an operator generation and a provider-runner
//! authority key. v1alpha1 behaviour is asserted unchanged here while the
//! existing `policy_registry.rs` suite stays untouched.

use std::{fs, path::PathBuf};

use bullet_wire::{
    AuthorityAudience, ContractCatalogV1, DogfoodBindingV1, KeyAlgorithmV1, KeyPurposeV1,
    LIVE_ADMISSION_MIN_GENERATION, POLICY_SCHEMA_VERSION, POLICY_SCHEMA_VERSION_V1ALPHA2,
    PolicySchemaVersion, PolicySnapshotV1, canonical_json, decode_canonical,
    refuse_dogfood_binding_as_live,
    v1alpha1::{INVARIANT_REGISTRY_HASH, SCHEMA_BUNDLE_HASH},
    validate_dogfood_admission, validate_live_admission,
};
use serde_json::{Value, json};

const FIXTURE: &str = "crates/bullet-wire/tests/fixtures/policy-v1alpha2-live-enabled.json";
const COMMITTED_POLICY: &str = "policy/v1alpha1/policy.json";
const ISSUER: &str = "bullet-kernel-local";
const KEY_ID: &str = "authority-test-1";
const PUBLIC_KEY_HEX: &str = "1eb9dbbbbc047c03fd70604e0071f0987e16b28b757225c11f00415d0e20b1a2";
const DOGFOOD_PUBLIC_KEY: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const DOGFOOD_GEN_REQUIRED: &str = "DOGFOOD_ADMISSION_REQUIRES_GENERATION";
const DOGFOOD_SIGNER_REQUIRED: &str = "DOGFOOD_ADMISSION_REQUIRES_SIGNER_KEY";
const V1ALPHA1_UNSAFE_REASON: &str =
    "v1alpha1 Gate 0 policy must remain offline, conservative, and T0-anchored";

type Relax = fn(&mut PolicySnapshotV1);

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf()
}

fn read(path: &str) -> Vec<u8> {
    fs::read(root().join(path)).unwrap()
}

fn fixture() -> PolicySnapshotV1 {
    decode_canonical(&read(FIXTURE)).unwrap()
}

fn committed() -> PolicySnapshotV1 {
    decode_canonical(&read(COMMITTED_POLICY)).unwrap()
}

fn runner_key(policy: &PolicySnapshotV1) -> usize {
    policy
        .issuer_keys
        .iter()
        .position(|key| key.issuer == ISSUER && key.key_id == KEY_ID)
        .unwrap()
}

fn add_dogfood_key(policy: &mut PolicySnapshotV1) -> usize {
    let mut key = policy.issuer_keys[runner_key(policy)].clone();
    key.issuer = "dogfood-operator".into();
    key.key_id = "dogfood-launch-1".into();
    key.key_purpose = KeyPurposeV1::DogfoodLaunchSigning;
    key.public_key = DOGFOOD_PUBLIC_KEY.into();
    key.audiences.clear();
    policy.issuer_keys.push(key);
    policy.issuer_keys.len() - 1
}

fn refusal(policy: &PolicySnapshotV1) -> &'static str {
    policy.validate().unwrap_err().code()
}

fn dogfood_refusal(policy: &PolicySnapshotV1) -> &'static str {
    validate_dogfood_admission(policy, &DogfoodBindingV1::read_only_propose())
        .unwrap_err()
        .code()
}

#[test]
fn fixture_is_canonical_strict_and_validates() {
    let bytes = read(FIXTURE);
    let policy = fixture();
    assert_eq!(canonical_json(&policy).unwrap(), bytes);
    serde_json::from_slice::<bullet_wire::v1alpha1::PolicySnapshotV1>(&bytes).unwrap();
    policy.validate().unwrap();
    policy.validate_at(policy.activation_at_unix_ms).unwrap();
    assert_eq!(policy.schema().unwrap(), PolicySchemaVersion::V1Alpha2);
    assert_eq!(policy.schema_version, POLICY_SCHEMA_VERSION_V1ALPHA2);
    assert_eq!(policy.policy_generation, LIVE_ADMISSION_MIN_GENERATION);
    assert!(policy.sandbox_policy.live_admission_enabled);
    assert_eq!(policy.schema_bundle_hash.to_string(), SCHEMA_BUNDLE_HASH);
    assert_eq!(
        policy.invariant_registry_hash.to_string(),
        INVARIANT_REGISTRY_HASH
    );
}

#[test]
fn fixture_differs_from_the_committed_policy_only_by_design() {
    let policy = fixture();
    let committed = committed();
    assert_eq!(committed.schema_version, POLICY_SCHEMA_VERSION);
    assert_eq!(committed.policy_generation, 1);
    assert!(!committed.sandbox_policy.live_admission_enabled);
    assert_eq!(policy.schema_bundle_hash, committed.schema_bundle_hash);
    assert_eq!(
        policy.invariant_registry_hash,
        committed.invariant_registry_hash
    );
    assert_eq!(
        policy.activation_at_unix_ms,
        committed.activation_at_unix_ms
    );
    assert_eq!(policy.expires_at_unix_ms, committed.expires_at_unix_ms);
    assert_eq!(policy.risk_policy, committed.risk_policy);
    assert_eq!(policy.evidence_policy, committed.evidence_policy);
    assert_eq!(policy.budget_policy, committed.budget_policy);
    assert_eq!(policy.route_policy, committed.route_policy);
    let mut sandbox = policy.sandbox_policy.clone();
    sandbox.live_admission_enabled = false;
    assert_eq!(sandbox, committed.sandbox_policy);
    assert_eq!(policy.issuer_keys.len(), 2);
    assert_eq!(policy.issuer_keys[0], committed.issuer_keys[0]);
    let key = &policy.issuer_keys[runner_key(&policy)];
    assert_eq!(key.key_purpose, KeyPurposeV1::AuthoritySigning);
    assert_eq!(key.algorithm, KeyAlgorithmV1::PasetoV4Public);
    assert_eq!(key.audiences, vec![AuthorityAudience::ProviderRunner]);
    assert_eq!(key.public_key, PUBLIC_KEY_HEX);
    assert_eq!(key.revoked_at_unix_ms, None);
}

#[test]
fn no_committed_policy_enables_live_admission() {
    let mut inspected = 0;
    for entry in fs::read_dir(root().join("policy/v1alpha1")).unwrap() {
        let path = entry.unwrap().path();
        let value = serde_json::from_slice::<Value>(&fs::read(&path).unwrap()).unwrap();
        if let Some(sandbox) = value.get("sandbox_policy") {
            assert_eq!(sandbox["live_admission_enabled"], json!(false), "{path:?}");
            assert_eq!(value["schema_version"], json!(POLICY_SCHEMA_VERSION));
            assert_eq!(value["policy_generation"], json!(1));
            inspected += 1;
        }
    }
    assert_eq!(inspected, 2, "policy.json and policy-template.json");
}

#[test]
fn live_admission_requires_a_qualifying_provider_runner_key() {
    let mut removed = fixture();
    removed.issuer_keys.remove(runner_key(&removed));
    assert_eq!(refusal(&removed), "LIVE_ADMISSION_REQUIRES_RUNNER_KEY");

    let mut wrong_audience = fixture();
    let index = runner_key(&wrong_audience);
    wrong_audience.issuer_keys[index].audiences = vec![AuthorityAudience::BulletGitd];
    assert_eq!(
        refusal(&wrong_audience),
        "LIVE_ADMISSION_REQUIRES_RUNNER_KEY"
    );

    let mut revoked = fixture();
    let index = runner_key(&revoked);
    revoked.issuer_keys[index].revoked_at_unix_ms = Some(revoked.expires_at_unix_ms - 1);
    assert_eq!(refusal(&revoked), "LIVE_ADMISSION_REQUIRES_RUNNER_KEY");

    let mut lapsed = fixture();
    let index = runner_key(&lapsed);
    let activation = lapsed.activation_at_unix_ms;
    lapsed.issuer_keys[index].activates_at_unix_ms = activation - 2_000_000;
    lapsed.issuer_keys[index].expires_at_unix_ms = activation - 1_000_000;
    lapsed.issuer_keys[index].retain_until_unix_ms = activation;
    assert_eq!(refusal(&lapsed), "LIVE_ADMISSION_REQUIRES_RUNNER_KEY");

    let mut offline = removed;
    offline.sandbox_policy.live_admission_enabled = false;
    offline.validate().unwrap();
}

#[test]
fn live_admission_requires_generation_two() {
    let mut first = fixture();
    first.policy_generation = 1;
    let error = first.validate().unwrap_err();
    assert_eq!(error.code(), "LIVE_ADMISSION_REQUIRES_GENERATION");
    assert!(
        error.reason().contains("generation 2"),
        "{}",
        error.reason()
    );

    let mut zero = fixture();
    zero.policy_generation = 0;
    assert_eq!(refusal(&zero), "INVALID_POLICY_WINDOW");

    let mut later = fixture();
    later.policy_generation = 7;
    later.validate().unwrap();

    let mut offline = fixture();
    offline.policy_generation = 1;
    offline.sandbox_policy.live_admission_enabled = false;
    offline.validate().unwrap();
}

#[test]
fn immutable_conservatism_set_is_unsafe_in_v1alpha2() {
    let relaxations: [(&str, Relax); 8] = [
        ("evolutionary_authority", |policy| {
            policy.route_policy.evolutionary_authority = true;
        }),
        ("maximum_lease_ttl_seconds", |policy| {
            policy.budget_policy.maximum_lease_ttl_seconds = 16;
        }),
        ("unknown_quota_is_headroom", |policy| {
            policy.budget_policy.unknown_quota_is_headroom = true;
        }),
        ("arbitrary_shell_gates", |policy| {
            policy.sandbox_policy.arbitrary_shell_gates = true;
        }),
        ("author_evidence_is_independent", |policy| {
            policy.evidence_policy.author_evidence_is_independent = true;
        }),
        ("unknown_satisfies_gate", |policy| {
            policy.evidence_policy.unknown_satisfies_gate = true;
        }),
        ("r2_requires_sealed_product_holdout", |policy| {
            policy.evidence_policy.r2_requires_sealed_product_holdout = false;
        }),
        ("universal_incumbent", |policy| {
            policy.route_policy.universal_incumbent = "T1".to_owned();
        }),
    ];
    for (name, relax) in relaxations {
        let mut policy = fixture();
        relax(&mut policy);
        let error = policy.validate().unwrap_err();
        assert_eq!(error.code(), "UNSAFE_POLICY", "{name}");
        assert!(
            error.reason().starts_with("v1alpha2"),
            "{name}: {}",
            error.reason()
        );
    }

    let mut stacked = fixture();
    stacked.route_policy.evolutionary_authority = true;
    stacked.policy_generation = 1;
    stacked.issuer_keys.remove(runner_key(&stacked));
    assert_eq!(refusal(&stacked), "UNSAFE_POLICY");
}

#[test]
fn v1alpha1_live_admission_remains_unsafe_policy_unchanged() {
    let mut downgraded = fixture();
    downgraded.schema_version = POLICY_SCHEMA_VERSION.to_owned();
    let error = downgraded.validate().unwrap_err();
    assert_eq!(error.code(), "UNSAFE_POLICY");
    assert_eq!(error.reason(), V1ALPHA1_UNSAFE_REASON);

    let mut committed_live = committed();
    committed_live.sandbox_policy.live_admission_enabled = true;
    let error = committed_live.validate().unwrap_err();
    assert_eq!(error.code(), "UNSAFE_POLICY");
    assert_eq!(error.reason(), V1ALPHA1_UNSAFE_REASON);

    let committed = committed();
    committed.validate().unwrap();
    committed
        .validate_at(committed.activation_at_unix_ms)
        .unwrap();
    assert_eq!(committed.schema().unwrap(), PolicySchemaVersion::V1Alpha1);
}

#[test]
fn unsupported_and_nested_schema_versions_fail_closed() {
    for version in ["v1alpha3", "v1alpha", "V1ALPHA2", ""] {
        let mut policy = fixture();
        policy.schema_version = version.to_owned();
        assert_eq!(refusal(&policy), "UNSUPPORTED_POLICY_SCHEMA", "{version}");
        assert_eq!(
            PolicySchemaVersion::parse(version).unwrap_err().code(),
            "UNSUPPORTED_POLICY_SCHEMA"
        );
    }
    let mut nested = fixture();
    nested.sandbox_policy.schema_version = POLICY_SCHEMA_VERSION_V1ALPHA2.to_owned();
    assert_eq!(refusal(&nested), "UNSUPPORTED_POLICY_SCHEMA");
    let mut keyed = fixture();
    let index = runner_key(&keyed);
    keyed.issuer_keys[index].schema_version = POLICY_SCHEMA_VERSION_V1ALPHA2.to_owned();
    assert_eq!(refusal(&keyed), "UNSUPPORTED_POLICY_SCHEMA");

    assert_eq!(
        PolicySchemaVersion::ACCEPTED,
        [POLICY_SCHEMA_VERSION, POLICY_SCHEMA_VERSION_V1ALPHA2]
    );
    for version in PolicySchemaVersion::ACCEPTED {
        assert_eq!(
            PolicySchemaVersion::parse(version).unwrap().as_str(),
            version
        );
    }
}

#[test]
fn authority_key_lookup_admits_the_fixture_runner_key() {
    let policy = fixture();
    let now = policy.activation_at_unix_ms;
    let key = policy
        .authority_key_at(ISSUER, KEY_ID, AuthorityAudience::ProviderRunner, now)
        .unwrap();
    assert_eq!(key.key_id, KEY_ID);
    assert_eq!(key.public_key, PUBLIC_KEY_HEX);
    assert_eq!(
        policy
            .authority_key_at(ISSUER, KEY_ID, AuthorityAudience::EffectBroker, now)
            .unwrap_err()
            .code(),
        "AUTHORITY_KEY_AUDIENCE_MISMATCH"
    );
    assert_eq!(
        policy
            .authority_key_at(
                "bullet-farm-offline-policy",
                "release-signing-alpha",
                AuthorityAudience::ProviderRunner,
                now,
            )
            .unwrap_err()
            .code(),
        "AUTHORITY_KEY_WRONG_PURPOSE"
    );
}

#[test]
fn validate_at_applies_the_authority_key_instant_semantics() {
    let policy = fixture();
    let activation = policy.activation_at_unix_ms;
    policy.validate_at(activation).unwrap();
    policy.validate_at(policy.expires_at_unix_ms - 1).unwrap();
    assert_eq!(
        policy.validate_at(activation - 1).unwrap_err().code(),
        "POLICY_NOT_ACTIVE"
    );
    assert_eq!(
        policy
            .validate_at(policy.expires_at_unix_ms)
            .unwrap_err()
            .code(),
        "POLICY_NOT_ACTIVE"
    );

    let mut delayed = fixture();
    let index = runner_key(&delayed);
    delayed.issuer_keys[index].activates_at_unix_ms = activation + 1_000;
    delayed.validate().unwrap();
    assert_eq!(
        delayed.validate_at(activation).unwrap_err().code(),
        "LIVE_ADMISSION_REQUIRES_RUNNER_KEY"
    );
    delayed.validate_at(activation + 1_000).unwrap();

    let mut offline = fixture();
    offline.sandbox_policy.live_admission_enabled = false;
    offline.issuer_keys.remove(runner_key(&offline));
    offline.validate_at(activation).unwrap();
}

#[test]
fn schema_bundle_admits_v1alpha2_only_for_the_policy_snapshot() {
    let catalog =
        decode_canonical::<ContractCatalogV1>(&read("contracts/v1alpha1/contract-catalog.json"))
            .unwrap();
    let bundle = catalog.json_schema_bundle().unwrap();
    assert_eq!(
        bundle["schemas"]["PolicySnapshotV1"]["properties"]["schema_version"],
        json!({"type": "string", "enum": ["v1alpha1", "v1alpha2"]})
    );
    for record in [
        "IssuerKeyV1",
        "SandboxPolicyV1",
        "RoutePolicyV1",
        "LaunchGrantClaimsV1",
    ] {
        assert_eq!(
            bundle["schemas"][record]["properties"]["schema_version"],
            json!({"type": "string", "const": "v1alpha1"}),
            "{record}"
        );
    }
    let committed_bundle =
        serde_json::from_slice::<Value>(&read("contracts/v1alpha1/schema-bundle.json")).unwrap();
    assert_eq!(
        committed_bundle["schemas"]["PolicySnapshotV1"]["properties"]["schema_version"]["enum"],
        json!(["v1alpha1", "v1alpha2"])
    );
}

#[test]
fn committed_policy_stays_offline_and_cannot_satisfy_live_or_dogfood() {
    let policy = committed();
    assert!(!policy.sandbox_policy.live_admission_enabled);
    policy.validate().unwrap();
    assert_eq!(
        validate_live_admission(&policy).unwrap_err().code(),
        "LIVE_ADMISSION_DISABLED"
    );
    assert_eq!(
        validate_dogfood_admission(&policy, &DogfoodBindingV1::read_only_propose())
            .unwrap_err()
            .code(),
        "UNSUPPORTED_POLICY_SCHEMA"
    );
}

#[test]
fn general_live_path_refuses_a_dogfood_binding() {
    let binding = DogfoodBindingV1::read_only_propose();
    assert_eq!(
        refuse_dogfood_binding_as_live(&binding).unwrap_err().code(),
        "LIVE_ADMISSION_REFUSES_DOGFOOD_BINDING"
    );
    let mut dogfood_only = fixture();
    dogfood_only.sandbox_policy.live_admission_enabled = false;
    add_dogfood_key(&mut dogfood_only);
    dogfood_only
        .issuer_keys
        .retain(|key| key.key_purpose == KeyPurposeV1::DogfoodLaunchSigning);
    dogfood_only.validate().unwrap();
    assert_eq!(
        validate_live_admission(&dogfood_only).unwrap_err().code(),
        "LIVE_ADMISSION_DISABLED"
    );
    dogfood_only.sandbox_policy.live_admission_enabled = true;
    assert_eq!(
        validate_live_admission(&dogfood_only).unwrap_err().code(),
        "LIVE_ADMISSION_REQUIRES_RUNNER_KEY"
    );
}

#[test]
fn dogfood_path_refuses_a_general_live_binding_and_unknown_fields() {
    let mut live = fixture();
    add_dogfood_key(&mut live);
    assert_eq!(dogfood_refusal(&live), "DOGFOOD_REFUSES_LIVE_ADMISSION");
    let mut provider_only = fixture();
    provider_only.sandbox_policy.live_admission_enabled = false;
    provider_only
        .issuer_keys
        .retain(|key| key.key_purpose == KeyPurposeV1::AuthoritySigning);
    assert_eq!(dogfood_refusal(&provider_only), DOGFOOD_SIGNER_REQUIRED);

    let mut first_generation = provider_only.clone();
    first_generation.policy_generation = 1;
    assert_eq!(dogfood_refusal(&first_generation), DOGFOOD_GEN_REQUIRED);

    let mut dogfood = provider_only;
    let key = add_dogfood_key(&mut dogfood);
    validate_dogfood_admission(&dogfood, &DogfoodBindingV1::read_only_propose()).unwrap();
    let mut invalid = DogfoodBindingV1::read_only_propose();
    invalid.schema_version = "v2".into();
    let error = validate_dogfood_admission(&dogfood, &invalid).unwrap_err();
    assert_eq!(error.code(), "INVALID_DOGFOOD_BINDING");

    let mut malformed = dogfood.clone();
    malformed.issuer_keys[key].public_key = "invalid".into();
    assert_eq!(dogfood_refusal(&malformed), "INVALID_DOGFOOD_PUBLIC_KEY");
    let mut lifecycle = dogfood.clone();
    lifecycle.issuer_keys[key].retain_until_unix_ms = 0;
    assert_eq!(dogfood_refusal(&lifecycle), "INVALID_ISSUER_KEY_LIFECYCLE");
    let mut unsafe_policy = dogfood.clone();
    unsafe_policy.route_policy.evolutionary_authority = true;
    assert_eq!(dogfood_refusal(&unsafe_policy), "UNSAFE_POLICY");
    let mut invalid_policy = dogfood.clone();
    invalid_policy.activation_at_unix_ms = invalid_policy.expires_at_unix_ms;
    assert_eq!(dogfood_refusal(&invalid_policy), "INVALID_POLICY_WINDOW");

    let mut removed = dogfood.clone();
    removed.issuer_keys.remove(key);
    let mut revoked = dogfood.clone();
    revoked.issuer_keys[key].revoked_at_unix_ms = Some(revoked.activation_at_unix_ms);
    let mut nonoverlap = dogfood;
    let expiry = nonoverlap.expires_at_unix_ms;
    nonoverlap.issuer_keys[key].activates_at_unix_ms = expiry + 100;
    nonoverlap.issuer_keys[key].expires_at_unix_ms = expiry + 200;
    nonoverlap.issuer_keys[key].retain_until_unix_ms = expiry + 15_200;
    for policy in [&removed, &revoked, &nonoverlap] {
        assert_eq!(dogfood_refusal(policy), DOGFOOD_SIGNER_REQUIRED);
    }

    let unknown = serde_json::from_str::<DogfoodBindingV1>(
        r#"{"schema_version":"v1alpha1","audience":"dogfood-runner","operation":"read-only-propose","extra":true}"#,
    );
    assert!(unknown.is_err());
}
