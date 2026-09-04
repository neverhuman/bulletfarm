use std::{
    cell::Cell,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Barrier},
    thread,
};

#[cfg(target_os = "linux")]
#[path = "tests/fence.rs"]
mod fence;

#[cfg(target_os = "linux")]
#[path = "tests/transaction_authority.rs"]
mod transaction_authority;

use super::*;
use crate::coord::generation::manifest::{GenerationManifest, Sha256Digest};

fn provenance() -> GenesisProvenance {
    GenesisProvenance {
        operator: "test-operator".to_owned(),
        policy_sha256: Sha256Digest::for_bytes(b"policy"),
        replay_contract_version: 1,
        replay_contract_sha256: Sha256Digest::for_bytes(b"replay"),
        bootstrap_commit_oid: "b".repeat(40),
        bootstrap_paths: vec!["src/coord".to_owned()],
    }
}

fn manifest() -> GenerationManifest {
    super::genesis::prepare(&provenance(), 30).unwrap().manifest
}

fn initialize(ledger: &Ledger) -> LedgerView {
    ledger.initialize_genesis(&provenance(), || Ok(30)).unwrap()
}

fn heartbeat(note: &str) -> Record {
    Record::Heartbeat {
        schema_version: GENERATION_SCHEMA_VERSION,
        at_unix_ms: 31,
        claim_id: "clm_test".to_owned(),
        agent: "test-agent".to_owned(),
        expires_unix_ms: 32,
        note: Some(note.to_owned()),
    }
}

fn ledger(root: &Path) -> Ledger {
    Ledger::new(root)
}
fn request(fill: char) -> String {
    format!("req_{}", fill.to_string().repeat(64))
}

#[cfg(unix)]
fn mode(path: &Path) -> u32 {
    use std::os::unix::fs::MetadataExt;
    fs::symlink_metadata(path).unwrap().mode() & 0o7777
}

#[cfg(unix)]
fn set_mode(path: &Path, value: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(value)).unwrap();
}

fn staged_name(name: &str, bytes: &[u8]) -> String {
    let digest = bullet_wire::hash_framed_bytes("bullet.coord.staged-file.v2", bytes).unwrap();
    format!(".{name}.next-{}", digest.to_hex())
}

fn coord_path(root: &Path) -> PathBuf {
    root.join(COORD_CHILD)
}

fn remove_current(root: &Path) {
    fs::remove_file(coord_path(root).join("CURRENT")).unwrap();
}

fn tamper_canonical_file(path: &Path) {
    let mut bytes = fs::read(path).unwrap();
    let index = bytes
        .iter()
        .rposition(u8::is_ascii_hexdigit)
        .expect("fixture contains a digest");
    bytes[index] = if bytes[index] == b'0' { b'1' } else { b'0' };
    set_mode(path, 0o600);
    fs::write(path, bytes).unwrap();
    set_mode(path, 0o400);
}

fn canonical_record(record: &Record) -> Vec<u8> {
    bullet_wire::canonical_json(record).unwrap()
}

fn canonical_records(records: &[Record]) -> Vec<Vec<u8>> {
    records.iter().map(canonical_record).collect()
}

#[cfg(target_os = "linux")]
#[derive(Debug, Eq, PartialEq)]
struct SnapshotNode {
    path: PathBuf,
    kind: u8,
    mode: u32,
    device: u64,
    inode: u64,
    links: u64,
    length: u64,
    bytes: Vec<u8>,
}

#[cfg(target_os = "linux")]
fn snapshot(root: &Path) -> Vec<SnapshotNode> {
    fn visit(root: &Path, path: &Path, nodes: &mut Vec<SnapshotNode>) {
        use std::os::unix::fs::MetadataExt;

        let metadata = fs::symlink_metadata(path).unwrap();
        let kind = if metadata.is_dir() {
            1
        } else if metadata.is_file() {
            2
        } else {
            3
        };
        nodes.push(SnapshotNode {
            path: path.strip_prefix(root).unwrap().to_owned(),
            kind,
            mode: metadata.mode() & 0o7777,
            device: metadata.dev(),
            inode: metadata.ino(),
            links: metadata.nlink(),
            length: metadata.len(),
            bytes: if metadata.is_file() {
                fs::read(path).unwrap()
            } else {
                Vec::new()
            },
        });
        if metadata.is_dir() {
            let mut children = fs::read_dir(path)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>();
            children.sort();
            for child in children {
                visit(root, &child, nodes);
            }
        }
    }

    let mut nodes = Vec::new();
    visit(root, root, &mut nodes);
    nodes
}

