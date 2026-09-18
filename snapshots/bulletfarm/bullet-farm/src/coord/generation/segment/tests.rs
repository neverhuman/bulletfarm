use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use super::*;
use crate::coord::model::{GENERATION_SCHEMA_VERSION, RecoveryBaselineBody};

const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const GEN: &str = "gen_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

struct Fixture {
    _root: tempfile::TempDir,
    segment: PathBuf,
    pending: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("temporary segment fixture");
        let segment = root.path().join("records.v2.jsonl");
        let pending = root.path().join("pending");
        fs::write(&segment, []).expect("create segment");
        fs::create_dir(&pending).expect("create pending directory");
        set_mode(&segment, 0o600);
        set_mode(&pending, 0o700);
        Self {
            _root: root,
            segment,
            pending,
        }
    }
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("set fixture mode");
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) {}

fn record(note: &str) -> Record {
    Record::Heartbeat {
        schema_version: GENERATION_SCHEMA_VERSION,
        at_unix_ms: 1,
        claim_id: "clm_test".to_owned(),
        agent: "codex-test".to_owned(),
        expires_unix_ms: 2,
        note: Some(note.to_owned()),
    }
}

fn baseline(marker: char) -> Record {
    Record::RecoveryBaselineV2 {
        schema_version: GENERATION_SCHEMA_VERSION,
        generation_id: GEN.to_owned(),
        body: RecoveryBaselineBody {
            manifest_blake3: format!("blake3:{}", marker.to_string().repeat(64)),
            incident_at_unix_ms: 1,
            recovered_at_unix_ms: 2,
            trusted_state_blake3: format!("blake3:{}", marker.to_string().repeat(64)),
            frozen_claims: Vec::new(),
        },
    }
}

fn genesis(generation_id: &str) -> Record {
    Record::GenesisV2 {
        schema_version: GENERATION_SCHEMA_VERSION,
        generation_id: generation_id.to_owned(),
        manifest_blake3: "blake3:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
            .to_owned(),
        created_at_unix_ms: 1,
    }
}

fn request<'a>(
    sequence: u64,
    previous_digest: &'a str,
    request_id: &'a str,
    record: &'a Record,
) -> AppendRequest<'a> {
    AppendRequest {
        generation_id: GEN,
        sequence,
        previous_digest,
        request_id,
        record,
    }
}

fn pending_intent(request: &AppendRequest<'_>, offset: u64) -> PendingIntent {
    let frame = encode_frame(request).expect("encode frame");
    PendingIntent {
        kind: "coord_segment_append_intent_v2".to_owned(),
        schema_version: 2,
        generation_id: request.generation_id.to_owned(),
        sequence: request.sequence,
        previous_digest: request.previous_digest.to_owned(),
        request_id: request.request_id.to_owned(),
        request_digest: append_request_digest(request).expect("digest append request"),
        segment_offset: offset,
        frame_length: frame.len() as u64,
        frame_digest: frame_digest(&frame).expect("digest frame"),
        frame_utf8: String::from_utf8(frame).expect("canonical UTF-8"),
    }
}

#[test]
fn append_is_durable_chained_and_exactly_idempotent() {
    let fixture = Fixture::new();
    let first_record = baseline('1');
    let first = request(1, GENESIS, "req-1", &first_record);
    let receipt =
        append(&fixture.segment, &fixture.pending, &first, GENESIS).expect("append first");
    assert_eq!(receipt.sequence, 1);
    assert_eq!(fs::read_dir(&fixture.pending).unwrap().count(), 0);

    let length = fs::metadata(&fixture.segment).unwrap().len();
    assert_eq!(
        append(&fixture.segment, &fixture.pending, &first, GENESIS).unwrap(),
        receipt
    );
    assert_eq!(fs::metadata(&fixture.segment).unwrap().len(), length);

    let second_record = record("second");
    let second = request(2, &receipt.envelope_digest, "req-2", &second_record);
    append(&fixture.segment, &fixture.pending, &second, GENESIS).expect("append second");
    let inspected = inspect(&fixture.segment, &fixture.pending, GEN, GENESIS).expect("inspect");
    assert_eq!(inspected.entries.len(), 2);
    assert_eq!(inspected.position.next_sequence, 3);
    assert_eq!(
        inspected.position.byte_length,
        fs::metadata(&fixture.segment).unwrap().len()
    );
}

