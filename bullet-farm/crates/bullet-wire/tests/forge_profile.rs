use std::fmt::Debug;

use bullet_wire::{
    Blake3Digest, FORGE_PROFILE_SCHEMA_VERSION, ForgeCapability, ForgeKind, ForgeProfileId,
    ForgeProfileRegistry, PRIMARY_FORGE_PROFILE_DIGEST_DOMAIN, PrimaryForgeProfileV1,
    REPLICATION_INTENT_DIGEST_DOMAIN, ReplicationIntentKind, ReplicationIntentV1, RepositoryId,
    WireError, canonical_json, decode_primary_forge_profile, decode_replication_intent,
    hash_canonical, hash_framed_bytes,
};
use serde::Serialize;
use serde_json::{Value, json};

const LOOPBACK: &str = "http://127.0.0.1:3000";
const ACTIVATED_AT: u64 = 1_756_200_000_000;
const SCHEMA_INVALID: &str = "DOCUMENT_SCHEMA_INVALID";
const BAD_URL: &str = "INVALID_FORGE_PROFILE_URL";
const NOT_LOOPBACK: &str = "FORGE_PROFILE_URL_NOT_LOOPBACK";
const NOT_HTTPS: &str = "FORGE_PROFILE_URL_NOT_HTTPS";
const BAD_REF: &str = "INVALID_REPLICATION_REF";
const REPLAY: &str = "FORGE_PROFILE_GENERATION_REPLAY";
const REGRESSION: &str = "FORGE_PROFILE_GENERATION_REGRESSION";
const NO_SOURCE: &str = "REPLICATION_SOURCE_NOT_ACTIVE";
const NOT_INTEGRATION: &str = "REPLICATION_INTENT_NOT_INTEGRATION";
const LACKS_INTEGRATION: &str = "FORGE_PROFILE_LACKS_INTEGRATION_SUBJECT";
const PROFILE_MISMATCH: &str = "FORGE_PROFILE_DIGEST_MISMATCH";
const CAP_DUPLICATE: &str = "FORGE_PROFILE_CAPABILITY_DUPLICATE";
const DUP_KEY: &str = "DUPLICATE_JSON_KEY";
const UNSAFE_INT: &str = "UNSAFE_JSON_INTEGER";
const NON_CANONICAL: &str = "NON_CANONICAL_JSON";

fn repository(seed: &str) -> RepositoryId {
    RepositoryId::from_digest(hash_framed_bytes("test.repository", seed.as_bytes()).unwrap())
}

fn draft(repository_seed: &str, generation: u64) -> PrimaryForgeProfileV1 {
    PrimaryForgeProfileV1 {
        schema_version: FORGE_PROFILE_SCHEMA_VERSION.to_owned(),
        repository_id: repository(repository_seed),
        forge_kind: ForgeKind::Jeryu,
        base_url: LOOPBACK.to_owned(),
        capabilities: vec![
            ForgeCapability::ExpectedOldOid,
            ForgeCapability::IntegrationSubject,
            ForgeCapability::ProtectedRefs,
            ForgeCapability::ReadBack,
        ],
        generation,
        activated_by: "operator:ben".to_owned(),
        activated_at_unix_ms: ACTIVATED_AT,
        digest: Blake3Digest::from_bytes([0; 32]),
    }
}

fn sealed(repository_seed: &str, generation: u64) -> PrimaryForgeProfileV1 {
    draft(repository_seed, generation).seal().unwrap()
}

fn draft_with_url(kind: ForgeKind, url: &str) -> PrimaryForgeProfileV1 {
    let mut profile = draft("alpha", 1);
    profile.forge_kind = kind;
    profile.base_url = url.to_owned();
    profile
}

fn intent(
    source: &PrimaryForgeProfileV1,
    destination: &PrimaryForgeProfileV1,
) -> ReplicationIntentV1 {
    ReplicationIntentV1 {
        schema_version: FORGE_PROFILE_SCHEMA_VERSION.to_owned(),
        intent_kind: ReplicationIntentKind::Mirror,
        source_profile_id: source.profile_id(),
        destination_profile_id: destination.profile_id(),
        refs: vec![
            "refs/heads/bullet/candidate/abc".to_owned(),
            "refs/tags/v1.0.0".to_owned(),
        ],
        digest: Blake3Digest::from_bytes([0; 32]),
    }
}

