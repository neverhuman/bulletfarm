use std::{
    fs,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use sha2::{Digest, Sha256};

use crate::coord::{
    ClaimState, CoordStore, RequestId,
    generation::{
        manifest::{CurrentPointer, Sha256Digest},
        recovery::{self, adoption_fixture},
    },
    model::{
        ForensicRecordRefV1, GENERATION_SCHEMA_VERSION, Record, RecoveryAdoptionClaimV1,
        RecoveryAdoptionRequestKindV1, RecoveryAdoptionWatermarkV1, RecoveryForensicArtifactKindV1,
        RecoveryForensicRecordKindV1, RecoveryGenerationRecordKindV1,
        RecoveryGenerationRecordRefV1, RecoveryGitExpectationV1, RecoveryProofObservationV1,
        RecoveryProofReceiptRecordV1, RecoveryProofRoleV1, RecoveryReceiptAdoptionRequestV1,
        RecoveryReceiptAdoptionSubjectV1, RecoveryReviewObservationV1,
        RecoveryReviewReceiptRecordV1, RecoveryReviewRoleV1,
    },
    recovery_adoption_verify,
    store::ledger::{AppendOutcome, Ledger, LedgerWatermark},
};

use super::super::git_fixture_support::{clone_repo, git_fixture};

#[path = "tests/published_recovery.rs"]
mod published_recovery;

struct Prepared {
    fixture: adoption_fixture::AdoptionRecoveryFixture,
    request: RecoveryReceiptAdoptionRequestV1,
    segment: std::path::PathBuf,
}

fn prepared() -> Prepared {
    let (source, git_expectation) = git_fixture();
    let parent = git_expectation.parent_oid.strip_prefix("sha1:").unwrap();
    let commit = git_expectation.commit_oid.strip_prefix("sha1:").unwrap();
    let fixture = adoption_fixture::fixture(parent, commit);
    clone_repo(source.path(), &fixture.family.path().join("bullet-kernel"));
    recovery::recover_with_writer_probe(&fixture.input, &fixture.manifest, |_| Ok(false)).unwrap();

    let pointer = CurrentPointer::for_manifest(&fixture.manifest).unwrap();
    let mut request = skeleton_request(&fixture, git_expectation, &pointer);
    let evidence_subject = recovery_adoption_verify::evidence_subject(&request).unwrap();
    let ledger = Ledger::new(fixture.family.path());
    let generation = fixture.manifest.generation_id().as_str().to_owned();

    let proof_body = RecoveryProofReceiptRecordV1::verified_pass(
        evidence_subject.clone(),
        "fresh-recovery-orchestrator".to_owned(),
        sha(b"proof command"),
        sha(b"proof output"),
        5,
    )
    .unwrap();
    let proof_id = proof_body.proof_receipt_id().to_owned();
    let proof_request = request_id('d');
    let proof = ledger
        .append(
            &generation,
            proof_request.as_str(),
            &Record::RecoveryProofReceiptV1 {
                schema_version: GENERATION_SCHEMA_VERSION,
                at_unix_ms: 21,
                body: proof_body,
            },
        )
        .unwrap();
    request.subject.proof_observations = vec![RecoveryProofObservationV1 {
        record: generation_ref(&proof, RecoveryGenerationRecordKindV1::ProofReceipt),
        expected_subject_blake3: evidence_subject.clone(),
        expected_role: RecoveryProofRoleV1::RecoveryProof,
    }];

    let review_body = RecoveryReviewReceiptRecordV1::verified_approval(
        evidence_subject.clone(),
        vec![proof_id],
        "fresh-recovery-orchestrator".to_owned(),
        "independent-recovery-reviewer".to_owned(),
        sha(b"review evidence"),
    )
    .unwrap();
    let review_request = request_id('e');
    let review = ledger
        .append(
            &generation,
            review_request.as_str(),
            &Record::RecoveryReviewReceiptV1 {
                schema_version: GENERATION_SCHEMA_VERSION,
                at_unix_ms: 22,
                body: review_body,
            },
        )
        .unwrap();
    request.subject.review_observation = RecoveryReviewObservationV1 {
        record: generation_ref(&review, RecoveryGenerationRecordKindV1::ReviewReceipt),
        expected_subject_blake3: evidence_subject,
        expected_role: RecoveryReviewRoleV1::IndependentReview,
    };
    request.expected_watermark = watermark(&review.watermark);
    request.validate().unwrap();
    let segment = fixture
        .family
        .path()
        .join(".bullet-family/coord/generations")
        .join(&generation)
        .join("events.jsonl");
    Prepared {
        fixture,
        request,
        segment,
    }
}

fn skeleton_request(
    fixture: &adoption_fixture::AdoptionRecoveryFixture,
    git_expectation: RecoveryGitExpectationV1,
    pointer: &CurrentPointer,
) -> RecoveryReceiptAdoptionRequestV1 {
    let trusted_sha = Sha256Digest::for_bytes(&fixture.trusted)
        .as_str()
        .to_owned();
    let frozen_sha = Sha256Digest::for_bytes(&fixture.frozen).as_str().to_owned();
    let generation = fixture.manifest.generation_id().as_str().to_owned();
    RecoveryReceiptAdoptionRequestV1 {
        kind: RecoveryAdoptionRequestKindV1::RecoveryReceiptAdoptionRequestV1,
        schema_version: 1,
        request_id: request_id('f'),
        expected_watermark: RecoveryAdoptionWatermarkV1 {
            generation_id: generation.clone(),
            manifest_blake3: pointer.manifest_blake3().to_owned(),
            last_sequence: 3,
            next_sequence: 4,
            head_envelope_blake3: ledger_digest('1'),
            last_record_blake3: ledger_digest('2'),
            last_request_id: request_id('e'),
            last_request_blake3: ledger_digest('3'),
            byte_length: 10_000,
        },
        subject: RecoveryReceiptAdoptionSubjectV1 {
            repo: "bullet-kernel".to_owned(),
            git_expectation: RecoveryGitExpectationV1 {
                parent_receipt_observation: forensic(
                    RecoveryForensicArtifactKindV1::TrustedPrefix,
                    RecoveryForensicRecordKindV1::CommitReceipt,
                    &trusted_sha,
                    &fixture.parent_receipt,
                ),
                ..git_expectation
            },
            claims: vec![
                adoption_claim(fixture, 0, "a.txt", &trusted_sha, &frozen_sha),
                adoption_claim(fixture, 1, "b.txt", &trusted_sha, &frozen_sha),
            ],
            group_receipt_observation: forensic(
                RecoveryForensicArtifactKindV1::FrozenLiveSource,
                RecoveryForensicRecordKindV1::CommitReceiptGroup,
                &frozen_sha,
                &fixture.group_receipt,
            ),
            proof_observations: vec![RecoveryProofObservationV1 {
                record: placeholder_generation_ref(
                    &generation,
                    2,
                    RecoveryGenerationRecordKindV1::ProofReceipt,
                    'd',
                ),
                expected_subject_blake3: blake('4'),
                expected_role: RecoveryProofRoleV1::RecoveryProof,
            }],
            review_observation: RecoveryReviewObservationV1 {
                record: placeholder_generation_ref(
                    &generation,
                    3,
                    RecoveryGenerationRecordKindV1::ReviewReceipt,
                    'e',
                ),
                expected_subject_blake3: blake('5'),
                expected_role: RecoveryReviewRoleV1::IndependentReview,
            },
        },
    }
}

fn adoption_claim(
    fixture: &adoption_fixture::AdoptionRecoveryFixture,
    index: usize,
    path: &str,
    trusted_sha: &str,
    frozen_sha: &str,
) -> RecoveryAdoptionClaimV1 {
    RecoveryAdoptionClaimV1 {
        claim_id: fixture.claim_ids[index].clone(),
        frozen_claim_blake3: fixture.frozen_claims[index].claim_blake3.clone(),
        trusted_claim_record: forensic(
            RecoveryForensicArtifactKindV1::TrustedPrefix,
            RecoveryForensicRecordKindV1::Claim,
            trusted_sha,
            &fixture.claim_records[index],
        ),
        committed_paths: vec![path.to_owned()],
        handoff_observation: forensic(
            RecoveryForensicArtifactKindV1::FrozenLiveSource,
            RecoveryForensicRecordKindV1::Handoff,
            frozen_sha,
            &fixture.handoff_records[index],
        ),
    }
}

fn forensic(
    artifact_kind: RecoveryForensicArtifactKindV1,
    expected_record_kind: RecoveryForensicRecordKindV1,
    artifact_sha256: &str,
    line: &adoption_fixture::LineRef,
) -> ForensicRecordRefV1 {
    ForensicRecordRefV1 {
        artifact_kind,
        artifact_sha256: artifact_sha256.to_owned(),
        record_index: line.index,
        byte_start: line.start,
        byte_end: line.end,
        record_sha256: line.sha256.clone(),
        expected_record_kind,
    }
}

fn generation_ref(
    outcome: &AppendOutcome,
    expected_record_kind: RecoveryGenerationRecordKindV1,
) -> RecoveryGenerationRecordRefV1 {
    RecoveryGenerationRecordRefV1 {
        generation_id: outcome.receipt.generation_id.clone(),
        sequence: outcome.receipt.sequence,
        request_id: RequestId::parse(outcome.receipt.request_id.clone()).unwrap(),
        request_blake3: outcome.receipt.request_digest.clone(),
        record_blake3: outcome.receipt.record_digest.clone(),
        envelope_blake3: outcome.receipt.envelope_digest.clone(),
        byte_offset: outcome.receipt.byte_offset,
        frame_length: outcome.receipt.frame_length,
        expected_record_kind,
    }
}

fn placeholder_generation_ref(
    generation_id: &str,
    sequence: u64,
    expected_record_kind: RecoveryGenerationRecordKindV1,
    marker: char,
) -> RecoveryGenerationRecordRefV1 {
    RecoveryGenerationRecordRefV1 {
        generation_id: generation_id.to_owned(),
        sequence,
        request_id: request_id(marker),
        request_blake3: ledger_digest(marker),
        record_blake3: ledger_digest(next(marker)),
        envelope_blake3: ledger_digest('9'),
        byte_offset: sequence * 100,
        frame_length: 50,
        expected_record_kind,
    }
}

fn watermark(value: &LedgerWatermark) -> RecoveryAdoptionWatermarkV1 {
    RecoveryAdoptionWatermarkV1 {
        generation_id: value.generation_id.clone(),
        manifest_blake3: value.manifest_blake3.clone(),
        last_sequence: value.last_sequence,
        next_sequence: value.next_sequence,
        head_envelope_blake3: value.head_envelope_digest.clone(),
        last_record_blake3: value.last_record_digest.clone(),
        last_request_id: RequestId::parse(value.last_request_id.clone()).unwrap(),
        last_request_blake3: value.last_request_digest.clone(),
        byte_length: value.byte_length,
    }
}

fn request_id(marker: char) -> RequestId {
    RequestId::parse(format!("req_{}", marker.to_string().repeat(64))).unwrap()
}

fn blake(marker: char) -> String {
    format!("blake3:{}", marker.to_string().repeat(64))
}

fn ledger_digest(marker: char) -> String {
    marker.to_string().repeat(64)
}

fn sha(value: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(value))
}