#[test]
fn same_request_id_with_changed_digest_fails_closed() {
    let fixture = Fixture::new();
    let value = baseline('3');
    let changed_value = baseline('4');
    let original = request(1, GENESIS, "req-conflict", &value);
    append(&fixture.segment, &fixture.pending, &original, GENESIS).unwrap();
    let changed = request(1, GENESIS, "req-conflict", &changed_value);
    assert_eq!(
        append(&fixture.segment, &fixture.pending, &changed, GENESIS)
            .unwrap_err()
            .code(),
        "COORD_REQUEST_CONFLICT"
    );
}

#[test]
fn every_absent_prefix_and_complete_frame_reconciles() {
    let value = baseline('5');
    let request = request(1, GENESIS, "req-prefix", &value);
    let intended = pending_intent(&request, 0);
    let frame = intended.frame_utf8.as_bytes();
    for cut in 0..=frame.len() {
        let fixture = Fixture::new();
        io::publish_intent(&fixture.pending, &intended).expect("publish intent");
        fs::write(&fixture.segment, &frame[..cut]).expect("inject exact prefix");
        let position = reconcile_pending(&fixture.segment, &fixture.pending, GEN, GENESIS)
            .expect("reconcile exact prefix");
        assert_eq!(position.next_sequence, 2, "cut {cut}");
        assert_eq!(fs::read(&fixture.segment).unwrap(), frame, "cut {cut}");
        assert_eq!(
            fs::read_dir(&fixture.pending).unwrap().count(),
            0,
            "cut {cut}"
        );
    }
}

#[test]
fn every_anonymous_intent_write_offset_is_absent_and_retryable() {
    let value = baseline('2');
    let request = request(1, GENESIS, "req-intent-offset", &value);
    let intent = pending_intent(&request, 0);
    let intent_length = bullet_wire::canonical_json(&intent).unwrap().len();
    for cut in 0..=intent_length {
        let fixture = Fixture::new();
        io::test_crash_at_offset(cut);
        let error = append(&fixture.segment, &fixture.pending, &request, GENESIS).unwrap_err();
        assert_eq!(error.code(), "COORD_TEST_CRASH", "cut={cut}");
        assert_eq!(
            fs::metadata(&fixture.segment).unwrap().len(),
            0,
            "cut={cut}"
        );
        assert_eq!(
            fs::read_dir(&fixture.pending).unwrap().count(),
            0,
            "cut={cut}"
        );
        append(&fixture.segment, &fixture.pending, &request, GENESIS)
            .unwrap_or_else(|error| panic!("cut={cut}: {error:?}"));
    }
}

#[test]
fn crash_after_intent_link_is_exact_complete_and_reconciles_once() {
    let fixture = Fixture::new();
    let value = baseline('9');
    let request = request(1, GENESIS, "req-after-link", &value);
    io::test_crash_after_link();
    let error = append(&fixture.segment, &fixture.pending, &request, GENESIS).unwrap_err();
    assert_eq!(error.code(), "COORD_TEST_CRASH");
    assert_eq!(fs::metadata(&fixture.segment).unwrap().len(), 0);
    assert_eq!(fs::read_dir(&fixture.pending).unwrap().count(), 1);

    let position = reconcile_pending(&fixture.segment, &fixture.pending, GEN, GENESIS).unwrap();
    assert_eq!(position.next_sequence, 2);
    assert_eq!(fs::read_dir(&fixture.pending).unwrap().count(), 0);
    assert_eq!(
        inspect(&fixture.segment, &fixture.pending, GEN, GENESIS)
            .unwrap()
            .entries
            .len(),
        1
    );
}

