use std::path::{Path, PathBuf};

use super::*;
use crate::coord::{
    generation::segment,
    store::ledger::{
        test_mutate_recovery_after_first_validation, test_swap_subject_before_pending_reconcile,
        test_swap_subject_before_return,
    },
};

#[derive(Clone, Copy, Debug)]
enum PublishedRecoveryTamper {
    TombstoneModeRollback,
    TombstoneNonempty,
    TombstoneSubstitution,
    RetiredSource,
    RetiredSourceMissing,
    RetiredSourceReplacement,
    TransientSibling,
    IntentMissing,
    IntentCorrupt,
    PreparedObservationMissing,
    PreparedObservationCorrupt,
    TombstoneObservationCorrupt,
    RetirementObservationMissing,
    RetirementObservationCorrupt,
}

impl PublishedRecoveryTamper {
    const ALL: [Self; 14] = [
        Self::TombstoneModeRollback,
        Self::TombstoneNonempty,
        Self::TombstoneSubstitution,
        Self::RetiredSource,
        Self::RetiredSourceMissing,
        Self::RetiredSourceReplacement,
        Self::TransientSibling,
        Self::IntentMissing,
        Self::IntentCorrupt,
        Self::PreparedObservationMissing,
        Self::PreparedObservationCorrupt,
        Self::TombstoneObservationCorrupt,
        Self::RetirementObservationMissing,
        Self::RetirementObservationCorrupt,
    ];

    const fn expected_code(self) -> &'static str {
        match self {
            Self::TombstoneSubstitution | Self::IntentMissing | Self::IntentCorrupt => {
                "COORD_RECOVERY_INTENT_OUTCOME_UNKNOWN"
            }
            Self::TombstoneModeRollback
            | Self::PreparedObservationMissing
            | Self::PreparedObservationCorrupt
            | Self::TombstoneObservationCorrupt => "TOMBSTONE_SEAL_OUTCOME_UNKNOWN",
            Self::RetirementObservationMissing | Self::RetirementObservationCorrupt => {
                "COORD_RETIREMENT_OUTCOME_UNKNOWN"
            }
            Self::RetiredSourceMissing | Self::TransientSibling => "COORD_RECOVERY_SUBJECT_CHANGED",
            Self::TombstoneNonempty | Self::RetiredSource | Self::RetiredSourceReplacement => {
                "INVALID_COORD_RECOVERY"
            }
        }
    }
}

fn recovered_fixture() -> adoption_fixture::AdoptionRecoveryFixture {
    let fixture = adoption_fixture::fixture(&"1".repeat(40), &"2".repeat(40));
    recovery::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false)).unwrap();
    Ledger::new(fixture.family.path()).status().unwrap();
    fixture
}

fn recovery_path(fixture: &adoption_fixture::AdoptionRecoveryFixture, name: &str) -> PathBuf {
    fixture
        .family
        .path()
        .join(".bullet-family/coord/recovery")
        .join(fixture.manifest.generation_id().as_str())
        .join(name)
}

fn generation_segment(fixture: &adoption_fixture::AdoptionRecoveryFixture) -> PathBuf {
    fixture
        .family
        .path()
        .join(".bullet-family/coord/generations")
        .join(fixture.manifest.generation_id().as_str())
        .join("events.jsonl")
}

fn generation_pending(fixture: &adoption_fixture::AdoptionRecoveryFixture) -> PathBuf {
    fixture
        .family
        .path()
        .join(".bullet-family/coord/generations")
        .join(fixture.manifest.generation_id().as_str())
        .join("pending")
}

fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
}

fn rewrite(path: &Path) {
    let mut bytes = fs::read(path).unwrap();
    bytes[0] ^= 1;
    set_mode(path, 0o600);
    fs::write(path, bytes).unwrap();
    set_mode(path, 0o400);
}

