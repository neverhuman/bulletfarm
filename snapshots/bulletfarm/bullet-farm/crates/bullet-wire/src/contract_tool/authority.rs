use serde_json::{Value, json};

use crate::{
    AcceptanceContractId, AttemptId, AuthorityAudience, AuthorityClaims, AuthoritySigningKey,
    CandidateId, ChangeId, CheckpointId, CommandId, ContentId, EffectIntentId, EffectReceiptId,
    EventId, EvidenceId, GateId, GraphRevisionId, IntegrationProofRoot, MissionId, MutationId,
    MutationOperation, MutationReservationId, OrganizationId, PlanRevisionId, PrincipalId,
    ProviderProfileId, RepositoryId, RpcRequestId, RunnerId, ScopeGrantId, SelectionGroupId,
    SourceDescriptorId, VariantId, WireError, WorkPackageId, WorkspaceId, authority_request_digest,
    canonical_json, hash_canonical,
};

/// Fixture-only deterministic PASETO v4.public test vector. Its private half
/// exists only here and in tests; normative policy never trusts it.
pub(super) const SECRET_KEY: [u8; 64] = [
    180, 203, 251, 67, 223, 76, 226, 16, 114, 125, 149, 62, 74, 113, 51, 7, 250, 25, 187, 125, 159,
    133, 4, 20, 56, 217, 225, 27, 148, 42, 55, 116, 30, 185, 219, 187, 188, 4, 124, 3, 253, 112,
    96, 78, 0, 113, 240, 152, 126, 22, 178, 139, 117, 114, 37, 193, 31, 0, 65, 93, 14, 32, 177,
    162,
];
pub(super) const PUBLIC_KEY_HEX: &str =
    "1eb9dbbbbc047c03fd70604e0071f0987e16b28b757225c11f00415d0e20b1a2";