fn intent_with_refs(source: &PrimaryForgeProfileV1, refs: &[&str]) -> ReplicationIntentV1 {
    let mut intent = intent(source, &sealed("mirror", 1));
    intent.refs = refs.iter().map(|name| (*name).to_owned()).collect();
    intent
}

fn value_of<T: Serialize>(record: &T) -> Value {
    serde_json::to_value(record).unwrap()
}

fn decode_profile(value: &Value) -> Result<PrimaryForgeProfileV1, WireError> {
    decode_primary_forge_profile(&canonical_json(value).unwrap())
}

fn decode_intent(value: &Value) -> Result<ReplicationIntentV1, WireError> {
    decode_replication_intent(&canonical_json(value).unwrap())
}

#[track_caller]
fn refused<T: Debug>(result: Result<T, WireError>, expected: &str) {
    let code = result.expect_err("expected a typed refusal").code();
    assert_eq!(code, expected);
}

#[test]
fn capability_and_kind_wire_names_follow_declaration_order() {
    let capabilities = [
        ForgeCapability::ExactShaChecks,
        ForgeCapability::ExpectedOldOid,
        ForgeCapability::IntegrationSubject,
        ForgeCapability::MergeGroups,
        ForgeCapability::ProtectedRefs,
        ForgeCapability::PullRequests,
        ForgeCapability::ReadBack,
    ];
    let names: Vec<String> = capabilities
        .iter()
        .map(|capability| value_of(capability).as_str().unwrap().to_owned())
        .collect();
    let expected = "exact_sha_checks expected_old_oid integration_subject merge_groups \
                    protected_refs pull_requests read_back";
    assert_eq!(names.join(" "), expected);
    assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(capabilities.windows(2).all(|pair| pair[0] < pair[1]));
    let kinds = [
        ForgeKind::Jeryu,
        ForgeKind::Github,
        ForgeKind::Gitlab,
        ForgeKind::LocalBare,
    ];
    assert_eq!(
        value_of(&kinds),
        json!(["jeryu", "github", "gitlab", "local-bare"])
    );
}

#[test]
fn sealed_profile_round_trips_and_binds_digest_to_identity_preimage() {
    let profile = sealed("alpha", 1);
    let bytes = canonical_json(&profile).unwrap();
    assert_eq!(decode_primary_forge_profile(&bytes).unwrap(), profile);

    let preimage = json!({
        "schema_version": "v1alpha1",
        "repository_id": profile.repository_id.to_string(),
        "forge_kind": "jeryu",
        "base_url": LOOPBACK,
        "capabilities": ["expected_old_oid", "integration_subject", "protected_refs", "read_back"],
        "generation": 1,
        "activated_by": "operator:ben",
        "activated_at_unix_ms": ACTIVATED_AT,
    });
    let expected = hash_canonical(PRIMARY_FORGE_PROFILE_DIGEST_DOMAIN, &preimage).unwrap();
    assert_eq!(profile.digest, expected);
    let id = profile.profile_id();
    assert_eq!(id.to_string(), format!("fpf_{}", expected.to_hex()));
    assert_eq!(id, ForgeProfileId::from_digest(expected));
    assert_ne!(sealed("alpha", 2).digest, profile.digest);
    assert_ne!(sealed("beta", 1).digest, profile.digest);
    let mut with_ignored_digest = draft("alpha", 1);
    with_ignored_digest.digest = Blake3Digest::from_bytes([0xff; 32]);
    assert_eq!(with_ignored_digest.seal().unwrap(), profile);
}