#[test]
fn divergent_or_longer_tail_preserves_intent_and_segment() {
    let value = baseline('6');
    let request = request(1, GENESIS, "req-divergent", &value);
    let intended = pending_intent(&request, 0);
    let subjects = vec![
        b"x".to_vec(),
        [intended.frame_utf8.as_bytes(), b"x"].concat(),
    ];
    for bytes in subjects {
        let fixture = Fixture::new();
        io::publish_intent(&fixture.pending, &intended).unwrap();
        fs::write(&fixture.segment, &bytes).unwrap();
        assert_eq!(
            reconcile_pending(&fixture.segment, &fixture.pending, GEN, GENESIS)
                .unwrap_err()
                .code(),
            "PARTIAL_COORD_WRITE"
        );
        assert_eq!(fs::read(&fixture.segment).unwrap(), bytes);
        assert!(fixture.pending.join(INTENT_NAME).exists());
    }
}

#[test]
fn status_refuses_pending_and_chain_corruption() {
    let fixture = Fixture::new();
    let value = baseline('7');
    let first = request(1, GENESIS, "req-pending", &value);
    io::publish_intent(&fixture.pending, &pending_intent(&first, 0)).unwrap();
    assert_eq!(
        inspect(&fixture.segment, &fixture.pending, GEN, GENESIS)
            .unwrap_err()
            .code(),
        "PENDING_COORD_APPEND"
    );

    reconcile_pending(&fixture.segment, &fixture.pending, GEN, GENESIS).unwrap();
    let wrong_record = record("wrong predecessor");
    let wrong_previous = "8".repeat(64);
    let wrong = request(2, &wrong_previous, "req-wrong", &wrong_record);
    let mut segment = OpenOptions::new()
        .append(true)
        .open(&fixture.segment)
        .unwrap();
    segment.write_all(&encode_frame(&wrong).unwrap()).unwrap();
    segment.sync_data().unwrap();
    assert_eq!(
        inspect(&fixture.segment, &fixture.pending, GEN, GENESIS)
            .unwrap_err()
            .code(),
        "CORRUPT_COORD_SEGMENT"
    );
}

#[test]
fn generation_id_requires_the_manifest_form() {
    let fixture = Fixture::new();
    assert_eq!(
        inspect(&fixture.segment, &fixture.pending, &"a".repeat(64), GENESIS,)
            .unwrap_err()
            .code(),
        "INVALID_COORD_GENERATION"
    );
}

#[test]
fn empty_status_is_creation_free() {
    let fixture = Fixture::new();
    let before = fs::read_dir(fixture._root.path()).unwrap().count();
    assert_eq!(
        inspect(&fixture.segment, &fixture.pending, GEN, GENESIS)
            .unwrap_err()
            .code(),
        "EMPTY_COORD_SEGMENT"
    );
    assert_eq!(fs::read_dir(fixture._root.path()).unwrap().count(), before);
    assert_eq!(fs::metadata(&fixture.segment).unwrap().len(), 0);
    assert_eq!(fs::read_dir(&fixture.pending).unwrap().count(), 0);
}

#[test]
fn baseline_position_and_post_baseline_schema_are_closed() {
    let not_baseline = record("wrong at sequence one");
    let wrong_first = request(1, GENESIS, "req-wrong-first", &not_baseline);
    assert_eq!(
        validate_append_request(&wrong_first, GENESIS)
            .unwrap_err()
            .code(),
        "INVALID_COORD_BASELINE"
    );

    let first = baseline('a');
    let wrong_previous = "b".repeat(64);
    let wrong_genesis = request(1, &wrong_previous, "req-wrong-genesis", &first);
    assert_eq!(
        validate_append_request(&wrong_genesis, GENESIS)
            .unwrap_err()
            .code(),
        "COORD_SEGMENT_POSITION_MISMATCH"
    );

    let legacy = Record::Heartbeat {
        schema_version: 1,
        at_unix_ms: 1,
        claim_id: "clm_legacy".to_owned(),
        agent: "legacy".to_owned(),
        expires_unix_ms: 2,
        note: None,
    };
    let legacy_request = request(2, GENESIS, "req-legacy", &legacy);
    assert_eq!(
        validate_append_request(&legacy_request, GENESIS)
            .unwrap_err()
            .code(),
        "UNSUPPORTED_SCHEMA"
    );
}

