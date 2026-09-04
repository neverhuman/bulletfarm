use std::str::FromStr;

use bullet_wire::{
    AttemptId, AuthorityAudience, AuthorityDecisionKind, AuthorityRequest, AuthoritySigningKey,
    AuthorityVerificationKey, Blake3Digest, FinalAuthorityDecision, MutationId, MutationOperation,
    MutationOutcome, MutationPermitClaims, MutationPermitExpectation, MutationReplayResult,
    MutationReservationId, MutationResultState, MutationSettlementRequest,
    MutationSettlementResult, ReplayDisposition, RepositoryId, SettlementStatus, WorkspaceId,
    authority_request_digest,
};

const SECRET_KEY: [u8; 64] = [
    180, 203, 251, 67, 223, 76, 226, 16, 114, 125, 149, 62, 74, 113, 51, 7, 250, 25, 187, 125, 159,
    133, 4, 20, 56, 217, 225, 27, 148, 42, 55, 116, 30, 185, 219, 187, 188, 4, 124, 3, 253, 112,
    96, 78, 0, 113, 240, 152, 126, 22, 178, 139, 117, 114, 37, 193, 31, 0, 65, 93, 14, 32, 177,
    162,
];
const PUBLIC_KEY: [u8; 32] = [
    30, 185, 219, 187, 188, 4, 124, 3, 253, 112, 96, 78, 0, 113, 240, 152, 126, 22, 178, 139, 117,
    114, 37, 193, 31, 0, 65, 93, 14, 32, 177, 162,
];

fn id<T: FromStr>(prefix: &str, value: char) -> T
where
    T::Err: std::fmt::Debug,
{
    format!("{prefix}{}", value.to_string().repeat(64))
        .parse()
        .unwrap()
}

fn digest(value: u8) -> Blake3Digest {
    Blake3Digest::from_bytes([value; 32])
}

fn signer() -> AuthoritySigningKey {
    AuthoritySigningKey::from_bytes("bullet-kernel-local", "authority-test-1", &SECRET_KEY).unwrap()
}

fn verifier() -> AuthorityVerificationKey {
    AuthorityVerificationKey::from_bytes("bullet-kernel-local", "authority-test-1", &PUBLIC_KEY)
        .unwrap()
}

fn permit_claims() -> MutationPermitClaims {
    MutationPermitClaims {
        schema_version: "v1alpha1".to_owned(),
        issuer: "bullet-kernel-local".to_owned(),
        audience: AuthorityAudience::BulletGitd,
        operation: MutationOperation::ApplyPatch,
        authority_envelope_digest: digest(1),
        authority_token_nonce: digest(2),
        mutation_id: id::<MutationId>("mut_", '3'),
        reservation_id: id::<MutationReservationId>("rsv_", '4'),
        request_digest: digest(5),
        repository_id: id::<RepositoryId>("rep_", '6'),
        workspace_id: id::<WorkspaceId>("wsp_", '7'),
        workspace_generation: 8,
        attempt_id: id::<AttemptId>("atm_", '9'),
        attempt_fence: 10,
        authority_epoch: 11,
        freeze_generation: 0,
        issued_at_unix_ms: 1_800_000_000_000,
        not_before_unix_ms: 1_800_000_000_000,
        expires_at_unix_ms: 1_800_000_001_000,
        permit_nonce: digest(12),
    }
}

fn expectation(claims: &MutationPermitClaims) -> MutationPermitExpectation {
    MutationPermitExpectation {
        audience: claims.audience,
        operation: claims.operation,
        authority_envelope_digest: claims.authority_envelope_digest,
        authority_token_nonce: claims.authority_token_nonce,
        mutation_id: claims.mutation_id.clone(),
        reservation_id: claims.reservation_id.clone(),
        request_digest: claims.request_digest,
        repository_id: claims.repository_id.clone(),
        workspace_id: claims.workspace_id.clone(),
        workspace_generation: claims.workspace_generation,
        attempt_id: claims.attempt_id.clone(),
        attempt_fence: claims.attempt_fence,
        authority_epoch: claims.authority_epoch,
        freeze_generation: claims.freeze_generation,
        now_unix_ms: claims.not_before_unix_ms,
    }
}

