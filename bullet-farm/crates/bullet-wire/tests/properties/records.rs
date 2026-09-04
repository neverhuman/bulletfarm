use std::{collections::BTreeMap, fmt::Debug};

use bullet_wire::{
    AttemptId, Blake3Digest, CandidateId, CandidateProofRoot, ChangeId, CheckpointId,
    ComponentCandidateManifest, ComponentCandidateProofManifest, ComponentCheckpointManifest,
    ComponentIntegrationProofManifest, ComponentPreservationReceiptManifest, ContentId,
    EffectReceiptId, FORGE_PROFILE_SCHEMA_VERSION, ForgeCapability, ForgeKind, ForgeProfileId,
    GateId, GraphRevisionId, PlanRevisionId, PrimaryForgeProfileV1, ReplicationIntentKind,
    ReplicationIntentV1, RepoPath, RepositoryId, SCHEMA_VERSION, VariantId, WireError,
    WorkPackageId, canonical_json, decode_canonical, decode_unique_value, hash_canonical,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use super::support::{CASES, Rng, code, ctx};

const UNSUPPORTED_SCHEMA: &[&str] = &["UNSUPPORTED_SCHEMA"];
const UNSUPPORTED_FORGE_SCHEMA: &[&str] = &["UNSUPPORTED_FORGE_PROFILE_SCHEMA"];
const UNSUPPORTED_REPLICATION_SCHEMA: &[&str] = &["UNSUPPORTED_REPLICATION_INTENT_SCHEMA"];
const INVALID_DOCUMENT_SCHEMA: &[&str] = &["DOCUMENT_SCHEMA_INVALID"];

enum MutationOracle {
    Rebind(Value),
    Refuse(Value, &'static [&'static str]),
}

fn flip_hex(text: &str) -> String {
    const HEX: &[u8] = b"0123456789abcdef";
    let last = text.as_bytes()[text.len() - 1];
    let index = HEX
        .iter()
        .position(|candidate| *candidate == last)
        .expect("typed hex field");
    format!(
        "{}{}",
        &text[..text.len() - 1],
        HEX[(index + 1) % 16] as char
    )
}

fn schema_mutation(name: &str, field: &Value) -> MutationOracle {
    let codes = match name {
        "forge-profile" => UNSUPPORTED_FORGE_SCHEMA,
        "replication-intent" => UNSUPPORTED_REPLICATION_SCHEMA,
        _ => UNSUPPORTED_SCHEMA,
    };
    let value = match field {
        Value::Number(number) => Value::from(number.as_u64().expect("u64 schema") + 1),
        Value::String(text) => Value::from(format!("{text}x")),
        other => panic!("unexpected schema shape: {other:?}"),
    };
    MutationOracle::Refuse(value, codes)
}

fn valid_number_mutation(name: &str, key: &str, field: &Value) -> Value {
    let admitted = matches!(
        (name, key),
        ("candidate", "attempt_fence")
            | (
                "checkpoint",
                "fence" | "workspace_generation" | "journal_start" | "journal_end"
            )
            | ("preservation", "fence" | "journal_start" | "journal_end")
            | ("forge-profile", "generation" | "activated_at_unix_ms")
    );
    assert!(admitted, "unrouted numeric field {name}.{key}");
    Value::from(field.as_u64().expect("u64 identity field") + 1)
}

fn valid_string_mutation(name: &str, key: &str, text: &str) -> Value {
    match (name, key) {
        ("forge-profile", "forge_kind") if text == "github" => Value::from("gitlab"),
        ("forge-profile", "forge_kind") => Value::from("github"),
        ("forge-profile", "base_url") => Value::from(format!("{text}/x")),
        ("forge-profile", "activated_by")
        | ("preservation", "daemon_identity")
        | ("integration-proof", "observed_ref") => Value::from(format!("{text}x")),
        ("preservation", "external_destination") => Value::from(format!("{text}/x")),
        _ if text.len() >= 40 && text.ends_with(|value: char| value.is_ascii_hexdigit()) => {
            Value::from(flip_hex(text))
        }
        _ => panic!("unrouted string field {name}.{key}"),
    }
}

fn valid_array_mutation(name: &str, key: &str, items: &[Value]) -> Value {
    let admitted = matches!(
        (name, key),
        (
            "candidate",
            "parent_candidate_ids" | "granted_scope" | "actual_scope"
        ) | ("candidate-proof", "gate_ids")
            | (
                "integration-proof",
                "approval_digests" | "effect_receipt_ids"
            )
            | ("forge-profile", "capabilities")
            | ("replication-intent", "refs")
    );
    assert!(admitted, "unrouted array field {name}.{key}");
    assert!(!items.is_empty(), "mutation source {name}.{key} is empty");
    Value::Array(items[..items.len() - 1].to_vec())
}