#[test]
fn genesis_is_the_other_closed_sequence_one_form() {
    let fixture = Fixture::new();
    let value = genesis(GEN);
    let first = request(1, GENESIS, "req-genesis", &value);
    validate_append_request(&first, GENESIS).expect("preflight genesis");
    let receipt = append(&fixture.segment, &fixture.pending, &first, GENESIS)
        .expect("append schema-2 genesis");
    let inspected = inspect(&fixture.segment, &fixture.pending, GEN, GENESIS).unwrap();
    assert_eq!(inspected.entries.len(), 1);
    assert!(matches!(
        inspected.entries[0].record,
        Record::GenesisV2 { .. }
    ));

    let wrong_generation =
        genesis("gen_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    let wrong = request(1, GENESIS, "req-wrong-generation", &wrong_generation);
    assert_eq!(
        validate_append_request(&wrong, GENESIS).unwrap_err().code(),
        "INVALID_COORD_BASELINE"
    );

    let repeated = request(2, &receipt.envelope_digest, "req-repeated-genesis", &value);
    assert_eq!(
        validate_append_request(&repeated, GENESIS)
            .unwrap_err()
            .code(),
        "INVALID_COORD_BASELINE"
    );
}

#[test]
fn request_digest_is_deterministic_and_subject_sensitive() {
    let first = baseline('c');
    let changed = baseline('d');
    let subject = request(1, GENESIS, "req-digest", &first);
    let same = request(1, GENESIS, "req-digest", &first);
    let changed = request(1, GENESIS, "req-digest", &changed);
    assert_eq!(
        append_request_digest(&subject).unwrap(),
        append_request_digest(&same).unwrap()
    );
    assert_ne!(
        append_request_digest(&subject).unwrap(),
        append_request_digest(&changed).unwrap()
    );
}

#[test]
fn segment_capacity_is_enforced_before_parse() {
    let fixture = Fixture::new();
    let file = OpenOptions::new()
        .write(true)
        .open(&fixture.segment)
        .unwrap();
    file.set_len(MAX_SEGMENT_BYTES + 1).unwrap();
    assert_eq!(
        inspect(&fixture.segment, &fixture.pending, GEN, GENESIS)
            .unwrap_err()
            .code(),
        "COORD_SEGMENT_CAPACITY_EXCEEDED"
    );
}

#[cfg(unix)]
#[test]
fn pending_symlink_is_never_followed() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    let real = fixture._root.path().join("real-pending");
    fs::create_dir(&real).unwrap();
    set_mode(&real, 0o700);
    fs::remove_dir(&fixture.pending).unwrap();
    symlink(&real, &fixture.pending).unwrap();
    assert_eq!(
        inspect(&fixture.segment, &fixture.pending, GEN, GENESIS)
            .unwrap_err()
            .code(),
        "CORRUPT_COORD_PENDING"
    );
}

#[cfg(unix)]
#[test]
fn descriptor_append_cannot_be_redirected_by_path_replacement() {
    let fixture = Fixture::new();
    let mut segment = open_segment(&fixture.segment, true).unwrap();
    let pending = open_pending(&fixture.pending).unwrap();
    let retained_segment = fixture._root.path().join("retained-segment");
    let retained_pending = fixture._root.path().join("retained-pending");
    fs::rename(&fixture.segment, &retained_segment).unwrap();
    fs::rename(&fixture.pending, &retained_pending).unwrap();
    fs::write(&fixture.segment, []).unwrap();
    fs::create_dir(&fixture.pending).unwrap();
    set_mode(&fixture.segment, 0o600);
    set_mode(&fixture.pending, 0o700);

    let value = baseline('f');
    let request = request(1, GENESIS, "req-retained", &value);
    append_files(&mut segment, &pending, &request, GENESIS).unwrap();
    let inspected = inspect_files(&mut segment, &pending, GEN, GENESIS).unwrap();

    assert_eq!(inspected.entries.len(), 1);
    assert!(!fs::read(&retained_segment).unwrap().is_empty());
    assert_eq!(fs::read_dir(&retained_pending).unwrap().count(), 0);
    assert!(fs::read(&fixture.segment).unwrap().is_empty());
    assert_eq!(fs::read_dir(&fixture.pending).unwrap().count(), 0);
}