#[cfg(target_os = "linux")]
#[test]
fn absent_status_is_creation_free() {
    let root = tempfile::tempdir().unwrap();
    let error = ledger(root.path()).status().unwrap_err();
    assert_eq!(error.code(), "COORD_NOT_INITIALIZED");
    assert!(!root.path().join(".bullet-family").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn genesis_append_and_request_lookup_are_exact() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let initialized = initialize(&ledger);
    let generation = initialized.watermark.generation_id.clone();
    assert_eq!(initialized.watermark.last_sequence, 1);
    assert_eq!(initialized.records.len(), 1);

    let outcome = ledger
        .append(&generation, &request('a'), &heartbeat("one"))
        .unwrap();
    assert_eq!(outcome.receipt.generation_id, generation);
    assert_eq!(outcome.receipt.request_id, request('a'));
    let status = ledger.status().unwrap();
    assert_eq!(status.records.len(), 2);
    assert_eq!(status.watermark.last_sequence, 2);
    assert_eq!(status.watermark.last_request_id, request('a'));
    assert_eq!(status.request(&request('a')), Some(&outcome.receipt));
}

#[cfg(target_os = "linux")]
#[test]
fn stale_generation_and_conflicting_request_do_not_mutate() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let view = initialize(&ledger);
    let generation = view.watermark.generation_id;
    let source = view.source;
    let before = fs::metadata(&source).unwrap().len();
    assert_eq!(
        ledger
            .append(
                &format!("gen_{}", "f".repeat(64)),
                &request('b'),
                &heartbeat("x"),
            )
            .unwrap_err()
            .code(),
        "COORD_SUBJECT_CHANGED"
    );
    assert_eq!(fs::metadata(&source).unwrap().len(), before);

    ledger
        .append(&generation, &request('c'), &heartbeat("one"))
        .unwrap();
    let committed = fs::metadata(&source).unwrap().len();
    assert_eq!(
        ledger
            .append(&generation, &request('c'), &heartbeat("two"),)
            .unwrap_err()
            .code(),
        "COORD_REQUEST_CONFLICT"
    );
    assert_eq!(fs::metadata(&source).unwrap().len(), committed);
}

#[cfg(target_os = "linux")]
#[test]
fn exact_or_changed_genesis_retry_never_reinvokes_clock() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let first = initialize(&ledger);
    let source_length = fs::metadata(&first.source).unwrap().len();
    let exact = ledger
        .initialize_genesis(&provenance(), || panic!("clock invoked on exact retry"))
        .unwrap();
    assert_eq!(exact.watermark, first.watermark);

    let mut changed = provenance();
    changed.operator = "other-operator".to_owned();
    let invoked = Cell::new(false);
    let error = ledger
        .initialize_genesis(&changed, || {
            invoked.set(true);
            Ok(99)
        })
        .unwrap_err();
    assert_eq!(error.code(), "COORD_GENESIS_CONFLICT");
    assert!(!invoked.get());
    assert_eq!(fs::metadata(&first.source).unwrap().len(), source_length);
}

#[cfg(target_os = "linux")]
#[test]
fn sealed_initialization_intent_stage_recovers_chosen_manifest_without_clock() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let coord = coord_path(root.path());
    super::fs::ensure_layout(root.path(), &coord).unwrap();
    let prepared = super::genesis::prepare(&provenance(), 44).unwrap();
    let stage = coord.join(staged_name(
        "genesis-init-intent.json",
        &prepared.intent_bytes,
    ));
    fs::write(&stage, &prepared.intent_bytes).unwrap();
    set_mode(&stage, 0o400);

    let view = ledger
        .initialize_genesis(&provenance(), || panic!("clock invoked for sealed intent"))
        .unwrap();
    assert_eq!(
        view.watermark.generation_id,
        prepared.manifest.generation_id().as_str()
    );
    assert_eq!(
        fs::read(coord.join("genesis-init-intent.json")).unwrap(),
        prepared.intent_bytes
    );
    assert!(!stage.exists());
}

