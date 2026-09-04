use super::*;

use crate::coord::generation::segment::{self, AppendRequest};

#[test]
fn record_constructor_cannot_mutate_genesis_authority_before_append() {
    for variant in ["current", "intent", "tombstone"] {
        let root = tempfile::tempdir().unwrap();
        let ledger = ledger(root.path());
        let initialized = initialize(&ledger);
        let generation = initialized.watermark.generation_id;
        let segment = initialized.source;
        let segment_length = fs::metadata(&segment).unwrap().len();
        let pending = segment.parent().unwrap().join("pending");
        let pending_before = snapshot(&pending);
        let coord = coord_path(root.path());
        if variant == "current" {
            super::super::transaction::test_replace_current_during_genesis_authority(
                coord.join("CURRENT"),
            );
        }

        let error = ledger
            .transact(&generation, &request('8'), |_| {
                match variant {
                    "current" => {}
                    "intent" => tamper_canonical_file(&coord.join("genesis-init-intent.json")),
                    "tombstone" => set_mode(&coord.join("events.jsonl"), 0o700),
                    _ => unreachable!(),
                }
                Ok(heartbeat("must not append"))
            })
            .unwrap_err();

        let expected = if variant == "current" {
            "COORD_SUBJECT_CHANGED"
        } else {
            "COORD_FENCE_UNKNOWN"
        };
        assert_eq!(error.code(), expected, "variant={variant}");
        assert_eq!(fs::metadata(&segment).unwrap().len(), segment_length);
        assert_eq!(snapshot(&pending), pending_before, "variant={variant}");
    }
}

#[test]
fn pending_reconciliation_revalidates_genesis_authority_before_effect() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let initialized = initialize(&ledger);
    let generation = initialized.watermark.generation_id.clone();
    let segment_path = initialized.source;
    let pending = segment_path.parent().unwrap().join("pending");
    let pending_record = heartbeat("pending must remain uncommitted");
    let pending_request_id = request('6');
    let genesis_digest = initialized
        .watermark
        .manifest_blake3
        .strip_prefix("blake3:")
        .unwrap()
        .to_owned();
    let pending_request = AppendRequest {
        generation_id: &generation,
        sequence: initialized.watermark.next_sequence,
        previous_digest: &initialized.watermark.head_envelope_digest,
        request_id: &pending_request_id,
        record: &pending_record,
    };
    segment::test_crash_after_intent_link();
    assert_eq!(
        segment::append(&segment_path, &pending, &pending_request, &genesis_digest)
            .unwrap_err()
            .code(),
        "COORD_TEST_CRASH"
    );
    let segment_before = fs::read(&segment_path).unwrap();
    let pending_before = snapshot(&pending);

    super::super::test_mutate_genesis_after_first_validation();
    let error = ledger
        .transact(&generation, &request('7'), |_| {
            panic!("record constructor ran before pending authority reconciliation")
        })
        .unwrap_err();

    assert_eq!(error.code(), "COORD_FENCE_UNKNOWN");
    assert_eq!(fs::read(segment_path).unwrap(), segment_before);
    assert_eq!(snapshot(&pending), pending_before);
}

#[test]
fn pending_reconciliation_revalidates_final_genesis_subject_before_effect() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let initialized = initialize(&ledger);
    let generation = initialized.watermark.generation_id.clone();
    let segment_path = initialized.source;
    let pending = segment_path.parent().unwrap().join("pending");
    let pending_record = heartbeat("pending must remain uncommitted");
    let pending_request_id = request('3');
    let genesis_digest = initialized
        .watermark
        .manifest_blake3
        .strip_prefix("blake3:")
        .unwrap()
        .to_owned();
    let pending_request = AppendRequest {
        generation_id: &generation,
        sequence: initialized.watermark.next_sequence,
        previous_digest: &initialized.watermark.head_envelope_digest,
        request_id: &pending_request_id,
        record: &pending_record,
    };
    segment::test_crash_after_intent_link();
    assert_eq!(
        segment::append(&segment_path, &pending, &pending_request, &genesis_digest)
            .unwrap_err()
            .code(),
        "COORD_TEST_CRASH"
    );
    let segment_before = fs::read(&segment_path).unwrap();
    let pending_before = snapshot(&pending);
    arm_pre_reconcile_current_swap(&coord_path(root.path()));

    let error = ledger
        .transact(&generation, &request('4'), |_| {
            panic!("record constructor ran before pending subject reconciliation")
        })
        .unwrap_err();

    assert_eq!(error.code(), "COORD_SUBJECT_CHANGED");
    assert_eq!(fs::read(segment_path).unwrap(), segment_before);
    assert_eq!(snapshot(&pending), pending_before);
}

