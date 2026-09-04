use sha2::{Digest, Sha256};

use super::*;
use crate::coord::{ClaimState, ClaimSummary, RequestId};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

pub(crate) fn fixture_request() -> RecoveryReceiptAdoptionRequestV1 {
    let leaves = vec![leaf("src/a.rs", '1'), leaf("src/b.rs", '3')];
    RecoveryReceiptAdoptionRequestV1 {
        kind: RecoveryAdoptionRequestKindV1::RecoveryReceiptAdoptionRequestV1,
        schema_version: RECOVERY_ADOPTION_REQUEST_SCHEMA_VERSION,
        request_id: RequestId::parse(tag("req_", '9', 64)).unwrap(),
        expected_watermark: RecoveryAdoptionWatermarkV1 {
            generation_id: tag("gen_", 'a', 64),
            manifest_blake3: tag("blake3:", 'b', 64),
            last_sequence: 7,
            next_sequence: 8,
            head_envelope_blake3: tag("", 'c', 64),
            last_record_blake3: tag("", 'd', 64),
            last_request_id: RequestId::parse(tag("req_", '7', 64)).unwrap(),
            last_request_blake3: tag("", 'e', 64),
            byte_length: 4_096,
        },
        subject: RecoveryReceiptAdoptionSubjectV1 {
            repo: "bullet-kernel".to_owned(),
            git_expectation: RecoveryGitExpectationV1 {
                object_format: RecoveryGitObjectFormatV1::Sha1,
                commit_oid: tag("sha1:", '1', 40),
                raw_commit_bytes: b"tree 1111111111111111111111111111111111111111\n\nfixture\n"
                    .to_vec(),
                raw_commit_sha256: sha(
                    b"tree 1111111111111111111111111111111111111111\n\nfixture\n",
                ),
                parent_oid: tag("sha1:", '2', 40),
                parent_tree_oid: tag("sha1:", '3', 40),
                parent_receipt_observation: forensic(
                    RecoveryForensicArtifactKindV1::TrustedPrefix,
                    RecoveryForensicRecordKindV1::CommitReceipt,
                    10,
                    '7',
                ),
                result_tree_oid: tag("sha1:", '4', 40),
                raw_tree_sha256: tag("sha256:", '5', 64),
                leaf_transitions: leaves,
            },
            claims: vec![
                claim('1', 'd', "src/a.rs", 11, 31),
                claim('2', 'e', "src/b.rs", 12, 32),
            ],
            group_receipt_observation: forensic(
                RecoveryForensicArtifactKindV1::FrozenLiveSource,
                RecoveryForensicRecordKindV1::CommitReceiptGroup,
                40,
                '4',
            ),
            proof_observations: vec![RecoveryProofObservationV1 {
                record: generation_record(RecoveryGenerationRecordKindV1::ProofReceipt, 5, '5'),
                expected_subject_blake3: tag("blake3:", '6', 64),
                expected_role: RecoveryProofRoleV1::RecoveryProof,
            }],
            review_observation: RecoveryReviewObservationV1 {
                record: generation_record(RecoveryGenerationRecordKindV1::ReviewReceipt, 6, '6'),
                expected_subject_blake3: tag("blake3:", '7', 64),
                expected_role: RecoveryReviewRoleV1::IndependentReview,
            },
        },
    }
}

fn claim(
    claim_marker: char,
    digest_marker: char,
    path: &str,
    claim_index: u64,
    handoff_index: u64,
) -> RecoveryAdoptionClaimV1 {
    RecoveryAdoptionClaimV1 {
        claim_id: tag("clm_", claim_marker, 64),
        frozen_claim_blake3: tag("blake3:", digest_marker, 64),
        trusted_claim_record: forensic(
            RecoveryForensicArtifactKindV1::TrustedPrefix,
            RecoveryForensicRecordKindV1::Claim,
            claim_index,
            claim_marker,
        ),
        committed_paths: vec![path.to_owned()],
        handoff_observation: forensic(
            RecoveryForensicArtifactKindV1::FrozenLiveSource,
            RecoveryForensicRecordKindV1::Handoff,
            handoff_index,
            digest_marker,
        ),
    }
}

fn leaf(path: &str, marker: char) -> RecoveryGitLeafTransitionV1 {
    let old = format!("old-{marker}").into_bytes();
    let new = format!("new-{marker}").into_bytes();
    RecoveryGitLeafTransitionV1 {
        status: RecoveryGitLeafStatusV1::Modified,
        path: path.to_owned(),
        old_mode: "100644".to_owned(),
        new_mode: "100644".to_owned(),
        old_blob_oid: tag("sha1:", marker, 40),
        new_blob_oid: tag("sha1:", next_hex(marker), 40),
        old_sha256: sha(&old),
        new_sha256: sha(&new),
        old_bytes: old,
        new_bytes: new,
    }
}