fn mutation_for(name: &str, key: &str, field: &Value) -> MutationOracle {
    if key == "schema_version" {
        return schema_mutation(name, field);
    }
    if (name, key) == ("replication-intent", "intent_kind") {
        return MutationOracle::Refuse(Value::from("mirror-x"), INVALID_DOCUMENT_SCHEMA);
    }
    let value = match field {
        Value::Number(_) => valid_number_mutation(name, key, field),
        Value::String(text) => valid_string_mutation(name, key, text),
        Value::Array(items) => valid_array_mutation(name, key, items),
        other => panic!("unrouted member {name}.{key}: {other:?}"),
    };
    MutationOracle::Rebind(value)
}

fn check_record_identity<T, I>(
    name: &str,
    build: impl Fn(&mut Rng) -> T,
    identity: impl Fn(T) -> Result<I, WireError>,
) where
    T: Serialize + DeserializeOwned + Clone,
    I: PartialEq + Debug,
{
    for index in 0..CASES {
        let (mut rng, context) = (Rng::for_case(index), format!("{name} {}", ctx(index)));
        let record = build(&mut rng);
        let original =
            identity(record.clone()).unwrap_or_else(|error| panic!("{context}: {error}"));
        let bytes = canonical_json(&record).expect("generated record");
        let decoded: T =
            decode_canonical(&bytes).unwrap_or_else(|error| panic!("{context}: {error}"));
        assert_eq!(
            identity(decoded).expect("decoded identity"),
            original,
            "{context}: identity unstable across re-encoding"
        );
        let pretty = serde_json::to_string_pretty(&record).expect("pretty record");
        let loose = decode_unique_value(pretty.as_bytes()).expect("pretty loose JSON");
        assert_eq!(
            canonical_json(&loose).expect("pretty canonical JSON"),
            bytes,
            "{context}: pretty text canonicalizes differently"
        );
        let value = serde_json::to_value(&record).expect("record value");
        for (key, field) in value.as_object().expect("record object") {
            if key == "digest" {
                continue;
            }
            let oracle = mutation_for(name, key, field);
            let mut mutated = value.clone();
            mutated[key] = match &oracle {
                MutationOracle::Rebind(value) | MutationOracle::Refuse(value, _) => value.clone(),
            };
            let encoded = canonical_json(&mutated).expect("mutated record");
            let result = decode_canonical::<T>(&encoded).and_then(&identity);
            match (oracle, result) {
                (MutationOracle::Rebind(_), Ok(mutated_id)) => assert_ne!(
                    mutated_id, original,
                    "{context}: valid field {key} mutation is unbound"
                ),
                (MutationOracle::Rebind(_), Err(error)) => {
                    panic!("{context}: valid field {key} mutation refused: {error}")
                }
                (MutationOracle::Refuse(_, codes), Err(error)) => assert!(
                    codes.contains(&error.code()),
                    "{context}: field {key} refusal was {error}"
                ),
                (MutationOracle::Refuse(_, _), Ok(_)) => {
                    panic!("{context}: invalid field {key} mutation was admitted")
                }
            }
        }
    }
}

fn candidate(rng: &mut Rng) -> ComponentCandidateManifest {
    ComponentCandidateManifest {
        schema_version: SCHEMA_VERSION,
        repository_id: RepositoryId::from_digest(rng.digest()),
        change_id: ChangeId::from_digest(rng.digest()),
        producing_attempt_id: AttemptId::from_digest(rng.digest()),
        attempt_fence: 1 + rng.next_u64() % 1_000_000,
        work_package_id: WorkPackageId::from_digest(rng.digest()),
        variant_id: VariantId::from_digest(rng.digest()),
        plan_revision_id: PlanRevisionId::from_digest(rng.digest()),
        graph_revision_id: GraphRevisionId::from_digest(rng.digest()),
        base_checkpoint_id: CheckpointId::from_digest(rng.digest()),
        base_commit: rng.oid(),
        head_commit: rng.oid(),
        tree_oid: rng.oid(),
        patch_digest: rng.digest(),
        parent_candidate_ids: vec![CandidateId::from_digest(rng.digest())],
        granted_scope: vec![repo_path("src"), repo_path(&format!("d{}", rng.label()))],
        actual_scope: vec![repo_path(&format!("src/{}.rs", rng.label()))],
        context_capsule_id: ContentId::from_digest(rng.digest()),
        configuration_snapshot_id: ContentId::from_digest(rng.digest()),
        policy_snapshot_id: ContentId::from_digest(rng.digest()),
        routing_snapshot_id: ContentId::from_digest(rng.digest()),
        environment_digest: rng.digest(),
        toolchain_digest: rng.digest(),
    }
}

