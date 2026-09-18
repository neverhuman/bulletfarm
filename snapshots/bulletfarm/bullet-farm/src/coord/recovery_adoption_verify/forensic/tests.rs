use super::*;
use crate::coord::model::recovery_adoption_request_fixture;

fn assert_order_mismatch(request: &RecoveryReceiptAdoptionRequestV1) {
    assert_eq!(
        verify_causal_order(request).unwrap_err().code(),
        "RECOVERY_EVIDENCE_MISMATCH"
    );
}

#[test]
fn fixture_references_are_strictly_causal() {
    verify_causal_order(&recovery_adoption_request_fixture()).unwrap();
}

#[test]
fn equal_or_reversed_record_positions_refuse() {
    let fixture = recovery_adoption_request_fixture();
    let mut equal_claim_handoff = fixture.clone();
    equal_claim_handoff.subject.claims[0]
        .handoff_observation
        .record_index = equal_claim_handoff.subject.claims[0]
        .trusted_claim_record
        .record_index;
    assert_order_mismatch(&equal_claim_handoff);

    let mut reversed_claim_handoff = fixture.clone();
    reversed_claim_handoff.subject.claims[0]
        .handoff_observation
        .record_index = reversed_claim_handoff.subject.claims[0]
        .trusted_claim_record
        .record_index
        - 1;
    assert_order_mismatch(&reversed_claim_handoff);

    let mut equal_handoff_group = fixture.clone();
    equal_handoff_group
        .subject
        .group_receipt_observation
        .record_index = equal_handoff_group.subject.claims[1]
        .handoff_observation
        .record_index;
    assert_order_mismatch(&equal_handoff_group);

    let mut reversed_handoff_group = fixture;
    reversed_handoff_group
        .subject
        .group_receipt_observation
        .record_index = reversed_handoff_group.subject.claims[1]
        .handoff_observation
        .record_index
        - 1;
    assert_order_mismatch(&reversed_handoff_group);
}

#[test]
fn interleaved_byte_ranges_refuse_even_with_increasing_indexes() {
    let fixture = recovery_adoption_request_fixture();
    let mut claim_handoff = fixture.clone();
    claim_handoff.subject.claims[0]
        .trusted_claim_record
        .byte_end = claim_handoff.subject.claims[0]
        .handoff_observation
        .byte_start
        + 1;
    assert_order_mismatch(&claim_handoff);

    let mut handoff_group = fixture;
    handoff_group.subject.claims[1].handoff_observation.byte_end =
        handoff_group.subject.group_receipt_observation.byte_start + 1;
    assert_order_mismatch(&handoff_group);
}
