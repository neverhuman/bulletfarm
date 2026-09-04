use std::{
    fs::{self, OpenOptions},
    os::unix::fs::{MetadataExt, PermissionsExt, symlink},
};

use tempfile::{TempDir, tempdir};

use super::{
    ContentExpectation, RecoveryInput, RecoveryState, SourceExpectation, authority, exchange,
    linux, platform_fs as io, verifier,
};
use crate::coord::{
    CoordError,
    generation::manifest::{
        ArtifactBinding, CreateBodyInput, GenerationManifest, RelativeArtifactPath, Sha256Digest,
        TrustedClaimOutcomeCounts, TrustedProjectionInventory, TrustedRecordKindCounts,
        create_body,
    },
    model::{ClaimState, FrozenClaimSubject, Record},
};

fn mode(path: &std::path::Path) -> u32 {
    fs::symlink_metadata(path).unwrap().permissions().mode() & 0o7777
}

fn binding(path: &str, bytes: &[u8], records: u64, lf: bool) -> ArtifactBinding {
    ArtifactBinding::new(
        RelativeArtifactPath::parse(path).unwrap(),
        bytes.len() as u64,
        Some(records),
        lf,
        Sha256Digest::for_bytes(bytes),
    )
    .unwrap()
}

struct RecoveryFixture {
    root: TempDir,
    input: RecoveryInput,
    manifest: GenerationManifest,
}

fn write_private(path: &std::path::Path, bytes: &[u8], mode: u32) {
    fs::write(path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

fn canonical_line(record: &Record) -> Vec<u8> {
    let mut bytes = bullet_wire::canonical_json(record).unwrap();
    bytes.push(b'\n');
    bytes
}

fn expectation(bytes: &[u8]) -> ContentExpectation {
    ContentExpectation {
        byte_length: bytes.len() as u64,
        sha256: Sha256Digest::for_bytes(bytes),
    }
}

fn source(path: &std::path::Path, bytes: &[u8]) -> SourceExpectation {
    SourceExpectation {
        path: path.to_path_buf(),
        content: expectation(bytes),
    }
}

fn fixture(valid_inventory: bool) -> RecoveryFixture {
    let root = tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o775)).unwrap();
    let claim_id = format!("clm_{}", "a".repeat(64));
    let claim = Record::Claim {
        schema_version: 1,
        at_unix_ms: 5,
        claim_id: claim_id.clone(),
        agent: "fixture-agent".to_owned(),
        lane: "fixture-lane".to_owned(),
        repo: "bullet-farm".to_owned(),
        paths: vec!["src/coord".to_owned()],
        expires_unix_ms: 60_005,
    };
    let prefix = canonical_line(&claim);
    let mut interrupted = prefix.clone();
    interrupted.extend(std::iter::repeat_n(b'x', 69));
    let mut tainted = interrupted.clone();
    tainted.push(b'\n');
    let mut frozen = interrupted.clone();
    frozen[prefix.len() + 11] = b'y';
    frozen.extend_from_slice(b"\n\n");
    let interrupted_path = root.path().join("interrupted.partial");
    let tainted_path = root.path().join("tainted.jsonl");
    let legacy_path = root.path().join("events.jsonl");
    write_private(&interrupted_path, &interrupted, 0o400);
    write_private(&tainted_path, &tainted, 0o400);
    write_private(&legacy_path, &frozen, 0o400);
    let source_meta = fs::metadata(&legacy_path).unwrap();

    let summaries = crate::coord::state::summaries(std::slice::from_ref(&claim), 10).unwrap();
    let trusted_digest =
        bullet_wire::hash_canonical("bullet-family.coord.trusted-state.v2", &summaries).unwrap();
    let active = summaries.get(&claim_id).unwrap();
    assert_eq!(active.state, ClaimState::Active);
    let claim_digest =
        bullet_wire::hash_canonical("bullet-family.coord.frozen-claim.v2", active).unwrap();
    let frozen_claims = vec![FrozenClaimSubject {
        claim_id,
        claim_blake3: format!("blake3:{}", claim_digest.to_hex()),
    }];
    let trusted_state = format!("blake3:{}", trusted_digest.to_hex());
    let placeholder = manifest(
        &prefix,
        &interrupted,
        &tainted,
        &frozen,
        source_meta.dev(),
        source_meta.ino(),
        &trusted_state,
        &frozen_claims,
        format!("blake3:{}", "0".repeat(64)),
    );
    let mut interrupted_file = fs::File::open(&interrupted_path).unwrap();
    let mut tainted_file = fs::File::open(&tainted_path).unwrap();
    let mut frozen_file = fs::File::open(&legacy_path).unwrap();
    let inventory = verifier::compute_post_prefix_inventory(
        &mut interrupted_file,
        &mut tainted_file,
        &mut frozen_file,
        &placeholder,
    )
    .unwrap();
    let manifest = manifest(
        &prefix,
        &interrupted,
        &tainted,
        &frozen,
        source_meta.dev(),
        source_meta.ino(),
        &trusted_state,
        &frozen_claims,
        if valid_inventory {
            inventory
        } else {
            format!("blake3:{}", "f".repeat(64))
        },
    );
    RecoveryFixture {
        input: RecoveryInput {
            coord_dir: root.path().to_path_buf(),
            trusted_prefix: expectation(&prefix),
            interrupted_capture: source(&interrupted_path, &interrupted),
            tainted_generation: source(&tainted_path, &tainted),
            frozen_live_source: source(&legacy_path, &frozen),
        },
        root,
        manifest,
    }
}