fn forensic(
    artifact_kind: RecoveryForensicArtifactKindV1,
    expected_record_kind: RecoveryForensicRecordKindV1,
    record_index: u64,
    marker: char,
) -> ForensicRecordRefV1 {
    ForensicRecordRefV1 {
        artifact_kind,
        artifact_sha256: tag(
            "sha256:",
            if artifact_kind == RecoveryForensicArtifactKindV1::TrustedPrefix {
                '8'
            } else {
                '9'
            },
            64,
        ),
        record_index,
        byte_start: record_index * 100,
        byte_end: record_index * 100 + 50,
        record_sha256: tag("sha256:", marker, 64),
        expected_record_kind,
    }
}

fn generation_record(
    expected_record_kind: RecoveryGenerationRecordKindV1,
    sequence: u64,
    marker: char,
) -> RecoveryGenerationRecordRefV1 {
    RecoveryGenerationRecordRefV1 {
        generation_id: tag("gen_", 'a', 64),
        sequence,
        request_id: RequestId::parse(tag("req_", marker, 64)).unwrap(),
        request_blake3: tag("", marker, 64),
        record_blake3: tag("", next_hex(marker), 64),
        envelope_blake3: tag("", 'f', 64),
        byte_offset: sequence * 100,
        frame_length: 50,
        expected_record_kind,
    }
}

fn verified(request: RecoveryReceiptAdoptionRequestV1) -> RecoveryReceiptAdoptionRecordV1 {
    RecoveryReceiptAdoptionRecordV1::verified(
        request,
        "recovery-operator".to_owned(),
        tag("sha256:", 'a', 64),
        tag("sha256:", 'b', 64),
        1,
        tag("sha256:", 'c', 64),
        "recovery-orchestrator".to_owned(),
        "independent-reviewer".to_owned(),
    )
    .unwrap()
}

fn tag(prefix: &str, marker: char, width: usize) -> String {
    format!("{prefix}{}", marker.to_string().repeat(width))
}

