use std::{fs, io::Write, os::unix::fs::PermissionsExt};

use super::*;
use crate::coord::generation::manifest::CurrentPointer;

fn owner() -> u32 {
    rustix::process::geteuid().as_raw()
}

fn sealed_predecessors(fixture: &RecoveryFixture, publish_prepared: bool) {
    let baseline_record = authority::baseline_record(&fixture.manifest).unwrap();
    let baseline = authority::baseline_subject(&fixture.manifest, &baseline_record).unwrap();
    let mut preflight =
        verifier::creation_free_preflight(&fixture.input, &fixture.manifest).unwrap();
    let authority = authority::Authority::acquire(fixture.root.path()).unwrap();
    verifier::revalidate_preflight(&mut preflight, &fixture.input, &fixture.manifest).unwrap();
    let layout = super::super::tree::Layout::ensure(
        authority.root(),
        fixture.manifest.generation_id().as_str(),
        owner(),
    )
    .unwrap();
    layout
        .build_generation(
            &mut preflight.interrupted,
            &mut preflight.tainted,
            &mut preflight.legacy,
            &fixture.manifest,
            &baseline_record,
            &baseline,
        )
        .unwrap();
    let sibling = exchange::sibling_name(fixture.manifest.generation_id().as_str()).unwrap();
    let prepared = exchange::prepare(
        authority.root(),
        &sibling,
        owner(),
        exchange::SiblingState::Absent,
    )
    .unwrap();
    let intent = authority::write_or_verify_intent(
        layout.recovery(),
        &fixture.manifest,
        &fixture.input.frozen_live_source.content,
        preflight.source_identity,
        verifier::identity(prepared.retained()).unwrap(),
        &baseline,
        true,
    )
    .unwrap();
    exchange::seal(authority.root(), &sibling, &prepared, owner()).unwrap();
    if publish_prepared {
        exchange::write_or_verify_prepared_observation(
            layout.recovery(),
            &fixture.manifest,
            &baseline,
            &intent,
            &sibling,
            &prepared,
            owner(),
            true,
        )
        .unwrap();
    }
}

#[test]
fn retained_exchange_inventories_before_and_after_read_only_seal() {
    let root = tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let recovery = root.path().join("recovery");
    fs::create_dir(&recovery).unwrap();
    fs::set_permissions(&recovery, fs::Permissions::from_mode(0o700)).unwrap();
    let legacy = root.path().join("events.jsonl");
    let mut stale = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&legacy)
        .unwrap();
    stale.write_all(b"trusted\npartial").unwrap();
    stale.sync_all().unwrap();
    fs::set_permissions(&legacy, fs::Permissions::from_mode(0o400)).unwrap();
    let source_identity = verifier::identity(&stale).unwrap();
    let root_fd = io::open_directory(root.path(), owner(), 0o700).unwrap();
    let recovery_fd = io::open_directory(&recovery, owner(), 0o700).unwrap();
    let sibling = exchange::sibling_name(&format!("gen_{}", "1".repeat(64))).unwrap();
    let prepared =
        exchange::prepare(&root_fd, &sibling, owner(), exchange::SiblingState::Absent).unwrap();
    let tombstone_identity = verifier::identity(prepared.retained()).unwrap();
    exchange::seal(&root_fd, &sibling, &prepared, owner()).unwrap();
    exchange::exchange(&root_fd, &sibling, &prepared, &stale, owner()).unwrap();
    assert_eq!(mode(&legacy), 0o400);
    assert_eq!(
        verifier::identity(prepared.retained()).unwrap(),
        tombstone_identity
    );
    exchange::retire(&root_fd, &recovery_fd, &sibling, &stale, owner()).unwrap();
    assert_eq!(
        fs::read(recovery.join("retired-v1.non-authoritative")).unwrap(),
        b"trusted\npartial"
    );
    assert!(verifier::has_other_writable_fd(source_identity).unwrap());
}

#[test]
fn inserted_child_after_seal_refuses_before_exchange() {
    let root = tempdir().unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
    write_private(&root.path().join("events.jsonl"), b"legacy\n", 0o400);
    let root_fd = io::open_directory(root.path(), owner(), 0o700).unwrap();
    let legacy =
        io::open_exact_file(&root.path().join("events.jsonl"), owner(), 0o400, false).unwrap();
    let sibling = exchange::sibling_name(&format!("gen_{}", "2".repeat(64))).unwrap();
    let prepared =
        exchange::prepare(&root_fd, &sibling, owner(), exchange::SiblingState::Absent).unwrap();
    exchange::seal(&root_fd, &sibling, &prepared, owner()).unwrap();
    let path = root.path().join(&sibling);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    write_private(&path.join("foreign"), b"x", 0o400);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(exchange::exchange(&root_fd, &sibling, &prepared, &legacy, owner()).is_err());
    assert!(root.path().join("events.jsonl").is_file());
}