#[allow(clippy::too_many_arguments)]
fn manifest(
    prefix: &[u8],
    interrupted: &[u8],
    tainted: &[u8],
    frozen: &[u8],
    device: u64,
    inode: u64,
    trusted_state: &str,
    frozen_claims: &[FrozenClaimSubject],
    inventory: String,
) -> GenerationManifest {
    let artifacts = serde_json::from_value(serde_json::json!({
        "trusted_prefix": binding("archive/trusted-prefix.jsonl", prefix, 1, true),
        "interrupted_capture": binding(
            "archive/interrupted-observation.jsonl.partial", interrupted, 1, false
        ),
        "tainted_generation": binding("archive/tainted-generation.jsonl", tainted, 2, true),
        "frozen_live_source": binding("archive/frozen-live-source.jsonl", frozen, 3, true),
    }))
    .unwrap();
    let body = create_body(CreateBodyInput {
        recovery_operator: "fixture-operator".to_owned(),
        recovery_policy_sha256: Sha256Digest::for_bytes(b"policy"),
        operator_decision_sha256: Sha256Digest::for_bytes(b"decision"),
        replay_contract_version: 1,
        replay_contract_sha256: Sha256Digest::for_bytes(b"replay"),
        bootstrap_commit_oid: "a".repeat(40),
        bootstrap_paths: vec!["src/coord".to_owned()],
        legacy_source_device: device,
        legacy_source_inode: inode,
        parent_generation: "legacy-v1".to_owned(),
        incident_at_unix_ms: 10,
        recovered_at_unix_ms: 20,
        trusted_record_count: 1,
        trusted_projection_inventory: TrustedProjectionInventory {
            record_kinds: TrustedRecordKindCounts {
                claim: 1,
                heartbeat: 0,
                handoff: 0,
                commit_receipt: 0,
                commit_receipt_correction: 0,
                commit_receipt_group: 0,
                commit_receipt_group_correction: 0,
            },
            claim_outcomes: TrustedClaimOutcomeCounts {
                total: 1,
                active: 1,
                expired: 0,
                handed_off_unreceipted: 0,
                receipted: 0,
            },
        },
        discarded_range: serde_json::from_value(serde_json::json!({
            "start_inclusive": prefix.len(), "end_exclusive": frozen.len()
        }))
        .unwrap(),
        ambiguous_tail_range: serde_json::from_value(serde_json::json!({
            "start_inclusive": prefix.len(), "end_exclusive": interrupted.len()
        }))
        .unwrap(),
        ambiguous_tail_sha256: Sha256Digest::for_bytes(&interrupted[prefix.len()..]),
        artifacts,
        trusted_state_blake3: trusted_state.to_owned(),
        frozen_claims: frozen_claims.to_vec(),
        post_prefix_inventory_blake3: inventory,
    })
    .unwrap();
    GenerationManifest::from_body(body).unwrap()
}

#[test]
fn prefix_copy_rewinds_before_full_interrupted_copy() {
    let root = tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let source_path = root.path().join("source");
    let prefix = b"{\"record\":1}\n";
    let full = [prefix.as_slice(), b"{\"kind\":\"part"].concat();
    fs::write(&source_path, &full).unwrap();
    fs::set_permissions(&source_path, fs::Permissions::from_mode(0o400)).unwrap();
    let mut source = OpenOptions::new().read(true).open(&source_path).unwrap();

    io::copy_prefix(
        &mut source,
        root.path(),
        &binding("archive/trusted-prefix.jsonl", prefix, 1, true),
    )
    .unwrap();
    io::copy_artifact(
        &mut source,
        root.path(),
        &binding(
            "archive/interrupted-observation.jsonl.partial",
            &full,
            1,
            false,
        ),
    )
    .unwrap();

    assert_eq!(
        fs::read(root.path().join("archive/trusted-prefix.jsonl")).unwrap(),
        prefix
    );
    assert_eq!(
        fs::read(
            root.path()
                .join("archive/interrupted-observation.jsonl.partial")
        )
        .unwrap(),
        full
    );
}

