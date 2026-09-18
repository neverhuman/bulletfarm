use std::collections::BTreeMap;

use super::*;
use crate::coord::RequestId;
use crate::coord::model::{
    FrozenClaimSubject, GENERATION_SCHEMA_VERSION, LEGACY_SCHEMA_VERSION, Record,
    RecoveryProofReceiptRecordV1, RecoveryReceiptAdoptionRequestV1, RecoveryReviewReceiptRecordV1,
    recovery_adoption_request_fixture,
};

const INCIDENT: u64 = 1_000;
const RECOVERED: u64 = 2_000;
const ADOPTED: u64 = 3_000;

fn record(request: RecoveryReceiptAdoptionRequestV1) -> RecoveryReceiptAdoptionRecordV1 {
    RecoveryReceiptAdoptionRecordV1::verified(
        request,
        "recovery-operator".to_owned(),
        format!("sha256:{}", "a".repeat(64)),
        format!("sha256:{}", "b".repeat(64)),
        1,
        format!("sha256:{}", "c".repeat(64)),
        "recovery-orchestrator".to_owned(),
        "independent-reviewer".to_owned(),
    )
    .unwrap()
}

fn frozen_claims(request: &RecoveryReceiptAdoptionRequestV1) -> BTreeMap<String, ClaimSummary> {
    request
        .subject
        .claims
        .iter()
        .map(|requested| {
            (
                requested.claim_id.clone(),
                ClaimSummary {
                    claim_id: requested.claim_id.clone(),
                    agent: format!("agent-{}", &requested.claim_id[4..8]),
                    lane: "recovery-fixture".to_owned(),
                    repo: request.subject.repo.clone(),
                    paths: requested.committed_paths.clone(),
                    claimed_at_unix_ms: 100,
                    last_event_unix_ms: 100,
                    expires_unix_ms: 60_100,
                    state: ClaimState::FrozenRecovery,
                    proof_command: None,
                    changed_paths: Vec::new(),
                    commit_oid: None,
                    commit_orchestrator: None,
                    commit_recorded_at_unix_ms: None,
                    recovery_adoption: None,
                },
            )
        })
        .collect()
}

fn authority(request: &RecoveryReceiptAdoptionRequestV1) -> RecoveryAdoptionAuthority {
    let body = RecoveryBaselineBody {
        manifest_blake3: request.expected_watermark.manifest_blake3.clone(),
        incident_at_unix_ms: INCIDENT,
        recovered_at_unix_ms: RECOVERED,
        trusted_state_blake3: format!("blake3:{}", "f".repeat(64)),
        frozen_claims: request
            .subject
            .claims
            .iter()
            .map(|claim| FrozenClaimSubject {
                claim_id: claim.claim_id.clone(),
                claim_blake3: claim.frozen_claim_blake3.clone(),
            })
            .collect(),
    };
    RecoveryAdoptionAuthority::from_baseline(&request.expected_watermark.generation_id, &body)
}

#[test]
fn group_transition_is_atomic_and_keeps_ordinary_receipt_fields_empty() {
    let request = recovery_adoption_request_fixture();
    let body = record(request.clone());
    let mut claims = frozen_claims(&request);
    apply(ADOPTED, &body, Some(&authority(&request)), &mut claims).unwrap();

    let summaries = claims
        .values()
        .map(|claim| claim.recovery_adoption.clone().unwrap())
        .collect::<Vec<_>>();
    assert!(
        claims
            .values()
            .all(|claim| claim.state == ClaimState::RecoveredReceipted)
    );
    assert!(summaries.windows(2).all(|pair| pair[0] == pair[1]));
    assert!(claims.values().all(|claim| {
        claim.proof_command.is_none()
            && claim.changed_paths.is_empty()
            && claim.commit_oid.is_none()
            && claim.commit_orchestrator.is_none()
            && claim.commit_recorded_at_unix_ms.is_none()
    }));
    assert_eq!(summaries[0].adopted_at_unix_ms, ADOPTED);
    assert_eq!(summaries[0].request_id, request.request_id.as_str());
}