#[test]
fn profile_decoder_refuses_unknown_members_duplicates_and_unsafe_numbers() {
    let profile = sealed("alpha", 1);
    let mut unknown = value_of(&profile);
    unknown["operator_override"] = json!(true);
    refused(decode_profile(&unknown), SCHEMA_INVALID);
    let mut nested = value_of(&profile);
    nested["capabilities"] = json!([{ "name": "read_back" }]);
    refused(decode_profile(&nested), SCHEMA_INVALID);
    let mut unknown_capability = value_of(&profile);
    unknown_capability["capabilities"] = json!(["force_push"]);
    refused(decode_profile(&unknown_capability), SCHEMA_INVALID);

    let duplicate = br#"{"generation":1,"generation":2}"#;
    refused(decode_primary_forge_profile(duplicate), DUP_KEY);

    let canonical = String::from_utf8(canonical_json(&profile).unwrap()).unwrap();
    let big = canonical.replace("\"generation\":1", "\"generation\":9007199254740992");
    assert_ne!(big, canonical);
    refused(decode_primary_forge_profile(big.as_bytes()), UNSAFE_INT);
    let padded = canonical.replace(':', ": ");
    refused(
        decode_primary_forge_profile(padded.as_bytes()),
        NON_CANONICAL,
    );
}

#[test]
fn jeryu_profile_base_url_must_be_an_explicit_loopback_host() {
    let refusals = [
        ("https://git.neverhuman.org", NOT_LOOPBACK),
        ("http://localhost.evil.example", NOT_LOOPBACK),
        ("http://127.0.0.1.nip.io:3000", NOT_LOOPBACK),
        ("http://[::1]:0", NOT_LOOPBACK),
        ("http://127.0.0.1:65536", NOT_LOOPBACK),
        ("http://0.0.0.0:3000", NOT_LOOPBACK),
        ("http://user@127.0.0.1:3000", BAD_URL),
        ("http://127.0.0.1:3000/../x", BAD_URL),
        ("http://127.0.0.1:3000/?x=1", BAD_URL),
        ("http://127.0.0.1:3000/git//x", BAD_URL),
        ("ssh://127.0.0.1", BAD_URL),
        ("http://127.0.0.1:3000 ", BAD_URL),
        ("", BAD_URL),
    ];
    for (url, expected) in refusals {
        refused(draft_with_url(ForgeKind::Jeryu, url).seal(), expected);
    }
}

#[test]
fn base_url_policy_is_exact_per_forge_kind() {
    let cases = [
        (ForgeKind::Jeryu, "http://[::1]:8443/", None),
        (ForgeKind::Jeryu, "https://localhost", None),
        (ForgeKind::Jeryu, "http://localhost:80/jeryu", None),
        (ForgeKind::Github, "http://api.github.com", Some(NOT_HTTPS)),
        (ForgeKind::Github, "https://api.github.com", None),
        (ForgeKind::Gitlab, "http://127.0.0.1:8080", None),
        (ForgeKind::Gitlab, "http://gitlab.example", Some(NOT_HTTPS)),
        (ForgeKind::LocalBare, "file:///srv/forge/repo.git", None),
        (ForgeKind::LocalBare, "file:///srv/../etc", Some(BAD_URL)),
        (ForgeKind::LocalBare, "file://srv/repo.git", Some(BAD_URL)),
        (ForgeKind::LocalBare, "https://127.0.0.1", Some(BAD_URL)),
    ];
    for (kind, url, expected) in cases {
        let outcome = draft_with_url(kind, url).seal().err().map(|e| e.code());
        assert_eq!(outcome, expected, "{kind:?} {url:?}");
    }
}

#[test]
fn profile_refuses_duplicate_and_unsorted_capabilities() {
    let mut duplicate = draft("alpha", 1);
    duplicate.capabilities = vec![ForgeCapability::ProtectedRefs; 2];
    refused(duplicate.seal(), CAP_DUPLICATE);
    let mut unsorted = draft("alpha", 1);
    unsorted.capabilities = vec![ForgeCapability::ReadBack, ForgeCapability::ProtectedRefs];
    refused(unsorted.seal(), "FORGE_PROFILE_CAPABILITY_UNSORTED");
    let mut document = value_of(&sealed("alpha", 1));
    document["capabilities"] = json!(["read_back", "read_back"]);
    refused(decode_profile(&document), CAP_DUPLICATE);
    let mut empty = draft("alpha", 1);
    empty.capabilities.clear();
    let empty = empty.seal().unwrap();
    refused(empty.integration_subject(), LACKS_INTEGRATION);
}