#[test]
fn permit_is_typed_signed_short_lived_and_cross_language_decodable() {
    let claims = permit_claims();
    let permit = signer().sign_mutation_permit(&claims).unwrap();
    assert_eq!(
        verifier()
            .verify_mutation_permit(&permit, &expectation(&claims))
            .unwrap(),
        claims
    );
    assert_eq!(signer().sign_mutation_permit(&claims).unwrap(), permit);
    serde_json::from_value::<bullet_wire::v1alpha1::MutationPermitClaimsV1>(
        serde_json::to_value(&claims).unwrap(),
    )
    .unwrap();
    serde_json::from_value::<bullet_wire::v1alpha1::SignedMutationPermitV1>(
        serde_json::to_value(&permit).unwrap(),
    )
    .unwrap();

    let mut too_long = claims;
    too_long.expires_at_unix_ms += 1;
    assert_eq!(
        signer().sign_mutation_permit(&too_long).unwrap_err().code(),
        "INVALID_MUTATION_PERMIT_WINDOW"
    );

    let mut wrong_audience = permit_claims();
    wrong_audience.operation = MutationOperation::DispatchEffect;
    assert_eq!(
        signer()
            .sign_mutation_permit(&wrong_audience)
            .unwrap_err()
            .code(),
        "INVALID_MUTATION_PERMIT_AUDIENCE"
    );
}

#[test]
fn every_reserved_subject_field_and_time_boundary_fail_closed() {
    let claims = permit_claims();
    let permit = signer().sign_mutation_permit(&claims).unwrap();
    let expected = expectation(&claims);
    let mut changed = Vec::new();

    let mut value = expected.clone();
    value.audience = AuthorityAudience::EffectBroker;
    changed.push(value);
    let mut value = expected.clone();
    value.operation = MutationOperation::Checkpoint;
    changed.push(value);
    let mut value = expected.clone();
    value.authority_envelope_digest = digest(20);
    changed.push(value);
    let mut value = expected.clone();
    value.authority_token_nonce = digest(21);
    changed.push(value);
    let mut value = expected.clone();
    value.mutation_id = id("mut_", 'a');
    changed.push(value);
    let mut value = expected.clone();
    value.reservation_id = id("rsv_", 'b');
    changed.push(value);
    let mut value = expected.clone();
    value.request_digest = digest(22);
    changed.push(value);
    let mut value = expected.clone();
    value.repository_id = id("rep_", 'c');
    changed.push(value);
    let mut value = expected.clone();
    value.workspace_id = id("wsp_", 'd');
    changed.push(value);
    let mut value = expected.clone();
    value.workspace_generation += 1;
    changed.push(value);
    let mut value = expected.clone();
    value.attempt_id = id("atm_", 'e');
    changed.push(value);
    let mut value = expected.clone();
    value.attempt_fence += 1;
    changed.push(value);
    let mut value = expected.clone();
    value.authority_epoch += 1;
    changed.push(value);
    let mut value = expected.clone();
    value.freeze_generation += 1;
    changed.push(value);

    for changed_expectation in changed {
        assert_eq!(
            verifier()
                .verify_mutation_permit(&permit, &changed_expectation)
                .unwrap_err()
                .code(),
            "MUTATION_PERMIT_SUBJECT_MISMATCH"
        );
    }

    let mut at_expiry = expected.clone();
    at_expiry.now_unix_ms = claims.expires_at_unix_ms - 1;
    verifier()
        .verify_mutation_permit(&permit, &at_expiry)
        .unwrap();
    at_expiry.now_unix_ms = claims.expires_at_unix_ms;
    assert_eq!(
        verifier()
            .verify_mutation_permit(&permit, &at_expiry)
            .unwrap_err()
            .code(),
        "MUTATION_PERMIT_EXPIRED"
    );
}

