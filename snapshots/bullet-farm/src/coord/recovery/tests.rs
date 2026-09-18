use std::{
    cell::Cell,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
};

use super::*;

mod audit;

#[derive(Clone, Copy)]
struct BoundaryClock {
    unix_ms: u64,
    boottime_ms: u64,
    namespace: (u64, u64),
}

thread_local! {
    static FAIL_FRESH_INSPECTION: Cell<bool> = const { Cell::new(false) };
    static AUTHORITY_CLOCK: Cell<Option<BoundaryClock>> = const { Cell::new(None) };
}

pub(super) fn take_fresh_inspection_failure() -> bool {
    FAIL_FRESH_INSPECTION.with(|value| value.replace(false))
}

fn fail_next_fresh_inspection() {
    FAIL_FRESH_INSPECTION.with(|value| value.set(true));
}

pub(super) fn rebind_clock_at_authority() {
    if let Some(clock) = AUTHORITY_CLOCK.with(|value| value.take()) {
        recovery_manifest::set_test_clock(clock.unix_ms, clock.boottime_ms, clock.namespace);
    }
}

fn schedule_authority_clock(unix_ms: u64, boottime_ms: u64, namespace: (u64, u64)) {
    AUTHORITY_CLOCK.with(|value| {
        value.set(Some(BoundaryClock {
            unix_ms,
            boottime_ms,
            namespace,
        }));
    });
}

struct SourceFixture {
    family: tempfile::TempDir,
    interrupted: PathBuf,
    tainted: PathBuf,
    frozen: PathBuf,
}

struct FacadeFixture {
    source: SourceFixture,
    manifest: PathBuf,
    command: RecoveryCommand,
    expected: GenerationManifest,
    inspection: RecoveryInspectionV1,
    _authority: recovery_manifest::TestAuthority,
    _clock: recovery_manifest::TestClockGuard,
}

fn fixture_command() -> FacadeFixture {
    let source = source_fixture();
    let inspection_command = RecoveryInspectionCommand {
        interrupted_capture: source.interrupted.clone(),
        tainted_generation: source.tainted.clone(),
        frozen_live_source: source.frozen.clone(),
    };
    let inspection = recovery_manifest::inspect(source.family.path(), &inspection_command).unwrap();
    let clock = recovery_manifest::install_test_clock(20);
    let authority = recovery_manifest::test_authority(&inspection, 10, 100).unwrap();
    let inspection_path = source.family.path().join("recovery-inspection.json");
    let authorization_path = source.family.path().join("recovery-authorization.json");
    let signature_path = source
        .family
        .path()
        .join("recovery-authorization-signature.json");
    let provenance_path = source
        .family
        .path()
        .join("recovery-bootstrap-provenance.json");
    let manifest = source.family.path().join("recovery-manifest.json");
    super::super::sealed::write(&inspection_path, &inspection).unwrap();
    super::super::sealed::write(&authorization_path, &authority.authorization).unwrap();
    super::super::sealed::write(&signature_path, &authority.signature).unwrap();
    super::super::sealed::write(&provenance_path, &authority.provenance).unwrap();
    let expected = recovery_manifest::authorize(
        &inspection,
        &authority.authorization,
        &authority.signature,
        &authority.provenance,
    )
    .unwrap()
    .manifest;
    super::super::sealed::write(&manifest, &expected).unwrap();
    let command = RecoveryCommand::new(
        manifest.clone(),
        inspection_path,
        authorization_path,
        signature_path,
        provenance_path,
        source.interrupted.clone(),
        source.tainted.clone(),
        source.frozen.clone(),
    );
    FacadeFixture {
        source,
        manifest,
        command,
        expected,
        inspection,
        _authority: authority,
        _clock: clock,
    }
}

fn source_fixture() -> SourceFixture {
    let family = tempfile::tempdir().unwrap();
    fs::set_permissions(family.path(), fs::Permissions::from_mode(0o700)).unwrap();
    let coord = family.path().join(".bullet-family/coord");
    let inputs = family.path().join("recovery-input");
    fs::create_dir_all(&coord).unwrap();
    fs::create_dir(&inputs).unwrap();
    fs::set_permissions(&coord, fs::Permissions::from_mode(0o775)).unwrap();
    fs::set_permissions(&inputs, fs::Permissions::from_mode(0o700)).unwrap();
    let claim = serde_json::json!({
        "kind": "claim",
        "schema_version": 1,
        "at_unix_ms": 5,
        "claim_id": format!("clm_{}", "a".repeat(64)),
        "agent": "fixture-agent",
        "lane": "fixture-lane",
        "repo": "bullet-farm",
        "paths": ["src/coord"],
        "expires_unix_ms": 60_005,
    });
    let mut trusted = bullet_wire::canonical_json(&claim).unwrap();
    trusted.push(b'\n');
    let mut interrupted_bytes = trusted.clone();
    interrupted_bytes.extend_from_slice(b"partial-record");
    let mut tainted_bytes = interrupted_bytes.clone();
    tainted_bytes.extend_from_slice(b"-tainted-and-committed\n");
    let mut frozen_bytes = trusted;
    frozen_bytes.extend_from_slice(b"different-frozen-record-one\n");
    frozen_bytes.extend_from_slice(b"different-frozen-record-two-with-padding\n");
    let interrupted = inputs.join("interrupted.partial");
    let tainted = inputs.join("tainted.jsonl");
    let frozen = coord.join("events.jsonl");
    write_private(&interrupted, &interrupted_bytes);
    write_private(&tainted, &tainted_bytes);
    write_private(&frozen, &frozen_bytes);
    SourceFixture {
        family,
        interrupted,
        tainted,
        frozen,
    }
}