#[test]
fn profile_refuses_tampered_content_and_invalid_scalars() {
    let mut tampered = sealed("alpha", 1);
    tampered.base_url = "http://[::1]:3000".to_owned();
    refused(tampered.validate(), PROFILE_MISMATCH);
    refused(decode_profile(&value_of(&tampered)), PROFILE_MISMATCH);

    let mut zero = draft("alpha", 0);
    refused(zero.clone().seal(), "INVALID_FORGE_PROFILE_GENERATION");
    zero.generation = 9_007_199_254_740_992;
    refused(zero.seal(), "INVALID_FORGE_PROFILE_GENERATION");
    for activator in ["", "op er", &"x".repeat(129), "op\u{200b}"] {
        let mut profile = draft("alpha", 1);
        profile.activated_by = activator.to_owned();
        refused(profile.seal(), "INVALID_FORGE_PROFILE_ACTIVATOR");
    }
    let mut late = draft("alpha", 1);
    late.activated_at_unix_ms = 9_007_199_254_740_992;
    refused(late.seal(), "INVALID_FORGE_PROFILE_TIME");
    let mut schema = draft("alpha", 1);
    schema.schema_version = "v1".to_owned();
    refused(schema.seal(), "UNSUPPORTED_FORGE_PROFILE_SCHEMA");
}

#[test]
fn registry_supersedes_the_active_generation_atomically() {
    let mut registry = ForgeProfileRegistry::default();
    let first = sealed("alpha", 1);
    let second = sealed("alpha", 2);
    assert_eq!(registry.activate(first.clone()).unwrap(), None);
    assert_eq!(registry.active(&repository("alpha")), Some(&first));
    assert_eq!(registry.activate(second.clone()).unwrap(), Some(first));
    assert_eq!(registry.active(&repository("alpha")), Some(&second));
    assert_eq!(registry.active_profiles().count(), 1);

    let other = sealed("beta", 7);
    assert_eq!(registry.activate(other.clone()).unwrap(), None);
    assert_eq!(registry.active(&repository("beta")), Some(&other));
    assert_eq!(registry.active(&repository("alpha")), Some(&second));
    assert_eq!(registry.active_profiles().count(), 2);
    assert_eq!(registry.active(&repository("gamma")), None);
}

#[test]
fn registry_refuses_replay_regression_and_a_second_profile_without_mutation() {
    let mut registry = ForgeProfileRegistry::default();
    let active = sealed("alpha", 2);
    registry.activate(active.clone()).unwrap();
    let snapshot = registry.clone();

    refused(registry.activate(active.clone()), REPLAY);
    let rival = draft_with_url(ForgeKind::Jeryu, "http://[::1]:3000");
    let rival = PrimaryForgeProfileV1 {
        generation: 2,
        ..rival
    }
    .seal()
    .unwrap();
    assert_ne!(rival.digest, active.digest);
    refused(registry.activate(rival), REPLAY);
    refused(registry.activate(sealed("alpha", 1)), REGRESSION);
    let mut tampered = sealed("alpha", 3);
    tampered.activated_by = "impostor".to_owned();
    refused(registry.activate(tampered), PROFILE_MISMATCH);

    assert_eq!(registry, snapshot);
    assert_eq!(registry.active_profiles().count(), 1);
    assert_eq!(registry.active(&repository("alpha")), Some(&active));
}

#[test]
fn replication_intent_round_trips_and_resolves_only_an_active_source() {
    let source = sealed("alpha", 1);
    let destination = sealed("mirror", 1);
    let intent = intent(&source, &destination).seal().unwrap();
    let bytes = canonical_json(&intent).unwrap();
    assert_eq!(decode_replication_intent(&bytes).unwrap(), intent);
    let preimage = json!({
        "schema_version": "v1alpha1",
        "intent_kind": "mirror",
        "source_profile_id": source.profile_id().to_string(),
        "destination_profile_id": destination.profile_id().to_string(),
        "refs": ["refs/heads/bullet/candidate/abc", "refs/tags/v1.0.0"],
    });
    assert_eq!(
        intent.digest,
        hash_canonical(REPLICATION_INTENT_DIGEST_DOMAIN, &preimage).unwrap()
    );

    let mut registry = ForgeProfileRegistry::default();
    refused(registry.replication_source(&intent), NO_SOURCE);
    registry.activate(source.clone()).unwrap();
    registry.activate(destination.clone()).unwrap();
    assert_eq!(registry.replication_source(&intent).unwrap(), &source);
    registry.activate(sealed("alpha", 2)).unwrap();
    refused(registry.replication_source(&intent), NO_SOURCE);
    let mut tampered = intent.clone();
    tampered.refs.push("refs/tags/v2.0.0".to_owned());
    refused(
        registry.replication_source(&tampered),
        "REPLICATION_INTENT_DIGEST_MISMATCH",
    );
}