fn sha(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

fn next_hex(marker: char) -> char {
    char::from_digit(marker.to_digit(16).unwrap() + 1, 16).unwrap()
}

#[test]
fn identities_are_deterministic_and_exclude_only_request_identity() {
    let request = fixture_request();
    request.validate().unwrap();
    let adoption_id = request.adoption_id().unwrap();
    let request_subject = request.request_subject_blake3().unwrap();
    assert_eq!(request.adoption_id().unwrap(), adoption_id);
    assert_eq!(request.request_subject_blake3().unwrap(), request_subject);

    let mut replay = request.clone();
    replay.request_id = RequestId::parse(tag("req_", '8', 64)).unwrap();
    assert_eq!(replay.adoption_id().unwrap(), adoption_id);
    assert_ne!(replay.request_subject_blake3().unwrap(), request_subject);

    let mut changed = request;
    changed.subject.claims[0].committed_paths[0] = "src/aa.rs".to_owned();
    changed.subject.git_expectation.leaf_transitions[0].path = "src/aa.rs".to_owned();
    assert_ne!(changed.adoption_id().unwrap(), adoption_id);
}

#[test]
fn recursive_schema_and_duplicate_keys_are_refused() {
    let request = fixture_request();
    let mut unknown = serde_json::to_value(&request).unwrap();
    unknown["subject"]["git_expectation"]["leaf_transitions"][0]["surprise"] =
        serde_json::json!(true);
    let unknown = bullet_wire::canonical_json(&unknown).unwrap();
    assert!(bullet_wire::decode_canonical::<RecoveryReceiptAdoptionRequestV1>(&unknown).is_err());

    let canonical = String::from_utf8(bullet_wire::canonical_json(&request).unwrap()).unwrap();
    let duplicate = canonical.replacen(
        "\"schema_version\":1",
        "\"schema_version\":1,\"schema_version\":1",
        1,
    );
    assert_ne!(duplicate, canonical);
    assert!(
        bullet_wire::decode_canonical::<RecoveryReceiptAdoptionRequestV1>(duplicate.as_bytes())
            .is_err()
    );

    let mut obsolete_tree_payload = serde_json::to_value(request).unwrap();
    obsolete_tree_payload["subject"]["git_expectation"]["raw_tree_bytes"] =
        serde_json::json!([1, 2, 3]);
    assert!(
        serde_json::from_value::<RecoveryReceiptAdoptionRequestV1>(obsolete_tree_payload).is_err()
    );
}

#[test]
fn bounds_ordering_partitions_and_observations_fail_closed() {
    let mut unsafe_watermark = fixture_request();
    unsafe_watermark.expected_watermark.byte_length = MAX_SAFE_INTEGER + 1;
    assert!(unsafe_watermark.validate().is_err());

    let mut tagged_ledger_digest = fixture_request();
    tagged_ledger_digest.expected_watermark.head_envelope_blake3 = tag("blake3:", 'c', 64);
    assert!(tagged_ledger_digest.validate().is_err());

    let mut uppercase_ledger_digest = fixture_request();
    uppercase_ledger_digest.subject.proof_observations[0]
        .record
        .request_blake3 = tag("", 'A', 64);
    assert!(uppercase_ledger_digest.validate().is_err());

    let mut zero_index = fixture_request();
    zero_index.subject.claims[0]
        .trusted_claim_record
        .record_index = 0;
    assert!(zero_index.validate().is_err());

    let mut invalid_proof = fixture_request();
    invalid_proof.subject.proof_observations[0]
        .record
        .expected_record_kind = RecoveryGenerationRecordKindV1::ReviewReceipt;
    assert!(invalid_proof.validate().is_err());

    let mut stale_evidence = fixture_request();
    stale_evidence.subject.proof_observations[0].record.sequence = 8;
    assert!(stale_evidence.validate().is_err());

    let mut wrong_parent_receipt = fixture_request();
    wrong_parent_receipt
        .subject
        .git_expectation
        .parent_receipt_observation
        .expected_record_kind = RecoveryForensicRecordKindV1::Claim;
    assert!(wrong_parent_receipt.validate().is_err());

    let mut unsorted = fixture_request();
    unsorted.subject.claims.swap(0, 1);
    assert!(unsorted.validate().is_err());

    let mut incomplete_partition = fixture_request();
    incomplete_partition.subject.claims.pop();
    assert!(incomplete_partition.validate().is_err());

    let mut singleton_group = fixture_request();
    singleton_group.subject.claims.truncate(1);
    singleton_group
        .subject
        .git_expectation
        .leaf_transitions
        .truncate(1);
    assert!(singleton_group.validate().is_err());

    let mut review_before_proof = fixture_request();
    review_before_proof
        .subject
        .review_observation
        .record
        .sequence = 4;
    assert!(review_before_proof.validate().is_err());

    let mut duplicate_request = fixture_request();
    let mut second_proof = duplicate_request.subject.proof_observations[0].clone();
    second_proof.record.sequence = 6;
    second_proof.record.byte_offset = 600;
    duplicate_request
        .subject
        .proof_observations
        .push(second_proof);
    duplicate_request.subject.review_observation.record.sequence = 7;
    assert!(duplicate_request.validate().is_err());

    let mut duplicate_review_request = fixture_request();
    duplicate_review_request
        .subject
        .review_observation
        .record
        .request_id = duplicate_review_request.subject.proof_observations[0]
        .record
        .request_id
        .clone();
    assert!(duplicate_review_request.validate().is_err());

    let mut too_many_proofs = fixture_request();
    let template = too_many_proofs.subject.proof_observations[0].clone();
    too_many_proofs.subject.proof_observations = (0_u64..=64)
        .map(|index| {
            let mut proof = template.clone();
            proof.record.sequence = index + 2;
            proof.record.request_id = RequestId::parse(format!("req_{:064x}", index + 1)).unwrap();
            proof.record.byte_offset = (index + 2) * 100;
            proof
        })
        .collect();
    too_many_proofs.subject.review_observation.record.sequence = 67;
    too_many_proofs
        .subject
        .review_observation
        .record
        .byte_offset = 6_700;
    too_many_proofs.expected_watermark.last_sequence = 67;
    too_many_proofs.expected_watermark.next_sequence = 68;
    too_many_proofs.expected_watermark.byte_length = 10_000;
    assert!(too_many_proofs.validate().is_err());

    let mut changed_bytes = fixture_request();
    changed_bytes
        .subject
        .git_expectation
        .raw_commit_bytes
        .push(b'x');
    assert!(changed_bytes.validate().is_err());
}

#[test]
fn only_consistent_independent_verified_records_validate() {
    let request = fixture_request();
    let value = verified(request.clone());
    value.validate().unwrap();
    assert_eq!(value.adoption_id, request.adoption_id().unwrap());

    assert!(
        RecoveryReceiptAdoptionRecordV1::verified(
            request,
            "recovery-operator".to_owned(),
            tag("sha256:", 'a', 64),
            tag("sha256:", 'b', 64),
            1,
            tag("sha256:", 'c', 64),
            "same-actor".to_owned(),
            "same-actor".to_owned(),
        )
        .is_err()
    );

    let mut tampered = value;
    tampered.request_subject_blake3 = tag("blake3:", '0', 64);
    assert!(tampered.validate().is_err());
}

#[test]
fn absent_adoption_provenance_preserves_the_legacy_projection_shape() {
    let summary = ClaimSummary {
        claim_id: tag("clm_", '1', 64),
        agent: "agent".to_owned(),
        lane: "lane".to_owned(),
        repo: "bullet-kernel".to_owned(),
        paths: vec!["src/a.rs".to_owned()],
        claimed_at_unix_ms: 1,
        last_event_unix_ms: 1,
        expires_unix_ms: 60_001,
        state: ClaimState::FrozenRecovery,
        proof_command: None,
        changed_paths: Vec::new(),
        commit_oid: None,
        commit_orchestrator: None,
        commit_recorded_at_unix_ms: None,
        recovery_adoption: None,
    };
    let canonical = String::from_utf8(bullet_wire::canonical_json(&summary).unwrap()).unwrap();
    assert!(!canonical.contains("recovery_adoption"));
}
