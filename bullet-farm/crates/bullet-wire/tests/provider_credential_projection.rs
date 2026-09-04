use std::collections::BTreeSet;

use bullet_wire::{
    Blake3Digest, CredentialProjectionProfileId, DOGFOOD_SCHEMA_VERSION, DogfoodRunId,
    LaunchProvider, MAX_CREDENTIAL_PROJECTION_TTL_MS, PrincipalId, ProviderCredentialProjectionId,
    ProviderCredentialProjectionV1, WireError, canonical_json,
    decode_provider_credential_projection,
};
use serde_json::{Value, json};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

fn digest(seed: u8) -> Blake3Digest {
    Blake3Digest::from_bytes([seed; 32])
}

fn projection(provider: LaunchProvider, seed: u8) -> ProviderCredentialProjectionV1 {
    ProviderCredentialProjectionV1 {
        schema_version: DOGFOOD_SCHEMA_VERSION.into(),
        projection_instance_id: ProviderCredentialProjectionId::from_digest(digest(seed)),
        credential_projection_profile_id: CredentialProjectionProfileId::from_digest(digest(2)),
        run_id: DogfoodRunId::from_digest(digest(3)),
        provider,
        service_identity_id: PrincipalId::from_digest(digest(4)),
        activates_at_unix_ms: 1_000,
        expires_at_unix_ms: 2_000,
        target_policy_digest: digest(5),
        secret_commitment_digest: digest(6),
    }
}

fn decode_value(value: &Value) -> Result<ProviderCredentialProjectionV1, WireError> {
    decode_provider_credential_projection(&serde_jcs::to_vec(value).unwrap())
}

fn code(result: Result<(), WireError>) -> &'static str {
    result.unwrap_err().code()
}

#[test]
fn all_four_providers_round_trip_as_secret_free_commitment_bodies() {
    let providers = [
        LaunchProvider::Claude,
        LaunchProvider::Codex,
        LaunchProvider::Cursor,
        LaunchProvider::Agy,
    ];
    let mut digests = BTreeSet::new();
    for (index, provider) in providers.into_iter().enumerate() {
        let projection = projection(provider, (index + 10) as u8);
        projection.validate().unwrap();
        let bytes = canonical_json(&projection).unwrap();
        assert_eq!(
            decode_provider_credential_projection(&bytes).unwrap(),
            projection
        );
        assert!(digests.insert(projection.projection_digest().unwrap()));
        assert!(
            projection
                .projection_instance_id
                .as_str()
                .starts_with("pcp_")
        );
        assert!(
            projection
                .credential_projection_profile_id
                .as_str()
                .starts_with("cpp_")
        );

        let object = serde_json::to_value(&projection).unwrap();
        let keys = object
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            keys,
            BTreeSet::from([
                "activates_at_unix_ms".into(),
                "credential_projection_profile_id".into(),
                "expires_at_unix_ms".into(),
                "projection_instance_id".into(),
                "provider".into(),
                "run_id".into(),
                "schema_version".into(),
                "secret_commitment_digest".into(),
                "service_identity_id".into(),
                "target_policy_digest".into(),
            ])
        );
    }
    assert_eq!(digests.len(), providers.len());
}

#[test]
fn canonical_decode_refuses_duplicate_unknown_and_secret_bearing_fields() {
    let projection = projection(LaunchProvider::Claude, 10);
    let text = String::from_utf8(canonical_json(&projection).unwrap()).unwrap();
    let duplicate = text.replacen(
        "\"provider\":\"claude\"",
        "\"provider\":\"claude\",\"provider\":\"claude\"",
        1,
    );
    assert!(decode_provider_credential_projection(duplicate.as_bytes()).is_err());

    for (name, value) in [
        ("secret_value", json!("not-a-secret")),
        ("credential_path", json!("/tmp/token")),
        ("environment_variable", json!("TOKEN")),
        ("target_allowlist", json!(["example.invalid"])),
    ] {
        let mut changed = serde_json::to_value(&projection).unwrap();
        changed[name] = value;
        assert!(decode_value(&changed).is_err(), "field {name} was accepted");
    }

    let mut alias = serde_json::to_value(&projection).unwrap();
    alias["provider"] = json!("antigravity");
    assert!(decode_value(&alias).is_err());
}

#[test]
fn validity_window_is_ordered_bounded_and_interoperable() {
    let mut projection = projection(LaunchProvider::Claude, 10);
    projection.expires_at_unix_ms = projection.activates_at_unix_ms;
    assert_eq!(code(projection.validate()), "CREDENTIAL_PROJECTION_INVALID");

    projection.expires_at_unix_ms =
        projection.activates_at_unix_ms + MAX_CREDENTIAL_PROJECTION_TTL_MS;
    projection.validate().unwrap();
    projection.expires_at_unix_ms += 1;
    assert_eq!(code(projection.validate()), "CREDENTIAL_PROJECTION_INVALID");

    projection = projection_for_time(MAX_SAFE_INTEGER + 1, MAX_SAFE_INTEGER + 2);
    assert_eq!(code(projection.validate()), "CREDENTIAL_PROJECTION_INVALID");

    projection = projection_for_time(2_000, 1_000);
    assert_eq!(code(projection.validate()), "CREDENTIAL_PROJECTION_INVALID");
}

#[test]
fn every_identity_and_commitment_changes_the_projection_digest() {
    let base = projection(LaunchProvider::Claude, 10);
    let expected = base.projection_digest().unwrap();
    let mutations: [fn(&mut ProviderCredentialProjectionV1); 9] = [
        |value| {
            value.projection_instance_id = ProviderCredentialProjectionId::from_digest(digest(20));
        },
        |value| {
            value.credential_projection_profile_id =
                CredentialProjectionProfileId::from_digest(digest(21));
        },
        |value| value.run_id = DogfoodRunId::from_digest(digest(22)),
        |value| value.provider = LaunchProvider::Codex,
        |value| value.service_identity_id = PrincipalId::from_digest(digest(23)),
        |value| value.activates_at_unix_ms += 1,
        |value| value.expires_at_unix_ms += 1,
        |value| value.target_policy_digest = digest(24),
        |value| value.secret_commitment_digest = digest(25),
    ];
    for mutate in mutations {
        let mut changed = base.clone();
        mutate(&mut changed);
        assert_ne!(changed.projection_digest().unwrap(), expected);
    }
}

fn projection_for_time(activates: u64, expires: u64) -> ProviderCredentialProjectionV1 {
    let mut projection = projection(LaunchProvider::Claude, 10);
    projection.activates_at_unix_ms = activates;
    projection.expires_at_unix_ms = expires;
    projection
}