fn journal(rng: &mut Rng) -> (u64, u64) {
    let start = rng.next_u64() % 1_000_000;
    (start, start + 1 + rng.next_u64() % 1_000)
}

fn repo_path(raw: &str) -> RepoPath {
    let encoded = canonical_json(&raw).expect("path string");
    decode_canonical(&encoded).expect("generated repository path")
}

#[test]
fn candidate_manifest_identity_is_stable_and_field_sensitive() {
    check_record_identity("candidate", candidate, |record| record.candidate_id());
    let mut bad = candidate(&mut Rng::for_case(0));
    bad.attempt_fence = 0;
    assert_eq!(code(bad.candidate_id()), Err("INVALID_FENCE"));
    bad.attempt_fence = 1;
    bad.actual_scope.push(repo_path("etc/passwd"));
    assert_eq!(code(bad.candidate_id()), Err("ACTUAL_SCOPE_EXCEEDS_GRANT"));
}

#[test]
fn checkpoint_manifest_identity_is_stable_and_field_sensitive() {
    let build = |rng: &mut Rng| {
        let (journal_start, journal_end) = journal(rng);
        ComponentCheckpointManifest {
            schema_version: SCHEMA_VERSION,
            repository_id: RepositoryId::from_digest(rng.digest()),
            attempt_id: AttemptId::from_digest(rng.digest()),
            fence: 1 + rng.next_u64() % 1_000_000,
            workspace_generation: rng.next_u64() % 1_000_000,
            base_commit: rng.oid(),
            head_commit: rng.oid(),
            tree_oid: rng.oid(),
            journal_start,
            journal_end,
            journal_digest: rng.digest(),
            cas_root: rng.digest(),
        }
    };
    check_record_identity("checkpoint", build, |record| record.checkpoint_id());
    let mut bad = build(&mut Rng::for_case(0));
    bad.journal_end = bad.journal_start - 1;
    assert_eq!(code(bad.checkpoint_id()), Err("INVALID_CHECKPOINT"));
}

#[test]
fn preservation_receipt_identity_is_stable_and_field_sensitive() {
    let build = |rng: &mut Rng| {
        let (journal_start, journal_end) = journal(rng);
        ComponentPreservationReceiptManifest {
            schema_version: SCHEMA_VERSION,
            attempt_id: AttemptId::from_digest(rng.digest()),
            fence: 1 + rng.next_u64() % 1_000_000,
            workspace_nonce: rng.digest(),
            tree_oid: rng.oid(),
            dirty_manifest_digest: rng.digest(),
            untracked_manifest_digest: rng.digest(),
            journal_start,
            journal_end,
            bundle_or_cas_digest: rng.digest(),
            external_destination: format!("s3://{}/bundle", rng.label()),
            external_destination_digest: rng.digest(),
            daemon_identity: format!("bullet-gitd:{}", rng.label()),
            signature_digest: rng.digest(),
        }
    };
    check_record_identity("preservation", build, |record| record.receipt_digest());
    let mut bad = build(&mut Rng::for_case(0));
    bad.fence = 0;
    assert_eq!(
        code(bad.receipt_digest()),
        Err("INVALID_PRESERVATION_RECEIPT")
    );
}

#[test]
fn candidate_proof_manifest_identity_is_stable_and_field_sensitive() {
    let build = |rng: &mut Rng| ComponentCandidateProofManifest {
        schema_version: SCHEMA_VERSION,
        candidate_id: CandidateId::from_digest(rng.digest()),
        checkpoint_id: CheckpointId::from_digest(rng.digest()),
        journal_digest: rng.digest(),
        cas_root: rng.digest(),
        preservation_receipt_digest: rng.digest(),
        gate_ids: (0..1 + rng.below(3))
            .map(|_| GateId::from_digest(rng.digest()))
            .collect(),
    };
    check_record_identity("candidate-proof", build, |record| record.proof_root());
    let mut bad = build(&mut Rng::for_case(0));
    bad.schema_version = 0;
    assert_eq!(code(bad.proof_root()), Err("UNSUPPORTED_SCHEMA"));
}