#[cfg(target_os = "linux")]
#[test]
fn partial_initialization_intent_stage_is_preserved_unknown_without_clock() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let coord = coord_path(root.path());
    super::fs::ensure_layout(root.path(), &coord).unwrap();
    let abandoned = super::genesis::prepare(&provenance(), 44).unwrap();
    let stage = coord.join(staged_name(
        "genesis-init-intent.json",
        &abandoned.intent_bytes,
    ));
    fs::write(&stage, b"partial").unwrap();
    set_mode(&stage, 0o600);
    let calls = Cell::new(0);
    let error = ledger
        .initialize_genesis(&provenance(), || {
            calls.set(calls.get() + 1);
            Ok(45)
        })
        .unwrap_err();
    assert_eq!(error.code(), "COORD_FENCE_UNKNOWN");
    assert_eq!(calls.get(), 0);
    assert_eq!(fs::read(&stage).unwrap(), b"partial");
    assert!(!coord.join("CURRENT").exists());
    assert!(!coord.join("events.jsonl").exists());
}

#[cfg(target_os = "linux")]
#[test]
fn legacy_mutation_preflight_is_typed_and_byte_exact_creation_free() {
    let root = tempfile::tempdir().unwrap();
    let coord = coord_path(root.path());
    fs::create_dir_all(&coord).unwrap();
    set_mode(&root.path().join(".bullet-family"), 0o700);
    set_mode(&coord, 0o775);
    fs::write(coord.join("events.jsonl"), b"legacy-exact\n").unwrap();
    set_mode(&coord.join("events.jsonl"), 0o400);
    let before = snapshot(&coord);

    let error = ledger(root.path())
        .transact(&format!("gen_{}", "1".repeat(64)), &request('7'), |_| {
            panic!("legacy mutation decision invoked")
        })
        .unwrap_err();
    assert_eq!(error.code(), "COORD_RECOVERY_REQUIRED");
    assert_eq!(snapshot(&coord), before);
    assert!(!coord.join("LOCK").exists());

    let error = ledger(root.path())
        .initialize_genesis(&provenance(), || panic!("legacy Genesis clock invoked"))
        .unwrap_err();
    assert_eq!(error.code(), "COORD_RECOVERY_REQUIRED");
    assert_eq!(snapshot(&coord), before);
}

#[cfg(target_os = "linux")]
#[test]
fn retry_returns_exact_transaction_without_invoking_decision() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let initialized = initialize(&ledger);
    let generation = initialized.watermark.generation_id;
    let request_id = request('d');
    let first = ledger
        .transact(&generation, &request_id, |view| {
            assert_eq!(view.records.len(), 1);
            Ok(heartbeat("created"))
        })
        .unwrap();
    assert!(!first.existing);
    let original_records = first.request_records().unwrap().to_vec();
    ledger
        .append(&generation, &request('9'), &heartbeat("later"))
        .unwrap();

    let invoked = Cell::new(false);
    let retry = ledger
        .transact(&generation, &request_id, |_| {
            invoked.set(true);
            Ok(heartbeat("must-not-run"))
        })
        .unwrap();
    assert!(retry.existing);
    assert!(!invoked.get());
    assert_eq!(
        canonical_record(&retry.record),
        canonical_record(&first.record)
    );
    assert_eq!(retry.receipt, first.receipt);
    assert_eq!(retry.watermark, first.watermark);
    assert_eq!(
        canonical_records(retry.request_records().unwrap()),
        canonical_records(&original_records)
    );
    assert_ne!(retry.view.watermark, retry.watermark);
}