#[test]
fn replication_intent_cannot_be_presented_as_integration() {
    let source = sealed("alpha", 1);
    let intent = intent_with_refs(&source, &["refs/heads/main"])
        .seal()
        .unwrap();
    refused(intent.integration_subject(), NOT_INTEGRATION);

    let smuggled = [
        (
            "integration_subject",
            json!({ "target_ref": "refs/heads/main", "candidate": "sha1:aa" }),
        ),
        ("target_ref", json!("refs/heads/main")),
        ("proof_root", json!("ipr_0000")),
        ("intent_kind", json!("integration")),
    ];
    for (member, value) in smuggled {
        let mut document = value_of(&intent);
        document[member] = value;
        refused(decode_intent(&document), SCHEMA_INVALID);
    }
    refused(decode_profile(&value_of(&intent)), SCHEMA_INVALID);

    let binding = source.integration_subject().unwrap();
    assert_eq!(binding.repository_id, repository("alpha"));
    assert_eq!(binding.profile_id, source.profile_id());
    assert_eq!(binding.generation, 1);
    let mut mirror_only = draft("mirror", 1);
    mirror_only.capabilities = vec![ForgeCapability::ExpectedOldOid, ForgeCapability::ReadBack];
    refused(
        mirror_only.seal().unwrap().integration_subject(),
        LACKS_INTEGRATION,
    );
}

#[test]
fn replication_intent_refuses_self_target_and_hostile_refs() {
    let source = sealed("alpha", 1);
    let mut own = intent(&source, &source);
    own.refs = vec!["refs/heads/x".to_owned()];
    refused(own.seal(), "REPLICATION_INTENT_SELF_TARGET");
    refused(intent_with_refs(&source, &[]).seal(), BAD_REF);
    let oversized: Vec<String> = (0..1025)
        .map(|index| format!("refs/heads/r{index:04}"))
        .collect();
    let oversized: Vec<&str> = oversized.iter().map(String::as_str).collect();
    refused(intent_with_refs(&source, &oversized).seal(), BAD_REF);

    let hostile = [
        "HEAD",
        "refs/heads/../main",
        "refs/heads/main.lock",
        "refs/heads/*",
        "refs/",
        "refs/heads//x",
        "refs/heads/.hidden",
        "refs/heads/a@{1}",
        "refs/heads/main.",
        " refs/heads/x",
        "refs/heads/\u{fc}ber",
        "refs/heads/x\n",
    ];
    for name in hostile {
        refused(intent_with_refs(&source, &[name]).seal(), BAD_REF);
    }
    refused(
        intent_with_refs(&source, &["refs/heads/a", "refs/heads/a"]).seal(),
        "REPLICATION_REF_DUPLICATE",
    );
    refused(
        intent_with_refs(&source, &["refs/heads/b", "refs/heads/a"]).seal(),
        "REPLICATION_REFS_UNSORTED",
    );

    let intent = intent(&source, &sealed("mirror", 1)).seal().unwrap();
    let mut wrong_prefix = value_of(&intent);
    wrong_prefix["source_profile_id"] = json!(format!("prf_{}", source.digest.to_hex()));
    refused(decode_intent(&wrong_prefix), SCHEMA_INVALID);
    let mut short_hex = value_of(&intent);
    short_hex["destination_profile_id"] = json!("fpf_abc");
    refused(decode_intent(&short_hex), SCHEMA_INVALID);
    let duplicate_member = br#"{"refs":["refs/heads/a"],"refs":["refs/heads/b"]}"#;
    refused(
        decode_replication_intent(duplicate_member),
        "DUPLICATE_JSON_KEY",
    );
}