pub(super) fn authority_golden() -> Result<(Value, crate::Blake3Digest), WireError> {
    let request = crate::v1alpha1::ApplyPatchRequestV1 {
        schema_version: "v1alpha1".to_owned(),
        mutation_id: format!("mut_{}", "1".repeat(64)),
        repository_id: format!("rep_{}", "4".repeat(64)),
        workspace_id: format!("wsp_{}", "f".repeat(64)),
        workspace_generation: 7,
        proposal: crate::v1alpha1::PatchProposalV1 {
            schema_version: "v1alpha1".to_owned(),
            proposal_id: format!("cnt_{}", "a".repeat(64)),
            producing_attempt_id: format!("atm_{}", "d".repeat(64)),
            base_checkpoint_id: format!("ckp_{}", "5".repeat(64)),
            base_checkpoint_digest: "6".repeat(64),
            operations: vec![crate::v1alpha1::PatchOperationV1 {
                schema_version: "v1alpha1".to_owned(),
                path: "src/lib.rs".to_owned(),
                preimage_kind: crate::v1alpha1::PatchPreimageKindV1::Digest,
                preimage_digest: Some("7".repeat(64)),
                mutation_kind: crate::v1alpha1::PatchMutationKindV1::Write,
                content_utf8: Some("pub fn golden() {}\n".to_owned()),
            }],
            gate_ids: vec![format!("gat_{}", "8".repeat(64))],
        },
    };
    let request_digest = authority_request_digest(&request)?;
    let claims = AuthorityClaims {
        schema_version: "v1alpha1".to_owned(),
        issuer: "bullet-kernel-local".to_owned(),
        audience: AuthorityAudience::BulletGitd,
        operation: MutationOperation::ApplyPatch,
        request_digest,
        mutation_id: parse_id("mut_", '1')?,
        subject_principal: parse_id("pri_", '2')?,
        organization_id: parse_id("org_", '3')?,
        repository_id: parse_id("rep_", '4')?,
        mission_id: parse_id("mis_", '5')?,
        acceptance_contract_id: parse_id("acc_", '6')?,
        plan_revision_id: parse_id("pln_", '7')?,
        graph_revision_id: parse_id("grf_", '8')?,
        graph_sequence: 9,
        work_package_id: parse_id("wpk_", 'a')?,
        selection_group_id: parse_id("sel_", 'b')?,
        variant_id: parse_id("var_", 'c')?,
        attempt_id: parse_id("atm_", 'd')?,
        attempt_fence: 10,
        runner_id: parse_id("run_", 'e')?,
        runner_epoch: 11,
        workspace_id: parse_id("wsp_", 'f')?,
        workspace_generation: request.workspace_generation,
        workspace_nonce: crate::Blake3Digest::from_bytes([12; 32]),
        scope_grant_digest: crate::Blake3Digest::from_bytes([13; 32]),
        scope_revision: 14,
        context_revision: 15,
        configuration_snapshot_id: parse_id("cnt_", '1')?,
        configuration_generation: 16,
        policy_snapshot_id: parse_id("cnt_", '2')?,
        policy_generation: 17,
        routing_snapshot_id: parse_id("cnt_", '3')?,
        routing_generation: 18,
        provider: "claude".to_owned(),
        model: "claude-test".to_owned(),
        adapter: "claude-stream-json-v1".to_owned(),
        provider_profile_id: parse_id("prf_", '4')?,
        credential_generation: 19,
        authority_epoch: 20,
        freeze_generation: 0,
        issued_at_unix_ms: 1_800_000_000_000,
        not_before_unix_ms: 1_800_000_000_000,
        expires_at_unix_ms: 1_800_000_015_000,
        token_nonce: crate::Blake3Digest::from_bytes([21; 32]),
    };
    let signer =
        AuthoritySigningKey::from_bytes("bullet-kernel-local", "authority-test-1", &SECRET_KEY)?;
    let envelope = signer.sign_for_request(&claims, &request)?;
    let envelope_digest = envelope.digest()?;
    let reservation_id = parse_id("rsv_", '9')?;
    let permit_claims = crate::MutationPermitClaims {
        schema_version: "v1alpha1".to_owned(),
        issuer: "bullet-kernel-local".to_owned(),
        audience: AuthorityAudience::BulletGitd,
        operation: MutationOperation::ApplyPatch,
        authority_envelope_digest: envelope_digest,
        authority_token_nonce: claims.token_nonce,
        mutation_id: claims.mutation_id.clone(),
        reservation_id,
        request_digest,
        repository_id: claims.repository_id.clone(),
        workspace_id: claims.workspace_id.clone(),
        workspace_generation: request.workspace_generation,
        attempt_id: claims.attempt_id.clone(),
        attempt_fence: claims.attempt_fence,
        authority_epoch: claims.authority_epoch,
        freeze_generation: claims.freeze_generation,
        issued_at_unix_ms: 1_800_000_000_100,
        not_before_unix_ms: 1_800_000_000_100,
        expires_at_unix_ms: 1_800_000_001_100,
        permit_nonce: crate::Blake3Digest::from_bytes([22; 32]),
    };
    let permit = signer.sign_mutation_permit(&permit_claims)?;
    let permit_digest = permit.digest()?;
    let decision = crate::FinalAuthorityDecision {
        schema_version: "v1alpha1".to_owned(),
        decision: crate::AuthorityDecisionKind::Authorized,
        replay: crate::ReplayDisposition::Fresh,
        mutation_id: claims.mutation_id.clone(),
        operation: MutationOperation::ApplyPatch,
        request_digest,
        reservation_id: Some(permit_claims.reservation_id.clone()),
        permit: Some(permit.clone()),
        replay_result: None,
        reason_code: None,
    };
    decision.validate_shape()?;
    let settlement_request = crate::MutationSettlementRequest {
        schema_version: "v1alpha1".to_owned(),
        reservation_id: permit_claims.reservation_id.clone(),
        mutation_id: claims.mutation_id.clone(),
        operation: MutationOperation::ApplyPatch,
        request_digest,
        permit: permit.clone(),
        permit_digest,
        outcome: crate::MutationOutcome::Committed,
        result_digest: crate::Blake3Digest::from_bytes([23; 32]),
        completed_at_unix_ms: 1_800_000_000_900,
    };
    settlement_request.validate_shape()?;
    let settlement_result = crate::MutationSettlementResult {
        schema_version: "v1alpha1".to_owned(),
        status: crate::SettlementStatus::Accepted,
        replay: crate::ReplayDisposition::Fresh,
        mutation_id: claims.mutation_id.clone(),
        reservation_id: permit_claims.reservation_id.clone(),
        result_digest: Some(settlement_request.result_digest),
        reason_code: None,
    };
    settlement_result.validate()?;
    let value = json!({
        "claims_canonical_json": String::from_utf8(canonical_json(&claims)?)
            .map_err(|error| WireError::new("GOLDEN_ENCODING_FAILED", error.to_string()))?,
        "claims_digest": claims.digest()?,
        "envelope": envelope,
        "envelope_digest": envelope_digest,
        "implicit_assertion_utf8": "bullet-farm.authority.v1alpha1",
        "mutation_decision": decision,
        "mutation_permit": permit,
        "mutation_permit_claims_canonical_json": String::from_utf8(canonical_json(&permit_claims)?)
            .map_err(|error| WireError::new("GOLDEN_ENCODING_FAILED", error.to_string()))?,
        "mutation_permit_digest": permit_digest,
        "mutation_permit_implicit_assertion_utf8": "bullet-farm.mutation-permit.v1alpha1",
        "public_key_hex": PUBLIC_KEY_HEX,
        "request": request,
        "request_digest": request_digest,
        "request_domain": MutationOperation::ApplyPatch.request_domain(),
        "schema_version": "v1alpha1",
        "settlement_request": settlement_request,
        "settlement_result": settlement_result,
    });
    let digest = hash_canonical("authority.golden.v1alpha1", &value)?;
    Ok((value, digest))
}

pub(super) trait GoldenId: Sized {
    fn parse_golden_id(value: &str) -> Result<Self, WireError>;
}

macro_rules! golden_ids {
    ($($type:ty),+ $(,)?) => {
        $(
            impl GoldenId for $type {
                fn parse_golden_id(value: &str) -> Result<Self, WireError> {
                    <$type>::parse_checked(value)
                }
            }
        )+
    };
}

golden_ids!(
    AcceptanceContractId,
    AttemptId,
    CandidateId,
    ChangeId,
    CheckpointId,
    CommandId,
    ContentId,
    EffectIntentId,
    EffectReceiptId,
    EventId,
    EvidenceId,
    GateId,
    GraphRevisionId,
    IntegrationProofRoot,
    MissionId,
    MutationId,
    MutationReservationId,
    OrganizationId,
    PlanRevisionId,
    PrincipalId,
    ProviderProfileId,
    RepositoryId,
    RpcRequestId,
    RunnerId,
    ScopeGrantId,
    SelectionGroupId,
    SourceDescriptorId,
    VariantId,
    WorkPackageId,
    WorkspaceId,
);

pub(super) fn parse_id<T: GoldenId>(prefix: &str, fill: char) -> Result<T, WireError> {
    let text = format!("{prefix}{}", fill.to_string().repeat(64));
    T::parse_golden_id(&text).map_err(|error| WireError::new("GOLDEN_ID_FAILED", error.to_string()))
}