#[test]
fn authority_tokens_cannot_be_relabelled_as_mutation_permits() {
    let claims = permit_claims();
    let permit = signer().sign_mutation_permit(&claims).unwrap();
    let mut damaged = permit.clone();
    let replacement = if damaged.paseto.ends_with('A') {
        'B'
    } else {
        'A'
    };
    damaged.paseto.pop();
    damaged.paseto.push(replacement);
    assert_eq!(
        verifier()
            .verify_mutation_permit(&damaged, &expectation(&claims))
            .unwrap_err()
            .code(),
        "INVALID_MUTATION_PERMIT_SIGNATURE"
    );

    let wrong_key =
        AuthorityVerificationKey::from_bytes("another-issuer", "authority-test-1", &PUBLIC_KEY)
            .unwrap();
    assert_eq!(
        wrong_key
            .verify_mutation_permit(&permit, &expectation(&claims))
            .unwrap_err()
            .code(),
        "MUTATION_PERMIT_KEY_MISMATCH"
    );
}

#[test]
fn decision_replay_and_settlement_branches_are_exclusive() {
    let claims = permit_claims();
    let permit = signer().sign_mutation_permit(&claims).unwrap();
    let mut decision = FinalAuthorityDecision {
        schema_version: "v1alpha1".to_owned(),
        decision: AuthorityDecisionKind::Authorized,
        replay: ReplayDisposition::Fresh,
        mutation_id: claims.mutation_id.clone(),
        operation: claims.operation,
        request_digest: claims.request_digest,
        reservation_id: Some(claims.reservation_id.clone()),
        permit: Some(permit.clone()),
        replay_result: None,
        reason_code: None,
    };
    decision.validate_shape().unwrap();
    serde_json::from_value::<bullet_wire::v1alpha1::FinalAuthorityDecisionV1>(
        serde_json::to_value(&decision).unwrap(),
    )
    .unwrap();

    decision.reason_code = Some("unexpected".to_owned());
    assert_eq!(
        decision.validate_shape().unwrap_err().code(),
        "INVALID_AUTHORITY_DECISION"
    );
    decision.reason_code = None;
    decision.decision = AuthorityDecisionKind::Settled;
    decision.replay = ReplayDisposition::ExactReplay;
    decision.permit = None;
    decision.replay_result = Some(MutationReplayResult {
        schema_version: "v1alpha1".to_owned(),
        reservation_id: claims.reservation_id.clone(),
        mutation_id: claims.mutation_id.clone(),
        operation: claims.operation,
        request_digest: claims.request_digest,
        state: MutationResultState::Committed,
        result_digest: Some(digest(30)),
        completed_at_unix_ms: Some(1_800_000_000_900),
    });
    decision.validate_shape().unwrap();
    decision.replay_result.as_mut().unwrap().request_digest = digest(31);
    assert_eq!(
        decision.validate_shape().unwrap_err().code(),
        "AUTHORITY_REPLAY_CONFLICT"
    );

    let mut settlement = MutationSettlementRequest {
        schema_version: "v1alpha1".to_owned(),
        reservation_id: claims.reservation_id.clone(),
        mutation_id: claims.mutation_id.clone(),
        operation: claims.operation,
        request_digest: claims.request_digest,
        permit: permit.clone(),
        permit_digest: permit.digest().unwrap(),
        outcome: MutationOutcome::Committed,
        result_digest: digest(32),
        completed_at_unix_ms: 1_800_000_000_900,
    };
    settlement.validate_shape().unwrap();
    serde_json::from_value::<bullet_wire::v1alpha1::MutationSettlementRequestV1>(
        serde_json::to_value(&settlement).unwrap(),
    )
    .unwrap();
    settlement.permit_digest = digest(33);
    assert_eq!(
        settlement.validate_shape().unwrap_err().code(),
        "INVALID_MUTATION_SETTLEMENT"
    );

    let mut result = MutationSettlementResult {
        schema_version: "v1alpha1".to_owned(),
        status: SettlementStatus::Accepted,
        replay: ReplayDisposition::Fresh,
        mutation_id: claims.mutation_id,
        reservation_id: claims.reservation_id,
        result_digest: Some(digest(32)),
        reason_code: None,
    };
    result.validate().unwrap();
    result.status = SettlementStatus::Conflict;
    assert_eq!(
        result.validate().unwrap_err().code(),
        "INVALID_MUTATION_SETTLEMENT_RESULT"
    );
}