#[test]
fn integration_proof_manifest_identity_is_stable_and_field_sensitive() {
    let build = |rng: &mut Rng| ComponentIntegrationProofManifest {
        schema_version: SCHEMA_VERSION,
        candidate_proof_root: CandidateProofRoot::from_digest(rng.digest()),
        selection_digest: rng.digest(),
        approval_digests: (0..1 + rng.below(3)).map(|_| rng.digest()).collect(),
        effect_receipt_ids: (0..1 + rng.below(3))
            .map(|_| EffectReceiptId::from_digest(rng.digest()))
            .collect(),
        integrated_commit: rng.oid(),
        observed_ref: format!("refs/heads/{}", rng.label()),
        observation_digest: rng.digest(),
    };
    check_record_identity("integration-proof", build, |record| record.proof_root());
    let mut bad = build(&mut Rng::for_case(0));
    bad.schema_version = SCHEMA_VERSION + 1;
    assert_eq!(code(bad.proof_root()), Err("UNSUPPORTED_SCHEMA"));
}

fn profile(rng: &mut Rng) -> PrimaryForgeProfileV1 {
    const KINDS: [ForgeKind; 3] = [ForgeKind::Jeryu, ForgeKind::Github, ForgeKind::Gitlab];
    const CAPABILITIES: [ForgeCapability; 6] = [
        ForgeCapability::ExactShaChecks,
        ForgeCapability::ExpectedOldOid,
        ForgeCapability::IntegrationSubject,
        ForgeCapability::MergeGroups,
        ForgeCapability::ProtectedRefs,
        ForgeCapability::PullRequests,
    ];
    let forge_kind = KINDS[rng.below(3)];
    let base_url = if forge_kind == ForgeKind::Jeryu || rng.coin() {
        format!("http://127.0.0.1:{}", 1 + rng.below(65_535))
    } else {
        format!("https://forge.example/{}", rng.label())
    };
    let mut capabilities: Vec<_> = CAPABILITIES.into_iter().filter(|_| rng.coin()).collect();
    capabilities.push(ForgeCapability::ReadBack);
    PrimaryForgeProfileV1 {
        schema_version: FORGE_PROFILE_SCHEMA_VERSION.to_owned(),
        repository_id: RepositoryId::from_digest(rng.digest()),
        forge_kind,
        base_url,
        capabilities,
        generation: 1 + rng.next_u64() % (1 << 40),
        activated_by: format!("operator:{}", rng.label()),
        activated_at_unix_ms: rng.next_u64() % (1 << 50),
        digest: Blake3Digest::from_bytes([0; 32]),
    }
}

#[test]
fn primary_forge_profile_identity_is_stable_and_field_sensitive() {
    check_record_identity("forge-profile", profile, |record| {
        record.seal().map(|sealed| sealed.profile_id())
    });
    let sealed = profile(&mut Rng::for_case(0))
        .seal()
        .expect("sealed profile");
    assert_eq!(
        sealed.profile_id(),
        ForgeProfileId::from_digest(sealed.expected_digest().expect("profile digest"))
    );
    let mut tampered = sealed.clone();
    tampered.digest = Blake3Digest::from_bytes([7; 32]);
    assert_eq!(
        code(tampered.validate()),
        Err("FORGE_PROFILE_DIGEST_MISMATCH")
    );
    tampered = sealed;
    tampered.generation = 0;
    assert_eq!(
        code(tampered.seal()),
        Err("INVALID_FORGE_PROFILE_GENERATION")
    );
}

#[test]
fn replication_intent_identity_is_stable_and_field_sensitive() {
    let build = |rng: &mut Rng| {
        let mut refs: Vec<_> = (0..2 + rng.below(4))
            .map(|index| format!("refs/heads/{}-{index}", rng.label()))
            .collect();
        refs.sort();
        ReplicationIntentV1 {
            schema_version: FORGE_PROFILE_SCHEMA_VERSION.to_owned(),
            intent_kind: ReplicationIntentKind::Mirror,
            source_profile_id: ForgeProfileId::from_digest(rng.digest()),
            destination_profile_id: ForgeProfileId::from_digest(rng.digest()),
            refs,
            digest: Blake3Digest::from_bytes([0; 32]),
        }
    };
    check_record_identity("replication-intent", build, |record| {
        record.seal().map(|sealed| sealed.digest)
    });
    let sealed = build(&mut Rng::for_case(0)).seal().expect("sealed intent");
    let mut tampered = sealed.clone();
    tampered
        .refs
        .last_mut()
        .expect("nonempty sealed refs")
        .push_str("-x");
    assert_eq!(
        code(tampered.validate()),
        Err("REPLICATION_INTENT_DIGEST_MISMATCH")
    );
    tampered = sealed;
    tampered.destination_profile_id = tampered.source_profile_id.clone();
    assert_eq!(code(tampered.seal()), Err("REPLICATION_INTENT_SELF_TARGET"));
    let empty: BTreeMap<String, String> = BTreeMap::new();
    assert!(hash_canonical("test.empty", &empty).is_ok());
}