#[test]
fn create_new_collision_preserves_existing_bytes() {
    let root = tempdir().unwrap();
    let subject = root.path().join("CURRENT");
    fs::write(&subject, b"existing\n").unwrap();
    let error = io::write_new_file(&subject, b"replacement\n", 0o400).unwrap_err();
    assert_eq!(error.code(), "COORD_RECOVERY_COLLISION");
    assert_eq!(fs::read(subject).unwrap(), b"existing\n");
}

#[test]
fn metadata_admission_rejects_symlinks_and_hardlinks() {
    let root = tempdir().unwrap();
    let owner = rustix::process::geteuid().as_raw();
    let target = root.path().join("target");
    fs::write(&target, b"subject\n").unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o400)).unwrap();
    let link = root.path().join("CURRENT");
    symlink(&target, &link).unwrap();
    assert!(io::open_exact_file(&link, owner, 0o400, false).is_err());

    fs::remove_file(&link).unwrap();
    fs::hard_link(&target, &link).unwrap();
    assert!(io::open_exact_file(&target, owner, 0o400, false).is_err());
    assert!(io::open_exact_file(&link, owner, 0o400, false).is_err());
}

#[test]
fn authority_bootstraps_and_retains_the_stable_lock() {
    let root = tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o775)).unwrap();
    let authority = authority::Authority::acquire(root.path()).unwrap();
    assert_eq!(mode(root.path()), 0o700);
    assert_eq!(mode(&root.path().join("LOCK")), 0o600);

    fs::remove_file(root.path().join("LOCK")).unwrap();
    write_private(&root.path().join("LOCK"), b"", 0o600);
    assert!(authority.revalidate_final(root.path()).is_err());
}

#[test]
fn nonempty_lock_refuses_before_tightening_root() {
    let root = tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o775)).unwrap();
    write_private(&root.path().join("LOCK"), b"not-empty", 0o600);
    assert!(authority::Authority::acquire(root.path()).is_err());
    assert_eq!(mode(root.path()), 0o775);
    assert_eq!(fs::read(root.path().join("LOCK")).unwrap(), b"not-empty");
}

#[test]
fn complete_rollover_builds_before_exchange_and_is_idempotent() {
    let fixture = fixture(true);
    let outcome =
        linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false)).unwrap();
    assert_eq!(outcome.state, RecoveryState::Published);
    assert_eq!(mode(fixture.root.path()), 0o700);
    assert_eq!(mode(&fixture.root.path().join("LOCK")), 0o600);
    assert_eq!(mode(&fixture.root.path().join("events.jsonl")), 0o400);
    assert!(fixture.root.path().join("CURRENT").is_file());
    let generation = fixture
        .root
        .path()
        .join("generations")
        .join(fixture.manifest.generation_id().as_str());
    assert!(generation.join("manifest.json").is_file());
    assert!(generation.join("events.jsonl").is_file());

    let repeated =
        linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false)).unwrap();
    assert_eq!(repeated.state, RecoveryState::AlreadyCurrent);
}

#[test]
fn inventory_mismatch_refuses_before_generation_or_exchange() {
    let fixture = fixture(false);
    let error = linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false))
        .unwrap_err();
    assert_eq!(error.code(), "INVALID_COORD_RECOVERY");
    assert!(fixture.root.path().join("events.jsonl").is_file());
    assert_eq!(mode(fixture.root.path()), 0o775);
    assert!(!fixture.root.path().join("LOCK").exists());
    assert!(!fixture.root.path().join("generations").exists());
    assert!(!fixture.root.path().join("CURRENT").exists());
}