fn snapshot(root: &Path) -> Vec<(PathBuf, u8, u32, Vec<u8>)> {
    use std::os::unix::fs::MetadataExt;

    fn visit(root: &Path, path: &Path, nodes: &mut Vec<(PathBuf, u8, u32, Vec<u8>)>) {
        let metadata = fs::symlink_metadata(path).unwrap();
        let kind = if metadata.is_dir() {
            1
        } else if metadata.is_file() {
            2
        } else {
            3
        };
        nodes.push((
            path.strip_prefix(root).unwrap().to_owned(),
            kind,
            metadata.mode() & 0o7777,
            if metadata.is_file() {
                fs::read(path).unwrap()
            } else {
                Vec::new()
            },
        ));
        if metadata.is_dir() && metadata.mode() & 0o100 != 0 {
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

fn apply_tamper(
    fixture: &adoption_fixture::AdoptionRecoveryFixture,
    tamper: PublishedRecoveryTamper,
) {
    let coord = fixture.family.path().join(".bullet-family/coord");
    let tombstone = coord.join("events.jsonl");
    match tamper {
        PublishedRecoveryTamper::TombstoneModeRollback => set_mode(&tombstone, 0),
        PublishedRecoveryTamper::TombstoneNonempty => {
            set_mode(&tombstone, 0o700);
            fs::write(tombstone.join("unexpected"), b"foreign").unwrap();
            set_mode(&tombstone, 0o400);
        }
        PublishedRecoveryTamper::TombstoneSubstitution => {
            fs::rename(&tombstone, coord.join("displaced-tombstone")).unwrap();
            fs::create_dir(&tombstone).unwrap();
            set_mode(&tombstone, 0o400);
        }
        PublishedRecoveryTamper::RetiredSource => {
            rewrite(&recovery_path(fixture, "retired-v1.non-authoritative"));
        }
        PublishedRecoveryTamper::RetiredSourceMissing => {
            fs::remove_file(recovery_path(fixture, "retired-v1.non-authoritative")).unwrap();
        }
        PublishedRecoveryTamper::RetiredSourceReplacement => {
            let path = recovery_path(fixture, "retired-v1.non-authoritative");
            let bytes = fs::read(&path).unwrap();
            fs::rename(&path, recovery_path(fixture, "displaced-retired-source")).unwrap();
            fs::write(&path, bytes).unwrap();
            set_mode(&path, 0o400);
        }
        PublishedRecoveryTamper::TransientSibling => {
            let source = recovery_path(fixture, "retired-v1.non-authoritative");
            let sibling = coord.join(format!(
                ".recovery-tombstone-{}",
                fixture.manifest.generation_id().as_str()
            ));
            fs::copy(source, &sibling).unwrap();
            set_mode(&sibling, 0o400);
        }
        PublishedRecoveryTamper::IntentMissing => {
            fs::remove_file(recovery_path(fixture, "intent.json")).unwrap();
        }
        PublishedRecoveryTamper::IntentCorrupt => rewrite(&recovery_path(fixture, "intent.json")),
        PublishedRecoveryTamper::PreparedObservationMissing => {
            fs::remove_file(recovery_path(
                fixture,
                "prepared-tombstone-seal-observation.json",
            ))
            .unwrap();
        }
        PublishedRecoveryTamper::PreparedObservationCorrupt => rewrite(&recovery_path(
            fixture,
            "prepared-tombstone-seal-observation.json",
        )),
        PublishedRecoveryTamper::TombstoneObservationCorrupt => {
            rewrite(&recovery_path(fixture, "tombstone-seal-observation.json"));
        }
        PublishedRecoveryTamper::RetirementObservationMissing => {
            fs::remove_file(recovery_path(
                fixture,
                "retirement-completion-observation.json",
            ))
            .unwrap();
        }
        PublishedRecoveryTamper::RetirementObservationCorrupt => rewrite(&recovery_path(
            fixture,
            "retirement-completion-observation.json",
        )),
    }
}

#[test]
fn published_recovery_hostiles_fail_with_exact_codes_without_replay_effects() {
    for tamper in PublishedRecoveryTamper::ALL {
        let fixture = recovered_fixture();
        let coord = fixture.family.path().join(".bullet-family/coord");
        apply_tamper(&fixture, tamper);
        let before = snapshot(&coord);
        let segment = generation_segment(&fixture);
        let segment_length = fs::metadata(&segment).unwrap().len();
        let error = Ledger::new(fixture.family.path()).status().unwrap_err();
        assert_eq!(error.code(), tamper.expected_code(), "{tamper:?}");
        assert_eq!(snapshot(&coord), before, "{tamper:?}");
        assert_eq!(fs::metadata(segment).unwrap().len(), segment_length);
    }
}

#[test]
fn final_recovery_validation_catches_mutation_after_the_first_probe() {
    let fixture = recovered_fixture();
    let coord = fixture.family.path().join(".bullet-family/coord");
    let segment = generation_segment(&fixture);
    let segment_length = fs::metadata(&segment).unwrap().len();
    let pending = generation_pending(&fixture);
    let pending_before = snapshot(&pending);
    let paths_before = snapshot(&coord)
        .into_iter()
        .map(|node| node.0)
        .collect::<Vec<_>>();
    test_mutate_recovery_after_first_validation();
    let error = Ledger::new(fixture.family.path()).status().unwrap_err();
    assert_eq!(error.code(), "TOMBSTONE_SEAL_OUTCOME_UNKNOWN");
    let paths_after = snapshot(&coord)
        .into_iter()
        .map(|node| node.0)
        .collect::<Vec<_>>();
    assert_eq!(paths_after, paths_before);
    assert_eq!(fs::metadata(segment).unwrap().len(), segment_length);
    assert_eq!(snapshot(&pending), pending_before);
}

#[test]
fn pending_reconciliation_revalidates_recovery_authority_before_effect() {
    let fixture = recovered_fixture();
    let ledger = Ledger::new(fixture.family.path());
    let view = ledger.status().unwrap();
    let generation = fixture.manifest.generation_id().as_str().to_owned();
    let segment = generation_segment(&fixture);
    let pending = generation_pending(&fixture);
    let pending_record = Record::Heartbeat {
        schema_version: GENERATION_SCHEMA_VERSION,
        at_unix_ms: 31,
        claim_id: fixture.claim_ids[0].clone(),
        agent: "frozen-agent".to_owned(),
        expires_unix_ms: 32,
        note: Some("pending must remain uncommitted".to_owned()),
    };
    let pending_request_id = request_id('6');
    let genesis_digest = view
        .watermark
        .manifest_blake3
        .strip_prefix("blake3:")
        .unwrap()
        .to_owned();
    let pending_request = segment::AppendRequest {
        generation_id: &generation,
        sequence: view.watermark.next_sequence,
        previous_digest: &view.watermark.head_envelope_digest,
        request_id: pending_request_id.as_str(),
        record: &pending_record,
    };
    segment::test_crash_after_intent_link();
    assert_eq!(
        segment::append(&segment, &pending, &pending_request, &genesis_digest)
            .unwrap_err()
            .code(),
        "COORD_TEST_CRASH"
    );
    let segment_before = fs::read(&segment).unwrap();
    let pending_before = snapshot(&pending);

    test_mutate_recovery_after_first_validation();
    let error = ledger
        .transact(&generation, request_id('7').as_str(), |_| {
            panic!("record constructor ran before pending authority reconciliation")
        })
        .unwrap_err();

    assert_eq!(error.code(), "TOMBSTONE_SEAL_OUTCOME_UNKNOWN");
    assert_eq!(fs::read(segment).unwrap(), segment_before);
    assert_eq!(snapshot(&pending), pending_before);
}

#[test]
fn pending_reconciliation_revalidates_final_recovery_subject_before_effect() {
    let fixture = recovered_fixture();
    let ledger = Ledger::new(fixture.family.path());
    let view = ledger.status().unwrap();
    let generation = fixture.manifest.generation_id().as_str().to_owned();
    let segment = generation_segment(&fixture);
    let pending = generation_pending(&fixture);
    let pending_record = Record::Heartbeat {
        schema_version: GENERATION_SCHEMA_VERSION,
        at_unix_ms: 31,
        claim_id: fixture.claim_ids[0].clone(),
        agent: "frozen-agent".to_owned(),
        expires_unix_ms: 32,
        note: Some("pending must remain uncommitted".to_owned()),
    };
    let pending_request_id = request_id('3');
    let genesis_digest = view
        .watermark
        .manifest_blake3
        .strip_prefix("blake3:")
        .unwrap()
        .to_owned();
    let pending_request = segment::AppendRequest {
        generation_id: &generation,
        sequence: view.watermark.next_sequence,
        previous_digest: &view.watermark.head_envelope_digest,
        request_id: pending_request_id.as_str(),
        record: &pending_record,
    };
    segment::test_crash_after_intent_link();
    assert_eq!(
        segment::append(&segment, &pending, &pending_request, &genesis_digest)
            .unwrap_err()
            .code(),
        "COORD_TEST_CRASH"
    );
    let segment_before = fs::read(&segment).unwrap();
    let pending_before = snapshot(&pending);
    let coord = fixture.family.path().join(".bullet-family/coord");
    let current = coord.join("CURRENT");
    let replacement = coord.join(".CURRENT.replacement-reconcile-test");
    fs::copy(&current, &replacement).unwrap();
    set_mode(&replacement, 0o400);
    test_swap_subject_before_pending_reconcile(current, replacement);

    let error = ledger
        .transact(&generation, request_id('4').as_str(), |_| {
            panic!("record constructor ran before pending subject reconciliation")
        })
        .unwrap_err();

    assert_eq!(error.code(), "COORD_SUBJECT_CHANGED");
    assert_eq!(fs::read(segment).unwrap(), segment_before);
    assert_eq!(snapshot(&pending), pending_before);
}

#[test]
fn recovery_return_gate_refuses_new_and_existing_request_subject_swaps() {
    for variant in ["new", "existing"] {
        let fixture = recovered_fixture();
        let ledger = Ledger::new(fixture.family.path());
        let generation = fixture.manifest.generation_id().as_str().to_owned();
        let request = request_id('0');
        let record = Record::Claim {
            schema_version: GENERATION_SCHEMA_VERSION,
            at_unix_ms: 31,
            claim_id: format!("clm_{}", "d".repeat(64)),
            agent: "return-agent".to_owned(),
            lane: "return-gate".to_owned(),
            repo: "bullet-farm".to_owned(),
            paths: vec!["return-gate.txt".to_owned()],
            expires_unix_ms: 60_031,
        };
        if variant == "existing" {
            ledger
                .append(&generation, request.as_str(), &record)
                .unwrap();
        }
        let coord = fixture.family.path().join(".bullet-family/coord");
        let current = coord.join("CURRENT");
        let replacement = coord.join(".CURRENT.replacement-return-test");
        fs::copy(&current, &replacement).unwrap();
        set_mode(&replacement, 0o400);
        test_swap_subject_before_return(current, replacement);

        let error = ledger
            .transact(&generation, request.as_str(), |_| {
                assert_eq!(variant, "new", "existing decision closure ran");
                Ok(record.clone())
            })
            .unwrap_err();

        assert_eq!(error.code(), "COORD_SUBJECT_CHANGED", "{variant}");
        assert!(
            ledger.status().unwrap().request(request.as_str()).is_some(),
            "{variant}"
        );
    }
}

#[test]
fn record_constructor_cannot_mutate_recovery_authority_before_append() {
    let fixture = recovered_fixture();
    let generation = fixture.manifest.generation_id().as_str().to_owned();
    let tombstone = fixture
        .family
        .path()
        .join(".bullet-family/coord/events.jsonl");
    let segment = generation_segment(&fixture);
    let segment_length = fs::metadata(&segment).unwrap().len();
    let pending = generation_pending(&fixture);
    let pending_before = snapshot(&pending);
    let claim_id = fixture.claim_ids[0].clone();
    let error = Ledger::new(fixture.family.path())
        .transact(&generation, request_id('9').as_str(), |_| {
            set_mode(&tombstone, 0);
            Ok(Record::Heartbeat {
                schema_version: GENERATION_SCHEMA_VERSION,
                at_unix_ms: 31,
                claim_id,
                agent: "frozen-agent".to_owned(),
                expires_unix_ms: 32,
                note: Some("must not append".to_owned()),
            })
        })
        .unwrap_err();
    assert_eq!(error.code(), "TOMBSTONE_SEAL_OUTCOME_UNKNOWN");
    assert_eq!(fs::metadata(segment).unwrap().len(), segment_length);
    assert_eq!(snapshot(&pending), pending_before);
}

#[test]
fn record_constructor_cannot_rewrite_manifest_before_append() {
    let fixture = recovered_fixture();
    let generation = fixture.manifest.generation_id().as_str().to_owned();
    let manifest = fixture
        .family
        .path()
        .join(".bullet-family/coord/generations")
        .join(&generation)
        .join("manifest.json");
    let segment = generation_segment(&fixture);
    let segment_length = fs::metadata(&segment).unwrap().len();
    let pending = generation_pending(&fixture);
    let pending_before = snapshot(&pending);
    let claim_id = fixture.claim_ids[0].clone();
    let error = Ledger::new(fixture.family.path())
        .transact(&generation, request_id('8').as_str(), |_| {
            rewrite(&manifest);
            Ok(Record::Heartbeat {
                schema_version: GENERATION_SCHEMA_VERSION,
                at_unix_ms: 31,
                claim_id,
                agent: "frozen-agent".to_owned(),
                expires_unix_ms: 32,
                note: Some("must not append".to_owned()),
            })
        })
        .unwrap_err();
    assert_eq!(error.code(), "INVALID_COORD_GENERATION");
    assert_eq!(fs::metadata(segment).unwrap().len(), segment_length);
    assert_eq!(snapshot(&pending), pending_before);
}
