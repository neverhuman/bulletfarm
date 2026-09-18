use super::*;

fn tagged(prefix: &str, marker: char) -> String {
    format!("{prefix}{}", marker.to_string().repeat(64))
}

fn proof() -> RecoveryProofReceiptRecordV1 {
    RecoveryProofReceiptRecordV1::verified_pass(
        tagged("blake3:", '1'),
        "recovery-orchestrator".to_owned(),
        tagged("sha256:", '2'),
        tagged("sha256:", '3'),
        12,
    )
    .unwrap()
}

#[test]
fn verified_pass_is_identity_bound_and_refuses_non_pass_counts() {
    let value = proof();
    value.validate().unwrap();
    assert!(value.proof_receipt_id.starts_with("rpf_"));

    let mut failed = value.clone();
    failed.failed_checks = 1;
    assert!(failed.validate().is_err());

    let mut tampered = value;
    tampered.proof_output_sha256 = tagged("sha256:", '4');
    assert!(tampered.validate().is_err());
}

#[test]
fn verified_review_binds_sorted_proofs_and_independence() {
    let proof = proof();
    let value = RecoveryReviewReceiptRecordV1::verified_approval(
        tagged("blake3:", '4'),
        vec![proof.proof_receipt_id.clone()],
        "recovery-orchestrator".to_owned(),
        "independent-reviewer".to_owned(),
        tagged("sha256:", '5'),
    )
    .unwrap();
    value.validate().unwrap();
    assert!(value.review_receipt_id.starts_with("rrv_"));

    assert!(
        RecoveryReviewReceiptRecordV1::verified_approval(
            tagged("blake3:", '4'),
            vec![proof.proof_receipt_id.clone()],
            "same-actor".to_owned(),
            "same-actor".to_owned(),
            tagged("sha256:", '5'),
        )
        .is_err()
    );

    let mut duplicate = value;
    duplicate.proof_receipt_ids.push(proof.proof_receipt_id);
    assert!(duplicate.validate().is_err());
}