#[test]
fn presealed_restart_requires_exact_intent() {
    let fixture = fixture(true);
    sealed_predecessors(&fixture, true);
    let intent = fixture
        .root
        .path()
        .join("recovery")
        .join(fixture.manifest.generation_id().as_str())
        .join("intent.json");
    fs::remove_file(intent).unwrap();
    let error = linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false))
        .unwrap_err();
    assert_eq!(error.code(), "COORD_RECOVERY_INTENT_OUTCOME_UNKNOWN");
    assert!(fixture.root.path().join("events.jsonl").is_file());
    assert!(!fixture.root.path().join("CURRENT").exists());
}

#[test]
fn presealed_restart_reconstructs_missing_prepared_observation() {
    let fixture = fixture(true);
    sealed_predecessors(&fixture, true);
    let observation = fixture
        .root
        .path()
        .join("recovery")
        .join(fixture.manifest.generation_id().as_str())
        .join("prepared-tombstone-seal-observation.json");
    fs::remove_file(&observation).unwrap();
    let outcome =
        linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false)).unwrap();
    assert_eq!(outcome.state, RecoveryState::ResumedAndPublished);
    assert_eq!(mode(&observation), 0o400);
    assert!(fixture.root.path().join("CURRENT").is_file());
}

#[test]
fn presealed_restart_with_exact_evidence_continues_once() {
    let fixture = fixture(true);
    sealed_predecessors(&fixture, true);
    let outcome =
        linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false)).unwrap();
    assert_eq!(outcome.state, RecoveryState::ResumedAndPublished);
    assert_eq!(mode(&fixture.root.path().join("events.jsonl")), 0o400);
    assert!(fixture.root.path().join("CURRENT").is_file());
}

#[test]
fn retired_restart_reconstructs_missing_retirement_observation() {
    let fixture = fixture(true);
    let paused =
        linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(true)).unwrap();
    assert_eq!(paused.state, RecoveryState::FrozenWaitingForLegacyWriters);
    let evidence = fixture
        .root
        .path()
        .join("recovery")
        .join(fixture.manifest.generation_id().as_str())
        .join("retirement-completion-observation.json");
    fs::remove_file(&evidence).unwrap();
    let outcome =
        linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false)).unwrap();
    assert_eq!(outcome.state, RecoveryState::ResumedAndPublished);
    assert_eq!(mode(&evidence), 0o400);
    assert!(fixture.root.path().join("CURRENT").is_file());
}

#[test]
fn legacy_mode_zero_tombstone_is_frozen_not_adopted() {
    let fixture = fixture(true);
    sealed_predecessors(&fixture, true);
    let sibling = exchange::sibling_path(
        fixture.root.path(),
        fixture.manifest.generation_id().as_str(),
    )
    .unwrap();
    fs::set_permissions(&sibling, fs::Permissions::from_mode(0o000)).unwrap();
    let error = linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false))
        .unwrap_err();
    assert_eq!(error.code(), "TOMBSTONE_LEGACY_RECONCILIATION_UNKNOWN");
    assert_eq!(mode(&sibling), 0);
    assert!(fixture.root.path().join("events.jsonl").is_file());
    assert!(!fixture.root.path().join("CURRENT").exists());
}

#[test]
fn top_level_mode_zero_tombstone_is_frozen_not_adopted() {
    let fixture = fixture(true);
    let legacy = fixture.root.path().join("events.jsonl");
    fs::remove_file(&legacy).unwrap();
    fs::create_dir(&legacy).unwrap();
    fs::set_permissions(&legacy, fs::Permissions::from_mode(0o0)).unwrap();
    let error = linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false))
        .unwrap_err();
    assert_eq!(error.code(), "TOMBSTONE_LEGACY_RECONCILIATION_UNKNOWN");
    assert_eq!(mode(&legacy), 0);
    assert_eq!(mode(fixture.root.path()), 0o775);
    assert!(!fixture.root.path().join("LOCK").exists());
    assert!(!fixture.root.path().join("CURRENT").exists());
}

#[test]
fn post_current_path_substitution_is_unknown_after_exact_rebind() {
    for substitution in 0..3 {
        let fixture = fixture(true);
        let root = fixture.root.path().to_owned();
        let generation_id = fixture.manifest.generation_id().as_str().to_owned();
        let error = linux::recover_with_post_publish_probe(
            &fixture.input,
            &fixture.manifest,
            |_| Ok(false),
            || {
                match substitution {
                    0 => {
                        let current = root.join("events.jsonl");
                        fs::rename(&current, root.join("events.retained")).unwrap();
                        fs::create_dir(&current).unwrap();
                        fs::set_permissions(&current, fs::Permissions::from_mode(0o400)).unwrap();
                    }
                    1 => {
                        let recovery = root.join("recovery").join(&generation_id);
                        let retired = recovery.join("retired-v1.non-authoritative");
                        fs::rename(&retired, recovery.join("retired.retained")).unwrap();
                        write_private(&retired, b"substitute\n", 0o400);
                    }
                    _ => {
                        let sibling = exchange::sibling_path(&root, &generation_id).unwrap();
                        fs::create_dir(&sibling).unwrap();
                        fs::set_permissions(&sibling, fs::Permissions::from_mode(0o400)).unwrap();
                    }
                }
                Ok(())
            },
        )
        .unwrap_err();
        assert_eq!(error.code(), "COORD_RECOVERY_FINAL_REBIND_UNKNOWN");
        assert!(fixture.root.path().join("CURRENT").is_file());
    }
}