#[cfg(target_os = "linux")]
#[test]
fn concurrent_transactions_decide_from_serialized_locked_views() {
    let root = tempfile::tempdir().unwrap();
    let initialized = initialize(&ledger(root.path()));
    let generation = initialized.watermark.generation_id;
    let root_path = root.path().to_owned();
    let barrier = Arc::new(Barrier::new(3));
    let mut handles = Vec::new();
    for fill in ['e', 'f'] {
        let root_path = root_path.clone();
        let generation = generation.clone();
        let barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            barrier.wait();
            ledger(&root_path)
                .transact(&generation, &request(fill), |view| {
                    Ok(heartbeat(&format!("seen-{}", view.records.len())))
                })
                .unwrap()
                .record
        }));
    }
    barrier.wait();
    let mut notes = handles
        .into_iter()
        .map(|handle| match handle.join().unwrap() {
            Record::Heartbeat { note, .. } => note.unwrap(),
            other => panic!("unexpected record: {other:?}"),
        })
        .collect::<Vec<_>>();
    notes.sort();
    assert_eq!(notes, ["seen-1", "seen-2"]);
    assert_eq!(ledger(root.path()).status().unwrap().records.len(), 3);
}

#[cfg(target_os = "linux")]
#[test]
fn partial_current_stage_is_repaired_before_publication() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let manifest = manifest();
    let pointer = CurrentPointer::for_manifest(&manifest).unwrap();
    let bytes = pointer.canonical_bytes().unwrap();
    super::fs::ensure_layout(root.path(), &root.path().join(COORD_CHILD)).unwrap();
    let coord = root.path().join(COORD_CHILD);
    let stage = coord.join(staged_name("CURRENT", &bytes));
    fs::write(&stage, &bytes[..bytes.len() / 2]).unwrap();
    set_mode(&stage, 0o600);

    let view = initialize(&ledger);
    assert_eq!(view.watermark.manifest_blake3, pointer.manifest_blake3());
    assert!(!stage.exists());
    assert_eq!(fs::read(coord.join("CURRENT")).unwrap(), bytes);
}

#[cfg(target_os = "linux")]
#[test]
fn partial_manifest_stage_and_retired_restart_are_reconciled() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let manifest = manifest();
    let coord = root.path().join(COORD_CHILD);
    super::fs::ensure_layout(root.path(), &coord).unwrap();
    let staging = coord
        .join("generations")
        .join(format!(".next-{}", manifest.generation_id().as_str()));
    fs::create_dir(coord.join("generations")).unwrap();
    fs::create_dir(&staging).unwrap();
    set_mode(coord.join("generations").as_path(), 0o700);
    set_mode(&staging, 0o700);
    let manifest_bytes = manifest.canonical_bytes().unwrap();
    let partial = staging.join(staged_name("manifest.json", &manifest_bytes));
    fs::write(&partial, &manifest_bytes[..manifest_bytes.len() / 2]).unwrap();
    set_mode(&partial, 0o600);

    let initialized = initialize(&ledger);
    assert_eq!(initialized.watermark.last_sequence, 1);
    assert!(!partial.exists());
    let tombstone = coord.join("events.jsonl");
    assert!(fs::symlink_metadata(&tombstone).unwrap().is_dir());
    assert_eq!(mode(&tombstone), 0);
}

#[cfg(target_os = "linux")]
#[test]
fn legacy_and_retired_status_are_typed_and_creation_free() {
    let root = tempfile::tempdir().unwrap();
    let coord = root.path().join(COORD_CHILD);
    fs::create_dir_all(&coord).unwrap();
    set_mode(root.path().join(".bullet-family").as_path(), 0o700);
    set_mode(&coord, 0o775);
    fs::write(coord.join("events.jsonl"), b"legacy\n").unwrap();
    set_mode(coord.join("events.jsonl").as_path(), 0o400);
    assert_eq!(
        ledger(root.path()).status().unwrap_err().code(),
        "COORD_RECOVERY_REQUIRED"
    );
    assert!(!coord.join("LOCK").exists());

    fs::remove_file(coord.join("events.jsonl")).unwrap();
    fs::create_dir(coord.join("events.jsonl")).unwrap();
    set_mode(coord.join("events.jsonl").as_path(), 0);
    set_mode(&coord, 0o700);
    assert_eq!(
        ledger(root.path()).status().unwrap_err().code(),
        "COORD_RECOVERY_IN_PROGRESS"
    );
    assert!(!coord.join("LOCK").exists());
}