#[test]
fn decision_and_settlement_verification_reject_cross_subject_permits() {
    let claims = permit_claims();
    let permit = signer().sign_mutation_permit(&claims).unwrap();
    let expected = expectation(&claims);
    let decision = FinalAuthorityDecision {
        schema_version: "v1alpha1".to_owned(),
        decision: AuthorityDecisionKind::Authorized,
        replay: ReplayDisposition::Fresh,
        mutation_id: claims.mutation_id.clone(),
        operation: claims.operation,
        request_digest: claims.request_digest,
        reservation_id: Some(claims.reservation_id.clone()),
        permit: Some(permit.clone()),
        replay_result: None,
        reason_code: None,
    };
    assert_eq!(
        decision
            .verify_authorized_permit(&verifier(), &expected)
            .unwrap(),
        claims
    );

    let mut changed_decisions = Vec::new();
    let mut changed = decision.clone();
    changed.mutation_id = id("mut_", 'a');
    changed_decisions.push(changed);
    let mut changed = decision.clone();
    changed.reservation_id = Some(id("rsv_", 'b'));
    changed_decisions.push(changed);
    let mut changed = decision.clone();
    changed.operation = MutationOperation::Checkpoint;
    changed_decisions.push(changed);
    let mut changed = decision.clone();
    changed.request_digest = digest(40);
    changed_decisions.push(changed);
    for changed in changed_decisions {
        assert_eq!(
            changed
                .verify_authorized_permit(&verifier(), &expected)
                .unwrap_err()
                .code(),
            "AUTHORITY_DECISION_SUBJECT_MISMATCH"
        );
    }

    let mut tampered_decision = decision.clone();
    tampered_decision.permit.as_mut().unwrap().paseto.push('A');
    assert_eq!(
        tampered_decision
            .verify_authorized_permit(&verifier(), &expected)
            .unwrap_err()
            .code(),
        "INVALID_MUTATION_PERMIT_SIGNATURE"
    );

    let settlement_subject = expected.subject();
    let settlement = MutationSettlementRequest {
        schema_version: "v1alpha1".to_owned(),
        reservation_id: claims.reservation_id.clone(),
        mutation_id: claims.mutation_id.clone(),
        operation: claims.operation,
        request_digest: claims.request_digest,
        permit: permit.clone(),
        permit_digest: permit.digest().unwrap(),
        outcome: MutationOutcome::Committed,
        result_digest: digest(41),
        completed_at_unix_ms: 1_800_000_500_000,
    };
    assert_eq!(
        settlement
            .verify_permit(&verifier(), &settlement_subject)
            .unwrap(),
        claims
    );

    let mut changed_settlements = Vec::new();
    let mut changed = settlement.clone();
    changed.mutation_id = id("mut_", 'a');
    changed_settlements.push(changed);
    let mut changed = settlement.clone();
    changed.reservation_id = id("rsv_", 'b');
    changed_settlements.push(changed);
    let mut changed = settlement.clone();
    changed.operation = MutationOperation::Checkpoint;
    changed_settlements.push(changed);
    let mut changed = settlement.clone();
    changed.request_digest = digest(42);
    changed_settlements.push(changed);
    for changed in changed_settlements {
        assert_eq!(
            changed
                .verify_permit(&verifier(), &settlement_subject)
                .unwrap_err()
                .code(),
            "MUTATION_SETTLEMENT_SUBJECT_MISMATCH"
        );
    }

    let mut tampered_settlement = settlement;
    tampered_settlement.permit.paseto.push('A');
    tampered_settlement.permit_digest = tampered_settlement.permit.digest().unwrap();
    assert_eq!(
        tampered_settlement
            .verify_permit(&verifier(), &settlement_subject)
            .unwrap_err()
            .code(),
        "INVALID_MUTATION_PERMIT_SIGNATURE"
    );
}

