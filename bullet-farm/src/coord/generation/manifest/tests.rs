use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
};

use super::types::{ByteRange, RecoveryArtifacts};
use super::*;
use crate::coord::model::FrozenClaimSubject;

fn claim(fill: char) -> String {
    format!("clm_{}", fill.to_string().repeat(64))
}

fn artifact(
    path: &str,
    length: u64,
    record_count: u64,
    ends_with_lf: bool,
    fill: u8,
) -> ArtifactBinding {
    ArtifactBinding::new(
        RelativeArtifactPath::parse(path).unwrap(),
        length,
        Some(record_count),
        ends_with_lf,
        Sha256Digest::for_bytes(&vec![fill; length as usize]),
    )
    .unwrap()
}

pub(super) fn body() -> GenerationManifestBody {
    create_body(CreateBodyInput {
        recovery_operator: "local-orchestrator".to_owned(),
        recovery_policy_sha256: Sha256Digest::for_bytes(b"policy"),
        operator_decision_sha256: Sha256Digest::for_bytes(b"decision"),
        replay_contract_version: 1,
        replay_contract_sha256: Sha256Digest::for_bytes(b"replay"),
        bootstrap_commit_oid: "a".repeat(40),
        bootstrap_paths: vec!["src/coord".to_owned()],
        legacy_source_device: 1,
        legacy_source_inode: 2,
        parent_generation: "legacy-v1".to_owned(),
        incident_at_unix_ms: 10,
        recovered_at_unix_ms: 20,
        trusted_record_count: 4_792,
        trusted_projection_inventory: super::inventory::trusted_inventory_fixture(),
        discarded_range: ByteRange {
            start_inclusive: 10,
            end_exclusive: 100,
        },
        ambiguous_tail_range: ByteRange {
            start_inclusive: 10,
            end_exclusive: 79,
        },
        ambiguous_tail_sha256: Sha256Digest::for_bytes(b"ambiguous"),
        artifacts: RecoveryArtifacts {
            trusted_prefix: artifact(TRUSTED_PREFIX_PATH, 10, 4_792, true, b'a'),
            interrupted_capture: artifact(INTERRUPTED_CAPTURE_PATH, 79, 4_792, false, b'b'),
            tainted_generation: artifact(TAINTED_GENERATION_PATH, 90, 4_800, true, b'c'),
            frozen_live_source: artifact(FROZEN_LIVE_SOURCE_PATH, 100, 4_900, true, b'd'),
        },
        trusted_state_blake3: format!("blake3:{}", "1".repeat(64)),
        frozen_claims: vec![frozen('b'), frozen('a')],
        post_prefix_inventory_blake3: format!("blake3:{}", "2".repeat(64)),
    })
    .unwrap()
}

fn genesis_body() -> GenerationManifestBody {
    create_genesis_body(CreateGenesisBodyInput {
        created_at_unix_ms: 30,
        operator: "local-orchestrator".to_owned(),
        policy_sha256: Sha256Digest::for_bytes(b"genesis-policy"),
        replay_contract_version: 1,
        replay_contract_sha256: Sha256Digest::for_bytes(b"replay"),
        bootstrap_commit_oid: "b".repeat(40),
        bootstrap_paths: vec!["src/coord/store".to_owned(), "src/coord".to_owned()],
    })
    .unwrap()
}