fn write_private(path: &Path, bytes: &[u8]) {
    fs::write(path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o400)).unwrap();
}

fn error_code<T>(result: Result<T, CoordError>) -> &'static str {
    match result {
        Ok(_) => panic!("recovery preparation unexpectedly succeeded"),
        Err(error) => error.code(),
    }
}

fn execute_with_writer_probe(
    family_root: &Path,
    command: &RecoveryCommand,
    writer_probe: impl FnMut((u64, u64)) -> Result<bool, CoordError>,
) -> Result<RecoveryExecution, CoordError> {
    let authority = prepare_authority(family_root, command)?;
    authority.authorized.require_read_only_replay()?;
    if let Some(execution) = verify_current(&authority.coord_dir, &authority.authorized.manifest)? {
        return Ok(execution);
    }
    authority.authorized.require_active()?;
    let prepared = prepare_mutation(family_root, command, authority)?;
    prepared.authorized.require_active()?;
    let outcome = crate::coord::generation::recovery::recover_with_writer_probe(
        &prepared.input,
        &prepared.manifest,
        writer_probe,
    )?;
    execution(outcome)
}

#[test]
fn inspection_is_creation_free_and_execution_restarts_exactly() {
    let fixture = fixture_command();
    let current = fixture
        .source
        .family
        .path()
        .join(".bullet-family/coord/CURRENT");
    assert!(!current.exists());
    let authority = prepare_authority(fixture.source.family.path(), &fixture.command).unwrap();
    let inspected =
        prepare_mutation(fixture.source.family.path(), &fixture.command, authority).unwrap();
    assert_eq!(
        inspected.manifest.generation_id().as_str(),
        fixture.expected.generation_id().as_str()
    );
    assert!(!current.exists());
    assert_eq!(
        fs::metadata(fixture.source.family.path().join(".bullet-family/coord"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o775
    );

    let first = execute_with_writer_probe(fixture.source.family.path(), &fixture.command, |_| {
        Ok(false)
    })
    .unwrap();
    assert_eq!(first.state, RecoveryExecutionState::Published);
    let replay = execute_with_writer_probe(fixture.source.family.path(), &fixture.command, |_| {
        panic!("already-current replay must not invoke the writer probe")
    })
    .unwrap();
    assert_eq!(replay.state, RecoveryExecutionState::AlreadyCurrent);
    assert_eq!(replay.generation_id, first.generation_id);
}

#[test]
fn writer_wait_is_typed_and_release_resumes_the_same_generation() {
    let fixture = fixture_command();
    let current = fixture
        .source
        .family
        .path()
        .join(".bullet-family/coord/CURRENT");
    let error =
        execute_with_writer_probe(fixture.source.family.path(), &fixture.command, |_| Ok(true))
            .unwrap_err();
    assert_eq!(error.code(), "COORD_RECOVERY_WRITER_WAIT");
    assert!(!current.exists());

    let resumed = execute_with_writer_probe(fixture.source.family.path(), &fixture.command, |_| {
        Ok(false)
    })
    .unwrap();
    assert_eq!(resumed.state, RecoveryExecutionState::ResumedAndPublished);
    assert_eq!(
        resumed.generation_id,
        fixture.expected.generation_id().as_str()
    );
    assert!(current.is_file());
}

#[test]
fn exchange_interruption_restarts_the_same_generation_through_the_facade() {
    let fixture = fixture_command();
    let current = fixture
        .source
        .family
        .path()
        .join(".bullet-family/coord/CURRENT");
    crate::coord::generation::recovery::test_crash_at_exchange();

    let error = execute_with_writer_probe(fixture.source.family.path(), &fixture.command, |_| {
        Ok(false)
    })
    .unwrap_err();
    assert_eq!(error.code(), "COORD_RECOVERY_TEST_INTERRUPTION");
    assert!(!current.exists());

    let resumed = execute_with_writer_probe(fixture.source.family.path(), &fixture.command, |_| {
        Ok(false)
    })
    .unwrap();
    assert_eq!(resumed.state, RecoveryExecutionState::ResumedAndPublished);
    assert_eq!(
        resumed.generation_id,
        fixture.expected.generation_id().as_str()
    );
    assert!(current.is_file());
}

#[test]
fn expired_authority_refuses_mutation_but_replays_exact_current_read_only() {
    let unpublished = fixture_command();
    let current = unpublished
        .source
        .family
        .path()
        .join(".bullet-family/coord/CURRENT");
    let _expired = recovery_manifest::install_test_clock(100);
    let error = execute_with_writer_probe(
        unpublished.source.family.path(),
        &unpublished.command,
        |_| panic!("expired authorization must not reach the writer probe"),
    )
    .unwrap_err();
    assert_eq!(error.code(), "RECOVERY_AUTHORIZATION_EXPIRED");
    assert!(!current.exists());
    drop(_expired);

    let published = fixture_command();
    let first =
        execute_with_writer_probe(published.source.family.path(), &published.command, |_| {
            Ok(false)
        })
        .unwrap();
    assert_eq!(first.state, RecoveryExecutionState::Published);
    let _expired = recovery_manifest::install_test_clock(100);
    let replay =
        execute_with_writer_probe(published.source.family.path(), &published.command, |_| {
            panic!("expired exact replay must not reach the writer probe")
        })
        .unwrap();
    assert_eq!(replay.state, RecoveryExecutionState::AlreadyCurrent);
    assert_eq!(replay.generation_id, first.generation_id);
}

#[test]
fn paths_manifest_mode_and_live_source_are_exact() {
    let fixture = fixture_command();
    let relative = RecoveryCommand {
        manifest: PathBuf::from("recovery-manifest.json"),
        ..fixture.command.clone()
    };
    assert_eq!(
        error_code(prepare_authority(fixture.source.family.path(), &relative)),
        "INVALID_COORD_RECOVERY"
    );

    fs::set_permissions(&fixture.manifest, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(
        error_code(prepare_authority(
            fixture.source.family.path(),
            &fixture.command
        )),
        "INVALID_RECOVERY_PRODUCTION"
    );
    fs::set_permissions(&fixture.manifest, fs::Permissions::from_mode(0o400)).unwrap();

    let substituted = RecoveryCommand {
        frozen_live_source: fixture.source.interrupted.clone(),
        ..fixture.command.clone()
    };
    assert_eq!(
        error_code(prepare_authority(
            fixture.source.family.path(),
            &substituted
        )),
        "INVALID_COORD_RECOVERY"
    );
}

#[test]
fn caller_fabricated_manifest_cannot_bypass_signed_authority() {
    let fixture = fixture_command();
    let fabricated_path = fixture
        .source
        .family
        .path()
        .join("fabricated-manifest.json");
    let fabricated = adoption_fixture_manifest();
    super::super::sealed::write(&fabricated_path, &fabricated).unwrap();
    let command = RecoveryCommand {
        manifest: fabricated_path,
        ..fixture.command.clone()
    };
    assert_eq!(
        error_code(prepare_authority(fixture.source.family.path(), &command)),
        "INVALID_COORD_RECOVERY"
    );
    assert!(
        !fixture
            .source
            .family
            .path()
            .join(".bullet-family/coord/CURRENT")
            .exists()
    );
}

fn adoption_fixture_manifest() -> GenerationManifest {
    crate::coord::generation::recovery::adoption_fixture::fixture(&"1".repeat(40), &"2".repeat(40))
        .manifest
}

#[test]
fn non_normal_and_symlinked_ancestor_paths_refuse_without_publication() {
    let fixture = fixture_command();
    let current = fixture
        .source
        .family
        .path()
        .join(".bullet-family/coord/CURRENT");
    let non_normal = RecoveryCommand {
        manifest: fixture
            .manifest
            .parent()
            .unwrap()
            .join(".")
            .join("recovery-manifest.json"),
        ..fixture.command.clone()
    };
    assert_eq!(
        error_code(prepare_authority(fixture.source.family.path(), &non_normal)),
        "INVALID_COORD_RECOVERY"
    );
    assert!(!current.exists());

    let alias = fixture.source.family.path().join("manifest-parent-link");
    symlink(fixture.source.family.path(), &alias).unwrap();
    let symlinked = RecoveryCommand {
        manifest: alias.join("recovery-manifest.json"),
        ..fixture.command.clone()
    };
    assert_eq!(
        error_code(prepare_authority(fixture.source.family.path(), &symlinked)),
        "INVALID_RECOVERY_PRODUCTION"
    );
    assert!(!current.exists());
}