#[test]
fn any_member_failure_leaves_the_entire_group_unchanged() {
    let request = recovery_adoption_request_fixture();
    let body = record(request.clone());
    let mut claims = frozen_claims(&request);
    claims
        .get_mut(&request.subject.claims[1].claim_id)
        .unwrap()
        .repo = "bullet-git".to_owned();
    let before = claims.clone();
    assert_eq!(
        apply(ADOPTED, &body, Some(&authority(&request)), &mut claims)
            .unwrap_err()
            .code(),
        "RECOVERY_EVIDENCE_MISMATCH"
    );
    assert_eq!(claims, before);

    let mut claims = frozen_claims(&request);
    claims
        .get_mut(&request.subject.claims[1].claim_id)
        .unwrap()
        .state = ClaimState::HandedOff;
    let before = claims.clone();
    assert_eq!(
        apply(ADOPTED, &body, Some(&authority(&request)), &mut claims)
            .unwrap_err()
            .code(),
        "RECOVERY_CLAIM_NOT_FROZEN"
    );
    assert_eq!(claims, before);
}

#[test]
fn authority_digest_coverage_and_ordinary_provenance_are_exact() {
    let request = recovery_adoption_request_fixture();
    let body = record(request.clone());
    let original = frozen_claims(&request);

    let mut claims = original.clone();
    assert_eq!(
        apply(ADOPTED, &body, None, &mut claims).unwrap_err().code(),
        "RECOVERY_AUTHORITY_INSUFFICIENT"
    );
    assert_eq!(claims, original);

    let mut stale = request.clone();
    stale.expected_watermark.generation_id = format!("gen_{}", "0".repeat(64));
    for proof in &mut stale.subject.proof_observations {
        proof.record.generation_id = stale.expected_watermark.generation_id.clone();
    }
    stale.subject.review_observation.record.generation_id =
        stale.expected_watermark.generation_id.clone();
    let mut claims = original.clone();
    assert_eq!(
        apply(
            ADOPTED,
            &record(stale),
            Some(&authority(&request)),
            &mut claims,
        )
        .unwrap_err()
        .code(),
        "STALE_COORD_GENERATION"
    );
    assert_eq!(claims, original);

    let mut wrong_digest = request.clone();
    wrong_digest.subject.claims[0].frozen_claim_blake3 = format!("blake3:{}", "0".repeat(64));
    let mut claims = original.clone();
    assert_eq!(
        apply(
            ADOPTED,
            &record(wrong_digest),
            Some(&authority(&request)),
            &mut claims,
        )
        .unwrap_err()
        .code(),
        "RECOVERY_EVIDENCE_MISMATCH"
    );
    assert_eq!(claims, original);

    let mut claims = original.clone();
    claims
        .get_mut(&request.subject.claims[0].claim_id)
        .unwrap()
        .paths = vec!["outside".to_owned()];
    let before = claims.clone();
    assert_eq!(
        apply(ADOPTED, &body, Some(&authority(&request)), &mut claims)
            .unwrap_err()
            .code(),
        "RECOVERY_EVIDENCE_MISMATCH"
    );
    assert_eq!(claims, before);

    let mut claims = original;
    claims
        .get_mut(&request.subject.claims[0].claim_id)
        .unwrap()
        .proof_command = Some("quarantined-proof".to_owned());
    let before = claims.clone();
    assert_eq!(
        apply(ADOPTED, &body, Some(&authority(&request)), &mut claims)
            .unwrap_err()
            .code(),
        "RECOVERY_EVIDENCE_MISMATCH"
    );
    assert_eq!(claims, before);
}

#[test]
fn a_second_logical_adoption_conflicts_without_changing_state() {
    let request = recovery_adoption_request_fixture();
    let body = record(request.clone());
    let authority = authority(&request);
    let mut claims = frozen_claims(&request);
    apply(ADOPTED, &body, Some(&authority), &mut claims).unwrap();
    let adopted = claims.clone();
    assert_eq!(
        apply(ADOPTED + 1, &body, Some(&authority), &mut claims)
            .unwrap_err()
            .code(),
        "RECOVERY_ADOPTION_CONFLICT"
    );
    assert_eq!(claims, adopted);
}

#[test]
fn a_disjoint_group_cannot_readopt_the_same_repository_commit() {
    let request = recovery_adoption_request_fixture();
    let mut other = request.clone();
    other.request_id = RequestId::parse(format!("req_{}", "8".repeat(64))).unwrap();
    other.subject.claims[0].claim_id = format!("clm_{}", "3".repeat(64));
    other.subject.claims[1].claim_id = format!("clm_{}", "4".repeat(64));
    other.subject.claims[0].frozen_claim_blake3 = format!("blake3:{}", "5".repeat(64));
    other.subject.claims[1].frozen_claim_blake3 = format!("blake3:{}", "6".repeat(64));
    other.validate().unwrap();

    let mut claims = frozen_claims(&request);
    claims.extend(frozen_claims(&other));
    let mut authority = authority(&request);
    authority.frozen_claims.extend(
        other
            .subject
            .claims
            .iter()
            .map(|claim| (claim.claim_id.clone(), claim.frozen_claim_blake3.clone())),
    );
    apply(ADOPTED, &record(request), Some(&authority), &mut claims).unwrap();
    let adopted = claims.clone();
    assert_eq!(
        apply(ADOPTED + 1, &record(other), Some(&authority), &mut claims)
            .unwrap_err()
            .code(),
        "RECOVERY_ADOPTION_CONFLICT"
    );
    assert_eq!(claims, adopted);
}