#[test]
fn generation_identity_is_canonical_sorted_and_sensitive() {
    let manifest = GenerationManifest::from_body(body()).unwrap();
    assert_eq!(
        manifest
            .body
            .recovery()
            .unwrap()
            .frozen_claims
            .iter()
            .map(|claim| claim.claim_id.as_str())
            .collect::<Vec<_>>(),
        vec![claim('a'), claim('b')]
    );
    let bytes = manifest.canonical_bytes().unwrap();
    assert_eq!(bytes.last(), Some(&b'\n'));
    assert_eq!(
        GenerationManifest::decode_canonical(&bytes).unwrap(),
        manifest
    );
    assert_eq!(manifest.generation_id().as_str().len(), 68);

    let mut changed = body();
    recovery_mut(&mut changed).recovered_at_unix_ms += 1;
    assert_ne!(generation_id(&changed).unwrap(), *manifest.generation_id());

    let mut pretty: serde_json::Value = serde_json::from_slice(&bytes[..bytes.len() - 1]).unwrap();
    pretty["unknown"] = serde_json::json!(true);
    let mut pretty = serde_json::to_vec_pretty(&pretty).unwrap();
    pretty.push(b'\n');
    assert!(GenerationManifest::decode_canonical(&pretty).is_err());
}

#[test]
fn body_refuses_duplicates_bad_lineage_and_unsafe_paths() {
    let duplicate = create_body(CreateBodyInput {
        frozen_claims: vec![frozen('a'), frozen('a')],
        ..input_from(body())
    });
    assert!(duplicate.is_err());

    let mut lineage = body();
    recovery_mut(&mut lineage).discarded_range.end_exclusive -= 1;
    assert!(lineage.validate().is_err());

    let mut unbound = body();
    recovery_mut(&mut unbound)
        .artifacts
        .interrupted_capture
        .record_count = None;
    assert!(unbound.validate().is_err());

    let mut unsupported_parent = body();
    recovery_mut(&mut unsupported_parent).parent_generation = format!("gen_{}", "3".repeat(64));
    assert!(unsupported_parent.validate().is_err());

    for path in ["/archive/x", "archive/../x", "archive\\x", "Archive/x"] {
        assert!(
            RelativeArtifactPath::parse(path).is_err(),
            "accepted {path}"
        );
    }
}

#[test]
fn genesis_is_closed_canonical_and_identity_bound() {
    let body = genesis_body();
    assert!(body.recovery().is_err());
    let manifest = GenerationManifest::from_body(body.clone()).unwrap();
    let bytes = manifest.canonical_bytes().unwrap();
    assert_eq!(
        GenerationManifest::decode_canonical(&bytes).unwrap(),
        manifest
    );

    let value = serde_json::to_value(&body).unwrap();
    let object = value.as_object().unwrap();
    assert_eq!(object.get("kind").unwrap(), "GENESIS");
    for forbidden in [
        "artifacts",
        "parent_generation",
        "frozen_claims",
        "lineage",
        "implicit_adoptions",
        "reason",
    ] {
        assert!(!object.contains_key(forbidden));
    }

    let mut changed = genesis_body();
    let GenerationManifestBody::Genesis(changed) = &mut changed else {
        unreachable!();
    };
    changed.created_at_unix_ms += 1;
    assert_ne!(
        generation_id(&changed_body(changed.clone())).unwrap(),
        *manifest.generation_id()
    );

    let mut polluted = value;
    polluted
        .as_object_mut()
        .unwrap()
        .insert("artifacts".to_owned(), serde_json::json!({}));
    assert!(serde_json::from_value::<GenerationManifestBody>(polluted).is_err());
}