fn seed_partial_artifact(fixture: &RecoveryFixture, bytes: &[u8]) {
    let generations = fixture.root.path().join("generations");
    fs::create_dir(&generations).unwrap();
    fs::set_permissions(&generations, fs::Permissions::from_mode(0o700)).unwrap();
    let stage = generations.join(format!(
        ".next-{}",
        fixture.manifest.generation_id().as_str()
    ));
    fs::create_dir(&stage).unwrap();
    fs::set_permissions(&stage, fs::Permissions::from_mode(0o700)).unwrap();
    let archive = stage.join("archive");
    fs::create_dir(&archive).unwrap();
    fs::set_permissions(&archive, fs::Permissions::from_mode(0o700)).unwrap();
    write_private(&archive.join("trusted-prefix.jsonl"), bytes, 0o600);
}

#[test]
fn deterministic_generation_stage_resumes_exact_partial_offsets() {
    for fraction in [0, 1, 2] {
        let fixture = fixture(true);
        let source = fs::read(&fixture.input.interrupted_capture.path).unwrap();
        let expected = &source[..fixture.input.trusted_prefix.byte_length as usize];
        let offset = expected.len() * fraction / 2;
        seed_partial_artifact(&fixture, &expected[..offset]);
        let outcome =
            linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false))
                .unwrap();
        assert_eq!(outcome.state, RecoveryState::Published);
    }
}

#[test]
fn deterministic_generation_stage_rejects_divergent_or_oversized_partial() {
    for bytes in [
        b"wrong".as_slice(),
        b"oversized-deterministic-partial".as_slice(),
    ] {
        let fixture = fixture(true);
        seed_partial_artifact(&fixture, bytes);
        let error =
            linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false))
                .unwrap_err();
        assert_eq!(error.code(), "COORD_GENERATION_BUILD_OUTCOME_UNKNOWN");
        assert!(fixture.root.path().join("events.jsonl").is_file());
        assert!(!fixture.root.path().join("CURRENT").exists());
    }
}

#[test]
fn final_and_stage_generation_topology_is_unknown() {
    let fixture = fixture(true);
    linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(true)).unwrap();
    let stage = fixture.root.path().join("generations").join(format!(
        ".next-{}",
        fixture.manifest.generation_id().as_str()
    ));
    fs::create_dir(&stage).unwrap();
    fs::set_permissions(&stage, fs::Permissions::from_mode(0o700)).unwrap();
    let error = linux::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false))
        .unwrap_err();
    assert_eq!(error.code(), "COORD_GENERATION_BUILD_OUTCOME_UNKNOWN");
}

#[test]
fn current_sealed_stage_is_adopted_and_foreign_stage_refuses() {
    let first = fixture(true);
    let authority = authority::Authority::acquire(first.root.path()).unwrap();
    let pointer = CurrentPointer::for_manifest(&first.manifest)
        .unwrap()
        .canonical_bytes()
        .unwrap();
    let stage = first.root.path().join(format!(
        ".CURRENT.next-{}",
        first.manifest.generation_id().as_str()
    ));
    write_private(&stage, &pointer, 0o400);
    authority
        .publish_current(
            first.root.path(),
            first.manifest.generation_id().as_str(),
            &pointer,
        )
        .unwrap();
    assert_eq!(
        fs::read(first.root.path().join("CURRENT")).unwrap(),
        pointer
    );

    let second = fixture(true);
    let authority = authority::Authority::acquire(second.root.path()).unwrap();
    write_private(
        &second.root.path().join("CURRENT.next.foreign"),
        b"x",
        0o400,
    );
    let error = authority
        .publish_current(
            second.root.path(),
            second.manifest.generation_id().as_str(),
            b"exact\n",
        )
        .unwrap_err();
    assert_eq!(error.code(), "COORD_CURRENT_OUTCOME_UNKNOWN");
}

#[test]
fn retained_authority_refuses_outer_root_replacement() {
    let parent = tempdir().unwrap();
    let root = parent.path().join("coord");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o775)).unwrap();
    let authority = authority::Authority::acquire(&root).unwrap();
    let moved = parent.path().join("moved");
    fs::rename(&root, &moved).unwrap();
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    write_private(&root.join("LOCK"), b"", 0o600);
    assert!(authority.revalidate_final(&root).is_err());
    assert!(!root.join("CURRENT").exists());
}