#[test]
fn schema_two_replay_reaches_recovered_receipted_from_the_exact_baseline() {
    let mut request = recovery_adoption_request_fixture();
    let mut records = request
        .subject
        .claims
        .iter()
        .enumerate()
        .map(|(index, claim)| Record::Claim {
            schema_version: LEGACY_SCHEMA_VERSION,
            at_unix_ms: 100,
            claim_id: claim.claim_id.clone(),
            agent: format!("legacy-agent-{index}"),
            lane: "legacy-lane".to_owned(),
            repo: request.subject.repo.clone(),
            paths: claim.committed_paths.clone(),
            expires_unix_ms: 60_100,
        })
        .collect::<Vec<_>>();
    let trusted = crate::coord::state::summaries(&records, INCIDENT).unwrap();
    let trusted_state_blake3 = format!(
        "blake3:{}",
        bullet_wire::hash_canonical("bullet-family.coord.trusted-state.v2", &trusted)
            .unwrap()
            .to_hex()
    );
    let frozen = trusted
        .values()
        .map(|claim| FrozenClaimSubject {
            claim_id: claim.claim_id.clone(),
            claim_blake3: format!(
                "blake3:{}",
                bullet_wire::hash_canonical("bullet-family.coord.frozen-claim.v2", claim)
                    .unwrap()
                    .to_hex()
            ),
        })
        .collect::<Vec<_>>();
    for requested in &mut request.subject.claims {
        requested.frozen_claim_blake3 = frozen
            .iter()
            .find(|claim| claim.claim_id == requested.claim_id)
            .unwrap()
            .claim_blake3
            .clone();
    }
    let evidence_subject = crate::coord::recovery_adoption_verify::evidence_subject(&request)
        .expect("mutated recovery fixture has a canonical evidence subject");
    for proof in &mut request.subject.proof_observations {
        proof.expected_subject_blake3 = evidence_subject.clone();
    }
    request.subject.review_observation.expected_subject_blake3 = evidence_subject;
    let baseline = RecoveryBaselineBody {
        manifest_blake3: request.expected_watermark.manifest_blake3.clone(),
        incident_at_unix_ms: INCIDENT,
        recovered_at_unix_ms: RECOVERED,
        trusted_state_blake3,
        frozen_claims: frozen,
    };
    records.push(Record::RecoveryBaselineV2 {
        schema_version: GENERATION_SCHEMA_VERSION,
        generation_id: request.expected_watermark.generation_id.clone(),
        body: baseline,
    });
    let proof = RecoveryProofReceiptRecordV1::verified_pass(
        request.subject.proof_observations[0]
            .expected_subject_blake3
            .clone(),
        "recovery-orchestrator".to_owned(),
        format!("sha256:{}", "8".repeat(64)),
        format!("sha256:{}", "9".repeat(64)),
        3,
    )
    .unwrap();
    records.push(Record::RecoveryProofReceiptV1 {
        schema_version: GENERATION_SCHEMA_VERSION,
        at_unix_ms: RECOVERED + 1,
        body: proof.clone(),
    });
    let review = RecoveryReviewReceiptRecordV1::verified_approval(
        request
            .subject
            .review_observation
            .expected_subject_blake3
            .clone(),
        vec![proof.proof_receipt_id().to_owned()],
        "recovery-orchestrator".to_owned(),
        "independent-reviewer".to_owned(),
        format!("sha256:{}", "a".repeat(64)),
    )
    .unwrap();
    records.push(Record::RecoveryReviewReceiptV1 {
        schema_version: GENERATION_SCHEMA_VERSION,
        at_unix_ms: RECOVERED + 2,
        body: review,
    });
    records.push(Record::RecoveryReceiptAdoptionV1 {
        schema_version: GENERATION_SCHEMA_VERSION,
        at_unix_ms: ADOPTED,
        body: record(request),
    });

    let projected = crate::coord::state::summaries(&records, ADOPTED).unwrap();
    assert!(projected.values().all(|claim| {
        claim.state == ClaimState::RecoveredReceipted && claim.recovery_adoption.is_some()
    }));
}