#[test]
fn final_genesis_validation_catches_mutation_after_the_first_probe() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let initialized = initialize(&ledger);
    let segment = initialized.source;
    let pending = segment.parent().unwrap().join("pending");
    let segment_before = fs::read(&segment).unwrap();
    let pending_before = snapshot(&pending);

    super::super::test_mutate_genesis_after_first_validation();
    let error = ledger.status().unwrap_err();

    assert_eq!(error.code(), "COORD_FENCE_UNKNOWN");
    assert_eq!(fs::read(segment).unwrap(), segment_before);
    assert_eq!(snapshot(&pending), pending_before);
}

#[test]
fn final_replay_reloads_same_inode_manifest_bytes() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let initialized = initialize(&ledger);
    let segment = initialized.source;
    let pending = segment.parent().unwrap().join("pending");
    let segment_before = fs::read(&segment).unwrap();
    let pending_before = snapshot(&pending);
    super::super::test_rewrite_manifest_before_final_replay(
        segment.parent().unwrap().join("manifest.json"),
    );

    let error = ledger.status().unwrap_err();

    assert_eq!(error.code(), "INVALID_COORD_GENERATION");
    assert_eq!(fs::read(segment).unwrap(), segment_before);
    assert_eq!(snapshot(&pending), pending_before);
}

#[test]
fn generation_swap_after_append_never_returns_false_success() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let initialized = initialize(&ledger);
    let generation = initialized.watermark.generation_id;
    let canonical = initialized.source.parent().unwrap().to_owned();
    let replacement = canonical.with_file_name(".replacement-generation-test");
    clone_genesis_generation(&canonical, &replacement);
    super::super::transaction::test_swap_generation_after_append(canonical.clone(), replacement);
    let request_id = request('5');

    let error = ledger
        .transact(&generation, &request_id, |_| {
            Ok(heartbeat("must never become false success"))
        })
        .unwrap_err();

    assert_eq!(error.code(), "COORD_SUBJECT_CHANGED");
    let canonical_status = ledger.status().unwrap();
    assert_eq!(canonical_status.watermark.last_sequence, 1);
    assert!(canonical_status.request(&request_id).is_none());
}

#[test]
fn final_subject_swap_refuses_success_after_durable_append() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let initialized = initialize(&ledger);
    let generation = initialized.watermark.generation_id;
    let coord = coord_path(root.path());
    let current = coord.join("CURRENT");
    let replacement = coord.join(".CURRENT.replacement-return-test");
    fs::copy(&current, &replacement).unwrap();
    set_mode(&replacement, 0o400);
    super::super::transaction::test_swap_subject_before_return(current, replacement);
    let request_id = request('2');

    let error = ledger
        .transact(&generation, &request_id, |_| {
            Ok(heartbeat("durable effect must still return non-green"))
        })
        .unwrap_err();

    assert_eq!(error.code(), "COORD_SUBJECT_CHANGED");
    let canonical_status = ledger.status().unwrap();
    assert_eq!(canonical_status.watermark.last_sequence, 2);
    assert!(canonical_status.request(&request_id).is_some());
}

#[test]
fn existing_request_final_subject_swap_never_returns_success() {
    let root = tempfile::tempdir().unwrap();
    let ledger = ledger(root.path());
    let initialized = initialize(&ledger);
    let generation = initialized.watermark.generation_id;
    let request_id = request('1');
    ledger
        .append(
            &generation,
            &request_id,
            &heartbeat("existing durable request"),
        )
        .unwrap();
    let coord = coord_path(root.path());
    let current = coord.join("CURRENT");
    let replacement = coord.join(".CURRENT.replacement-existing-return-test");
    fs::copy(&current, &replacement).unwrap();
    set_mode(&replacement, 0o400);
    super::super::transaction::test_swap_subject_before_return(current, replacement);

    let error = ledger
        .transact(&generation, &request_id, |_| {
            panic!("existing request decision closure must not run")
        })
        .unwrap_err();

    assert_eq!(error.code(), "COORD_SUBJECT_CHANGED");
    assert!(ledger.status().unwrap().request(&request_id).is_some());
}

fn clone_genesis_generation(source: &Path, destination: &Path) {
    fs::create_dir(destination).unwrap();
    set_mode(destination, 0o700);
    for (name, mode) in [("manifest.json", 0o400), ("events.jsonl", 0o600)] {
        fs::copy(source.join(name), destination.join(name)).unwrap();
        set_mode(&destination.join(name), mode);
    }
    fs::create_dir(destination.join("pending")).unwrap();
    set_mode(&destination.join("pending"), 0o700);
}

fn arm_pre_reconcile_current_swap(coord: &Path) {
    let current = coord.join("CURRENT");
    let replacement = coord.join(".CURRENT.replacement-reconcile-test");
    fs::copy(&current, &replacement).unwrap();
    set_mode(&replacement, 0o400);
    super::super::test_swap_subject_before_pending_reconcile(current, replacement);
}