fn next(marker: char) -> char {
    char::from_digit(marker.to_digit(16).unwrap() + 1, 16).unwrap()
}

#[test]
fn adoption_is_atomic_and_exact_replay_runs_no_effect_or_clock() {
    let prepared = prepared();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let store = CoordStore::with_clock(prepared.fixture.family.path().to_owned(), move || {
        observed.fetch_add(1, Ordering::SeqCst);
        Ok(30)
    });
    let applied = store.adopt_recovery_receipts(&prepared.request).unwrap();
    assert!(!applied.replayed);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(applied.projection.iter().all(|claim| {
        claim.state == ClaimState::RecoveredReceipted && claim.recovery_adoption.is_some()
    }));
    let committed_length = fs::metadata(&prepared.segment).unwrap().len();

    fs::rename(
        prepared.fixture.family.path().join("bullet-kernel/.git"),
        prepared
            .fixture
            .family
            .path()
            .join("bullet-kernel/.git-after-adoption"),
    )
    .unwrap();
    let replay_store = CoordStore::with_clock(prepared.fixture.family.path().to_owned(), || {
        panic!("clock invoked during exact adoption replay")
    });
    let replayed = replay_store
        .adopt_recovery_receipts(&prepared.request)
        .unwrap();
    assert!(replayed.replayed);
    assert_eq!(replayed.receipt, applied.receipt);
    assert_eq!(replayed.watermark, applied.watermark);
    assert_eq!(replayed.projection, applied.projection);
    assert_eq!(
        fs::metadata(&prepared.segment).unwrap().len(),
        committed_length
    );

    let mut changed = prepared.request.clone();
    changed.subject.claims[0].committed_paths[0] = "changed.txt".to_owned();
    assert_eq!(
        replay_store
            .adopt_recovery_receipts(&changed)
            .unwrap_err()
            .code(),
        "COORD_REQUEST_CONFLICT"
    );
    let mut stale = prepared.request;
    stale.request_id = request_id('a');
    assert_eq!(
        replay_store
            .adopt_recovery_receipts(&stale)
            .unwrap_err()
            .code(),
        "STALE_COORD_WATERMARK"
    );
    assert_eq!(
        fs::metadata(prepared.segment).unwrap().len(),
        committed_length
    );
}

#[test]
fn forensic_or_generation_evidence_tamper_never_appends_or_calls_clock() {
    for generation_tamper in [false, true] {
        let prepared = prepared();
        let before = fs::metadata(&prepared.segment).unwrap().len();
        let mut request = prepared.request;
        request.request_id = request_id(if generation_tamper { '1' } else { '2' });
        if generation_tamper {
            request.subject.proof_observations[0].record.record_blake3 = ledger_digest('0');
        } else {
            request.subject.claims[0].trusted_claim_record.record_sha256 = sha(b"wrong record");
        }
        let store = CoordStore::with_clock(prepared.fixture.family.path().to_owned(), || {
            panic!("clock invoked after evidence mismatch")
        });
        assert_eq!(
            store.adopt_recovery_receipts(&request).unwrap_err().code(),
            "RECOVERY_EVIDENCE_MISMATCH"
        );
        assert_eq!(fs::metadata(prepared.segment).unwrap().len(), before);
    }
}