#[test]
fn writable_old_descriptor_refuses_before_mutation_then_resumes() {
    let fixture = fixture(true);
    let legacy = fixture.root.path().join("events.jsonl");
    fs::set_permissions(&legacy, fs::Permissions::from_mode(0o600)).unwrap();
    let writer = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&legacy)
        .unwrap();
    fs::set_permissions(&legacy, fs::Permissions::from_mode(0o400)).unwrap();

    let error = linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false))
        .unwrap_err();
    assert_eq!(error.code(), "LEGACY_WRITE_AUTHORITY_UNKNOWN");
    assert_eq!(mode(fixture.root.path()), 0o775);
    assert!(!fixture.root.path().join("LOCK").exists());
    assert!(!fixture.root.path().join("recovery").exists());
    assert!(!fixture.root.path().join("generations").exists());
    assert!(!fixture.root.path().join("CURRENT").exists());
    assert!(legacy.is_file());

    drop(writer);
    let resumed =
        linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false)).unwrap();
    assert_eq!(resumed.state, RecoveryState::Published);
    assert!(fixture.root.path().join("CURRENT").is_file());
}

#[test]
fn exact_transition_interruptions_resume_once() {
    for point in [
        linux::TransitionCrash::Seal,
        linux::TransitionCrash::Exchange,
        linux::TransitionCrash::Retire,
    ] {
        let fixture = fixture(true);
        linux::test_crash_at(point);
        let error =
            linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false))
                .unwrap_err();
        assert_eq!(error.code(), "COORD_RECOVERY_TEST_INTERRUPTION");
        assert!(!fixture.root.path().join("CURRENT").exists());

        let outcome =
            linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false))
                .unwrap();
        assert_eq!(outcome.state, RecoveryState::ResumedAndPublished);
        assert_eq!(mode(&fixture.root.path().join("events.jsonl")), 0o400);
        assert!(fixture.root.path().join("CURRENT").is_file());
        for evidence in [
            "intent.json",
            "prepared-tombstone-seal-observation.json",
            "tombstone-seal-observation.json",
            "retirement-completion-observation.json",
        ] {
            let path = fixture
                .root
                .path()
                .join("recovery")
                .join(fixture.manifest.generation_id().as_str())
                .join(evidence);
            assert_eq!(mode(&path), 0o400);
        }
    }
}

#[test]
fn divergent_tombstone_observation_remains_frozen() {
    let fixture = fixture(true);
    let paused =
        linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(true)).unwrap();
    assert_eq!(paused.state, RecoveryState::FrozenWaitingForLegacyWriters);
    let observation = fixture
        .root
        .path()
        .join("recovery")
        .join(fixture.manifest.generation_id().as_str())
        .join("tombstone-seal-observation.json");
    fs::set_permissions(&observation, fs::Permissions::from_mode(0o600)).unwrap();
    write_private(&observation, b"corrupt\n", 0o400);
    let error = linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false))
        .unwrap_err();
    assert_eq!(error.code(), "TOMBSTONE_SEAL_OUTCOME_UNKNOWN");
    assert!(!fixture.root.path().join("CURRENT").exists());
    assert_eq!(mode(&fixture.root.path().join("events.jsonl")), 0o400);
}

#[test]
fn final_tombstone_rebind_before_current_refuses_publication() {
    let fixture = fixture(true);
    let mut probes = 0_u8;
    let error = linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| {
        probes += 1;
        if probes == 2 {
            let tombstone = fixture.root.path().join("events.jsonl");
            fs::rename(&tombstone, fixture.root.path().join("displaced-tombstone")).unwrap();
            fs::create_dir(&tombstone).unwrap();
            fs::set_permissions(&tombstone, fs::Permissions::from_mode(0o400)).unwrap();
        }
        Ok(false)
    })
    .unwrap_err();
    assert_eq!(error.code(), "COORD_RECOVERY_FINAL_REBIND_UNKNOWN");
    assert!(!fixture.root.path().join("CURRENT").exists());
}

#[test]
fn retired_source_rebind_after_current_refuses_green_outcome() {
    let fixture = fixture(true);
    let retired = fixture
        .root
        .path()
        .join("recovery")
        .join(fixture.manifest.generation_id().as_str())
        .join("retired-v1.non-authoritative");
    let displaced = retired.with_file_name("displaced-retired-source");
    let error = linux::recover_with_post_publish_probe(
        &fixture.input,
        &fixture.manifest,
        |_| Ok(false),
        || {
            fs::rename(&retired, &displaced).map_err(CoordError::io)?;
            fs::copy(&displaced, &retired).map_err(CoordError::io)?;
            fs::set_permissions(&retired, fs::Permissions::from_mode(0o400)).map_err(CoordError::io)
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), "COORD_RECOVERY_FINAL_REBIND_UNKNOWN");
    assert!(fixture.root.path().join("CURRENT").is_file());
}

#[path = "tests/exchange.rs"]
mod exchange_tests;

#[path = "tests/adoption_fixture.rs"]
pub(in crate::coord) mod adoption_fixture;