#[test]
fn every_closed_body_field_is_required() {
    let recovery = serde_json::to_value(body()).unwrap();
    assert_every_top_level_field_required(&recovery);
    for range in ["discarded_range", "ambiguous_tail_range"] {
        for field in ["start_inclusive", "end_exclusive"] {
            assert_body_rejects(remove_object_field(&recovery, &[range], field));
        }
    }
    for artifact in [
        "trusted_prefix",
        "interrupted_capture",
        "tainted_generation",
        "frozen_live_source",
    ] {
        assert_body_rejects(remove_object_field(&recovery, &["artifacts"], artifact));
        for field in [
            "relative_path",
            "byte_length",
            "record_count",
            "ends_with_lf",
            "sha256",
        ] {
            assert_body_rejects(remove_object_field(
                &recovery,
                &["artifacts", artifact],
                field,
            ));
        }
    }
    for field in ["claim_id", "claim_blake3"] {
        let mut missing = recovery.clone();
        missing["frozen_claims"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert_body_rejects(missing);
    }

    let genesis = serde_json::to_value(genesis_body()).unwrap();
    assert_every_top_level_field_required(&genesis);
}

#[test]
fn every_open_provenance_field_changes_generation_identity() {
    let base = generation_id(&body()).unwrap();
    let candidates = [
        changed_recovery(|body| body.recovery_operator = "other-operator".to_owned()),
        changed_recovery(|body| {
            body.recovery_policy_sha256 = Sha256Digest::for_bytes(b"other-policy")
        }),
        changed_recovery(|body| {
            body.operator_decision_sha256 = Sha256Digest::for_bytes(b"other-decision")
        }),
        changed_recovery(|body| {
            body.replay_contract_sha256 = Sha256Digest::for_bytes(b"other-replay")
        }),
        changed_recovery(|body| body.bootstrap_commit_oid = "c".repeat(40)),
        changed_recovery(|body| body.bootstrap_paths = vec!["src/coord/model.rs".to_owned()]),
        changed_recovery(|body| body.legacy_source_device += 1),
        changed_recovery(|body| body.legacy_source_inode += 1),
        changed_recovery(|body| body.incident_at_unix_ms += 1),
        changed_recovery(|body| body.recovered_at_unix_ms += 1),
        changed_recovery(|body| {
            body.trusted_record_count += 1;
            body.trusted_projection_inventory.record_kinds.heartbeat += 1;
            body.artifacts.trusted_prefix.record_count = Some(body.trusted_record_count);
            body.artifacts.interrupted_capture.record_count = Some(body.trusted_record_count);
        }),
        changed_recovery(|body| {
            body.artifacts.trusted_prefix.byte_length = 11;
            body.artifacts.trusted_prefix.sha256 = Sha256Digest::for_bytes(&[b'a'; 11]);
            body.artifacts.interrupted_capture.byte_length = 80;
            body.artifacts.interrupted_capture.sha256 = Sha256Digest::for_bytes(&[b'b'; 80]);
            body.discarded_range.start_inclusive = 11;
            body.ambiguous_tail_range.start_inclusive = 11;
            body.ambiguous_tail_range.end_exclusive = 80;
        }),
        changed_recovery(|body| {
            body.ambiguous_tail_sha256 = Sha256Digest::for_bytes(b"other-ambiguous")
        }),
        changed_recovery(|body| {
            body.artifacts.tainted_generation.sha256 = Sha256Digest::for_bytes(b"other-tainted")
        }),
        changed_recovery(|body| body.artifacts.tainted_generation.record_count = Some(4_801)),
        changed_recovery(|body| body.trusted_state_blake3 = format!("blake3:{}", "4".repeat(64))),
        changed_recovery(|body| {
            body.frozen_claims[0].claim_blake3 = format!("blake3:{}", "5".repeat(64))
        }),
        changed_recovery(|body| {
            body.post_prefix_inventory_blake3 = format!("blake3:{}", "6".repeat(64))
        }),
    ];
    for candidate in candidates {
        assert_ne!(generation_id(&candidate).unwrap(), base);
    }

    let genesis_base = generation_id(&genesis_body()).unwrap();
    let genesis_candidates = [
        changed_genesis(|body| body.created_at_unix_ms += 1),
        changed_genesis(|body| body.operator = "other-operator".to_owned()),
        changed_genesis(|body| body.policy_sha256 = Sha256Digest::for_bytes(b"other-policy")),
        changed_genesis(|body| {
            body.replay_contract_sha256 = Sha256Digest::for_bytes(b"other-replay")
        }),
        changed_genesis(|body| body.bootstrap_commit_oid = "d".repeat(40)),
        changed_genesis(|body| body.bootstrap_paths = vec!["src/coord/model.rs".to_owned()]),
    ];
    for candidate in genesis_candidates {
        assert_ne!(generation_id(&candidate).unwrap(), genesis_base);
    }
    assert_ne!(base, genesis_base);
}

#[test]
fn current_pointer_binds_the_exact_manifest() {
    let manifest = GenerationManifest::from_body(body()).unwrap();
    let pointer = CurrentPointer::for_manifest(&manifest).unwrap();
    let bytes = pointer.canonical_bytes().unwrap();
    let decoded = CurrentPointer::decode_canonical(&bytes).unwrap();
    decoded.verify_manifest(&manifest).unwrap();

    let changed = GenerationManifest::from_body({
        let mut body = body();
        recovery_mut(&mut body).recovered_at_unix_ms += 1;
        body
    })
    .unwrap();
    assert!(decoded.verify_manifest(&changed).is_err());
}

#[test]
#[cfg(target_os = "linux")]
fn current_load_is_creation_free_and_descriptor_safe() {
    let root = tempfile::tempdir().unwrap();
    let coord = root.path().join("coord");
    fs::create_dir(&coord).unwrap();
    fs::set_permissions(&coord, fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(load_current(&coord).unwrap(), None);
    assert!(!coord.join(CURRENT_FILE).exists());

    let manifest = GenerationManifest::from_body(genesis_body()).unwrap();
    let pointer = CurrentPointer::for_manifest(&manifest).unwrap();
    fs::write(coord.join(CURRENT_FILE), pointer.canonical_bytes().unwrap()).unwrap();
    fs::set_permissions(coord.join(CURRENT_FILE), fs::Permissions::from_mode(0o400)).unwrap();
    assert_eq!(load_current(&coord).unwrap(), Some(pointer));

    fs::set_permissions(coord.join(CURRENT_FILE), fs::Permissions::from_mode(0o600)).unwrap();
    assert!(load_current(&coord).is_err());
    fs::remove_file(coord.join(CURRENT_FILE)).unwrap();
    symlink("missing", coord.join(CURRENT_FILE)).unwrap();
    assert!(load_current(&coord).is_err());
}

#[test]
#[cfg(target_os = "linux")]
fn manifest_and_artifact_admission_is_identity_exact() {
    let root = tempfile::tempdir().unwrap();
    let generation = root.path().join("generation");
    fs::create_dir(&generation).unwrap();
    fs::set_permissions(&generation, fs::Permissions::from_mode(0o700)).unwrap();
    fs::create_dir(generation.join("archive")).unwrap();
    fs::set_permissions(
        generation.join("archive"),
        fs::Permissions::from_mode(0o700),
    )
    .unwrap();

    let manifest = GenerationManifest::from_body(body()).unwrap();
    write_generation_manifest(&generation, &manifest).unwrap();
    assert_eq!(
        load_and_verify(&generation, manifest.generation_id()).unwrap(),
        manifest
    );

    let artifact = &manifest.body.recovery().unwrap().artifacts.trusted_prefix;
    let artifact_path = generation.join(artifact.relative_path.as_str());
    fs::write(&artifact_path, vec![b'a'; artifact.byte_length as usize]).unwrap();
    fs::set_permissions(&artifact_path, fs::Permissions::from_mode(0o400)).unwrap();
    verify_artifact(&generation, artifact, &artifact.relative_path).unwrap();

    fs::set_permissions(&artifact_path, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(verify_artifact(&generation, artifact, &artifact.relative_path).is_err());
    fs::set_permissions(&artifact_path, fs::Permissions::from_mode(0o400)).unwrap();
    fs::hard_link(&artifact_path, generation.join("extra-link")).unwrap();
    assert!(verify_artifact(&generation, artifact, &artifact.relative_path).is_err());
}

#[test]
#[cfg(target_os = "linux")]
fn symlinked_generation_and_artifact_refuse() {
    let root = tempfile::tempdir().unwrap();
    let real = root.path().join("real");
    fs::create_dir(&real).unwrap();
    fs::set_permissions(&real, fs::Permissions::from_mode(0o700)).unwrap();
    let linked = root.path().join("linked");
    symlink(&real, &linked).unwrap();
    let manifest = GenerationManifest::from_body(body()).unwrap();
    assert!(write_generation_manifest(&linked, &manifest).is_err());
}

fn input_from(body: GenerationManifestBody) -> CreateBodyInput {
    let GenerationManifestBody::RecoveryBaseline(body) = body else {
        panic!("expected recovery body");
    };
    CreateBodyInput {
        recovery_operator: body.recovery_operator,
        recovery_policy_sha256: body.recovery_policy_sha256,
        operator_decision_sha256: body.operator_decision_sha256,
        replay_contract_version: body.replay_contract_version,
        replay_contract_sha256: body.replay_contract_sha256,
        bootstrap_commit_oid: body.bootstrap_commit_oid,
        bootstrap_paths: body.bootstrap_paths,
        legacy_source_device: body.legacy_source_device,
        legacy_source_inode: body.legacy_source_inode,
        parent_generation: body.parent_generation,
        incident_at_unix_ms: body.incident_at_unix_ms,
        recovered_at_unix_ms: body.recovered_at_unix_ms,
        trusted_record_count: body.trusted_record_count,
        trusted_projection_inventory: body.trusted_projection_inventory,
        discarded_range: body.discarded_range,
        ambiguous_tail_range: body.ambiguous_tail_range,
        ambiguous_tail_sha256: body.ambiguous_tail_sha256,
        artifacts: body.artifacts,
        trusted_state_blake3: body.trusted_state_blake3,
        frozen_claims: body.frozen_claims,
        post_prefix_inventory_blake3: body.post_prefix_inventory_blake3,
    }
}

fn frozen(fill: char) -> FrozenClaimSubject {
    FrozenClaimSubject {
        claim_id: claim(fill),
        claim_blake3: format!("blake3:{}", fill.to_string().repeat(64)),
    }
}

pub(super) fn recovery_mut(body: &mut GenerationManifestBody) -> &mut RecoveryManifestBody {
    let GenerationManifestBody::RecoveryBaseline(body) = body else {
        panic!("expected recovery body");
    };
    body
}

fn changed_body(body: GenesisManifestBody) -> GenerationManifestBody {
    GenerationManifestBody::Genesis(body)
}

fn changed_recovery(edit: impl FnOnce(&mut RecoveryManifestBody)) -> GenerationManifestBody {
    let mut candidate = body();
    edit(recovery_mut(&mut candidate));
    candidate
}

fn changed_genesis(edit: impl FnOnce(&mut GenesisManifestBody)) -> GenerationManifestBody {
    let mut candidate = genesis_body();
    let GenerationManifestBody::Genesis(body) = &mut candidate else {
        unreachable!();
    };
    edit(body);
    candidate
}

fn assert_every_top_level_field_required(value: &serde_json::Value) {
    for field in value.as_object().unwrap().keys() {
        assert_body_rejects(remove_object_field(value, &[], field));
    }
}

fn remove_object_field(
    value: &serde_json::Value,
    parents: &[&str],
    field: &str,
) -> serde_json::Value {
    let mut changed = value.clone();
    let mut subject = &mut changed;
    for parent in parents {
        subject = subject.get_mut(*parent).unwrap();
    }
    subject.as_object_mut().unwrap().remove(field).unwrap();
    changed
}

fn assert_body_rejects(value: serde_json::Value) {
    let admitted = serde_json::from_value::<GenerationManifestBody>(value).and_then(|body| {
        body.validate()
            .map(|()| body)
            .map_err(serde::de::Error::custom)
    });
    assert!(admitted.is_err());
}