#[test]
fn replay_time_and_settlement_reasons_match_generated_constraints() {
    let claims = permit_claims();
    let replay = MutationReplayResult {
        schema_version: "v1alpha1".to_owned(),
        reservation_id: claims.reservation_id.clone(),
        mutation_id: claims.mutation_id.clone(),
        operation: claims.operation,
        request_digest: claims.request_digest,
        state: MutationResultState::Committed,
        result_digest: Some(digest(50)),
        completed_at_unix_ms: Some(9_007_199_254_740_992),
    };
    assert_eq!(
        replay.validate().unwrap_err().code(),
        "INVALID_MUTATION_REPLAY_RESULT"
    );

    for invalid_reason in ["", "line\nbreak"] {
        let result = MutationSettlementResult {
            schema_version: "v1alpha1".to_owned(),
            status: SettlementStatus::Conflict,
            replay: ReplayDisposition::Conflict,
            mutation_id: claims.mutation_id.clone(),
            reservation_id: claims.reservation_id.clone(),
            result_digest: None,
            reason_code: Some(invalid_reason.to_owned()),
        };
        assert_eq!(
            result.validate().unwrap_err().code(),
            "INVALID_MUTATION_SETTLEMENT_RESULT"
        );
    }
}

#[test]
fn exact_request_types_validate_before_their_fixed_domain_is_hashed() {
    let mut request = bullet_wire::v1alpha1::ApplyPatchRequestV1 {
        schema_version: "v1alpha1".to_owned(),
        mutation_id: format!("mut_{}", "1".repeat(64)),
        repository_id: format!("rep_{}", "2".repeat(64)),
        workspace_id: format!("wsp_{}", "3".repeat(64)),
        workspace_generation: 4,
        proposal: bullet_wire::v1alpha1::PatchProposalV1 {
            schema_version: "v1alpha1".to_owned(),
            proposal_id: format!("cnt_{}", "5".repeat(64)),
            producing_attempt_id: format!("atm_{}", "6".repeat(64)),
            base_checkpoint_id: format!("ckp_{}", "7".repeat(64)),
            base_checkpoint_digest: "8".repeat(64),
            operations: vec![bullet_wire::v1alpha1::PatchOperationV1 {
                schema_version: "v1alpha1".to_owned(),
                path: "src/lib.rs".to_owned(),
                preimage_kind: bullet_wire::v1alpha1::PatchPreimageKindV1::Digest,
                preimage_digest: Some("9".repeat(64)),
                mutation_kind: bullet_wire::v1alpha1::PatchMutationKindV1::Write,
                content_utf8: Some("one\ntwo\n".to_owned()),
            }],
            gate_ids: vec![format!("gat_{}", "a".repeat(64))],
        },
    };
    request.validate().unwrap();
    assert_eq!(
        authority_request_digest(&request).unwrap(),
        request.digest().unwrap()
    );

    request.workspace_generation = 0;
    assert!(authority_request_digest(&request).is_err());
    request.workspace_generation = 4;
    request
        .proposal
        .operations
        .push(request.proposal.operations[0].clone());
    assert_eq!(
        authority_request_digest(&request).unwrap_err().code(),
        "PATH_COLLISION"
    );
}
