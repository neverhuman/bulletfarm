use std::{fs, path::PathBuf, str::FromStr};

use bullet_wire::{
    AcceptanceContractId, AttemptId, AuthorityAudience, AuthorityClaims, AuthorityExpectation,
    AuthoritySigningKey, AuthorityVerificationKey, Blake3Digest, ContentId,
    FinalAuthorityCheckRequest, GraphRevisionId, MissionId, MutationId, MutationOperation,
    MutationPermitClaims, MutationPermitExpectation, MutationSettlementRequest,
    MutationSettlementResult, OrganizationId, PlanRevisionId, PrincipalId, ProviderProfileId,
    RepositoryId, RunnerId, SelectionGroupId, SignedAuthorityEnvelope, SignedMutationPermit,
    VariantId, WorkPackageId, WorkspaceId, authority_request_digest, canonical_json,
    decode_canonical, hash_canonical,
};
use pasetors::{
    keys::AsymmetricSecretKey,
    version4::{PublicToken, V4},
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

fn claims() -> AuthorityClaims {
    let request_digest = authority_request_digest(&apply_request()).unwrap();
    AuthorityClaims {
        schema_version: "v1alpha1".to_owned(),
        issuer: "bullet-kernel-local".to_owned(),
        audience: AuthorityAudience::BulletGitd,
        operation: MutationOperation::ApplyPatch,
        request_digest,
        mutation_id: id::<MutationId>("mut_", '1'),
        subject_principal: id::<PrincipalId>("pri_", '2'),
        organization_id: id::<OrganizationId>("org_", '3'),
        repository_id: id::<RepositoryId>("rep_", '4'),
        mission_id: id::<MissionId>("mis_", '5'),
        acceptance_contract_id: id::<AcceptanceContractId>("acc_", '6'),
        plan_revision_id: id::<PlanRevisionId>("pln_", '7'),
        graph_revision_id: id::<GraphRevisionId>("grf_", '8'),
        graph_sequence: 9,
        work_package_id: id::<WorkPackageId>("wpk_", 'a'),
        selection_group_id: id::<SelectionGroupId>("sel_", 'b'),
        variant_id: id::<VariantId>("var_", 'c'),
        attempt_id: id::<AttemptId>("atm_", 'd'),
        attempt_fence: 10,
        runner_id: id::<RunnerId>("run_", 'e'),
        runner_epoch: 11,
        workspace_id: id::<WorkspaceId>("wsp_", 'f'),
        workspace_generation: 7,
        workspace_nonce: digest(12),
        scope_grant_digest: digest(13),
        scope_revision: 14,
        context_revision: 15,
        configuration_snapshot_id: id::<ContentId>("cnt_", '1'),
        configuration_generation: 16,
        policy_snapshot_id: id::<ContentId>("cnt_", '2'),
        policy_generation: 17,
        routing_snapshot_id: id::<ContentId>("cnt_", '3'),
        routing_generation: 18,
        provider: "claude".to_owned(),
        model: "claude-test".to_owned(),
        adapter: "claude-stream-json-v1".to_owned(),
        provider_profile_id: id::<ProviderProfileId>("prf_", '4'),
        credential_generation: 19,
        authority_epoch: 20,
        freeze_generation: 0,
        issued_at_unix_ms: 1_800_000_000_000,
        not_before_unix_ms: 1_800_000_000_000,
        expires_at_unix_ms: 1_800_000_015_000,
        token_nonce: digest(21),
    }
}

fn apply_request() -> bullet_wire::v1alpha1::ApplyPatchRequestV1 {
    bullet_wire::v1alpha1::ApplyPatchRequestV1 {
        schema_version: "v1alpha1".to_owned(),
        mutation_id: format!("mut_{}", "1".repeat(64)),
        repository_id: format!("rep_{}", "4".repeat(64)),
        workspace_id: format!("wsp_{}", "f".repeat(64)),
        workspace_generation: 7,
        proposal: bullet_wire::v1alpha1::PatchProposalV1 {
            schema_version: "v1alpha1".to_owned(),
            proposal_id: format!("cnt_{}", "a".repeat(64)),
            producing_attempt_id: format!("atm_{}", "d".repeat(64)),
            base_checkpoint_id: format!("ckp_{}", "5".repeat(64)),
            base_checkpoint_digest: "6".repeat(64),
            operations: vec![bullet_wire::v1alpha1::PatchOperationV1 {
                schema_version: "v1alpha1".to_owned(),
                path: "src/lib.rs".to_owned(),
                preimage_kind: bullet_wire::v1alpha1::PatchPreimageKindV1::Digest,
                preimage_digest: Some("7".repeat(64)),
                mutation_kind: bullet_wire::v1alpha1::PatchMutationKindV1::Write,
                content_utf8: Some("pub fn golden() {}\n".to_owned()),
            }],
            gate_ids: vec![format!("gat_{}", "8".repeat(64))],
        },
    }
}

fn clone_request() -> bullet_wire::v1alpha1::CloneWorkspaceRequestV1 {
    let scope_grant = bullet_wire::v1alpha1::ScopeGrantV1 {
        schema_version: "v1alpha1".to_owned(),
        scope_grant_id: format!("sgr_{}", "1".repeat(64)),
        scope_revision: 14,
        normalized_paths: vec!["src/lib.rs".to_owned()],
        protected_resources: vec!["source".to_owned()],
        envelope_class: "read-only-source".to_owned(),
    };
    let scope_grant_digest = hash_canonical("authority.scope-grant.v1alpha1", &scope_grant)
        .unwrap()
        .to_string();
    bullet_wire::v1alpha1::CloneWorkspaceRequestV1 {
        schema_version: "v1alpha1".to_owned(),
        mutation_id: format!("mut_{}", "1".repeat(64)),
        repository_id: format!("rep_{}", "4".repeat(64)),
        workspace_id: format!("wsp_{}", "f".repeat(64)),
        base_oid: format!("sha1:{}", "1".repeat(40)),
        source_descriptor_id: format!("src_{}", "2".repeat(64)),
        workspace_generation: 7,
        scope_grant,
        scope_grant_digest,
        trusted_commit_time_unix_ms: 1_800_000_000_000,
    }
}

fn dispatch_request() -> bullet_wire::v1alpha1::DispatchEffectRequestV1 {
    bullet_wire::v1alpha1::DispatchEffectRequestV1 {
        schema_version: "v1alpha1".to_owned(),
        mutation_id: format!("mut_{}", "1".repeat(64)),
        repository_id: format!("rep_{}", "4".repeat(64)),
        workspace_id: format!("wsp_{}", "f".repeat(64)),
        workspace_generation: 7,
        effect_intent_id: format!("efi_{}", "1".repeat(64)),
        effect_intent_digest: "2".repeat(64),
        effect_kind: "protected-integration".to_owned(),
        endpoint_identity: "jeryu-local".to_owned(),
        logical_key: "candidate-fence-20".to_owned(),
        desired_state_digest: "3".repeat(64),
        expected_state_digest: "4".repeat(64),
        candidate_id: format!("can_{}", "5".repeat(64)),
        candidate_proof_root: format!("cpr_{}", "6".repeat(64)),
        policy_snapshot_id: format!("cnt_{}", "2".repeat(64)),
        authority_epoch: 20,
        freeze_generation: 0,
    }
}

fn signer() -> AuthoritySigningKey {
    AuthoritySigningKey::from_bytes("bullet-kernel-local", "authority-test-1", &SECRET_KEY).unwrap()
}

fn verifier() -> AuthorityVerificationKey {
    AuthorityVerificationKey::from_bytes("bullet-kernel-local", "authority-test-1", &PUBLIC_KEY)
        .unwrap()
}

fn expectation(claims: &AuthorityClaims) -> AuthorityExpectation {
    AuthorityExpectation {
        audience: claims.audience,
        operation: claims.operation,
        request_digest: claims.request_digest,
        now_unix_ms: claims.not_before_unix_ms,
    }
}

fn sign_raw_payload(payload: &[u8], purpose: &str) -> SignedAuthorityEnvelope {
    let secret = AsymmetricSecretKey::<V4>::from(&SECRET_KEY).unwrap();
    let footer = canonical_json(&serde_json::json!({
        "issuer": "bullet-kernel-local",
        "key_id": "authority-test-1",
        "purpose": purpose,
        "schema_version": "v1alpha1",
    }))
    .unwrap();
    SignedAuthorityEnvelope {
        schema_version: "v1alpha1".to_owned(),
        issuer: "bullet-kernel-local".to_owned(),
        key_id: "authority-test-1".to_owned(),
        paseto: PublicToken::sign(
            &secret,
            payload,
            Some(&footer),
            Some(b"bullet-farm.authority.v1alpha1"),
        )
        .unwrap(),
    }
}

#[test]
fn exact_paseto_round_trip_and_envelope_digest_are_deterministic() {
    let claims = claims();
    let request = apply_request();
    let envelope = signer().sign_for_request(&claims, &request).unwrap();
    assert!(envelope.paseto.starts_with("v4.public."));
    assert_eq!(
        verifier().verify(&envelope, &expectation(&claims)).unwrap(),
        claims
    );
    assert_eq!(
        signer().sign_for_request(&claims, &request).unwrap(),
        envelope
    );
    assert_eq!(envelope.digest().unwrap(), envelope.digest().unwrap());
    serde_json::from_value::<bullet_wire::v1alpha1::AuthorityClaimsV1>(
        serde_json::to_value(&claims).unwrap(),
    )
    .unwrap();
    serde_json::from_value::<bullet_wire::v1alpha1::SignedAuthorityEnvelopeV1>(
        serde_json::to_value(&envelope).unwrap(),
    )
    .unwrap();

    let mut final_check = FinalAuthorityCheckRequest {
        schema_version: "v1alpha1".to_owned(),
        envelope_digest: envelope.digest().unwrap(),
        envelope,
        mutation_id: claims.mutation_id.clone(),
        audience: claims.audience,
        operation: claims.operation,
        request_digest: claims.request_digest,
    };
    assert_eq!(
        final_check
            .verify(&verifier(), claims.not_before_unix_ms)
            .unwrap(),
        claims
    );
    final_check.mutation_id = id("mut_", '0');
    assert_eq!(
        final_check
            .verify(&verifier(), claims.not_before_unix_ms)
            .unwrap_err()
            .code(),
        "AUTHORITY_MUTATION_MISMATCH"
    );
    final_check.envelope_digest = digest(99);
    assert_eq!(
        final_check
            .verify(&verifier(), claims.not_before_unix_ms)
            .unwrap_err()
            .code(),
        "AUTHORITY_ENVELOPE_DIGEST_MISMATCH"
    );
}

#[test]
fn signature_footer_key_and_envelope_mutations_fail_closed() {
    let claims = claims();
    let envelope = signer()
        .sign_for_request(&claims, &apply_request())
        .unwrap();
    let expected = expectation(&claims);

    let mut damaged = envelope.clone();
    let replacement = if damaged.paseto.ends_with('A') {
        'B'
    } else {
        'A'
    };
    damaged.paseto.pop();
    damaged.paseto.push(replacement);
    assert!(verifier().verify(&damaged, &expected).is_err());

    for changed in [
        SignedAuthorityEnvelope {
            issuer: "another-issuer".to_owned(),
            ..envelope.clone()
        },
        SignedAuthorityEnvelope {
            key_id: "another-key".to_owned(),
            ..envelope.clone()
        },
        SignedAuthorityEnvelope {
            schema_version: "v2".to_owned(),
            ..envelope
        },
    ] {
        assert!(verifier().verify(&changed, &expected).is_err());
    }
}

#[test]
fn audience_operation_request_and_time_boundaries_are_exact() {
    let claims = claims();
    let envelope = signer()
        .sign_for_request(&claims, &apply_request())
        .unwrap();
    let mut expected = expectation(&claims);
    expected.audience = AuthorityAudience::EffectBroker;
    assert_eq!(
        verifier().verify(&envelope, &expected).unwrap_err().code(),
        "AUTHORITY_AUDIENCE_MISMATCH"
    );
    expected = expectation(&claims);
    expected.operation = MutationOperation::Checkpoint;
    assert_eq!(
        verifier().verify(&envelope, &expected).unwrap_err().code(),
        "AUTHORITY_OPERATION_MISMATCH"
    );
    expected = expectation(&claims);
    expected.request_digest = digest(99);
    assert_eq!(
        verifier().verify(&envelope, &expected).unwrap_err().code(),
        "AUTHORITY_REQUEST_MISMATCH"
    );
    expected = expectation(&claims);
    expected.now_unix_ms = claims.not_before_unix_ms - 1;
    assert_eq!(
        verifier().verify(&envelope, &expected).unwrap_err().code(),
        "AUTHORITY_NOT_YET_VALID"
    );
    expected.now_unix_ms = claims.expires_at_unix_ms - 1;
    verifier().verify(&envelope, &expected).unwrap();
    expected.now_unix_ms = claims.expires_at_unix_ms;
    assert_eq!(
        verifier().verify(&envelope, &expected).unwrap_err().code(),
        "AUTHORITY_EXPIRED"
    );
}

#[test]
fn request_domains_and_every_claim_field_are_identity_sensitive() {
    let request = apply_request();
    let original_request_digest = authority_request_digest(&request).unwrap();
    let mut changed_requests = Vec::new();
    let mut changed = request.clone();
    changed.mutation_id = format!("mut_{}", "0".repeat(64));
    changed_requests.push(("mutation_id", changed));
    let mut changed = request.clone();
    changed.repository_id = format!("rep_{}", "0".repeat(64));
    changed_requests.push(("repository_id", changed));
    let mut changed = request.clone();
    changed.workspace_id = format!("wsp_{}", "0".repeat(64));
    changed_requests.push(("workspace_id", changed));
    let mut changed = request.clone();
    changed.workspace_generation += 1;
    changed_requests.push(("workspace_generation", changed));
    let mut changed = request.clone();
    changed.proposal.proposal_id = format!("cnt_{}", "0".repeat(64));
    changed_requests.push(("proposal.proposal_id", changed));
    let mut changed = request.clone();
    changed.proposal.operations[0].content_utf8 = Some("pub fn changed() {}\n".to_owned());
    changed_requests.push(("proposal.operations.content_utf8", changed));
    let mut changed = request.clone();
    changed.proposal.gate_ids = vec![format!("gat_{}", "9".repeat(64))];
    changed_requests.push(("proposal.gate_ids", changed));
    for (field, changed) in changed_requests {
        assert_ne!(
            authority_request_digest(&changed).unwrap(),
            original_request_digest,
            "request field {field}"
        );
    }
    let mut invalid_schema = request;
    invalid_schema.schema_version = "v2".to_owned();
    assert!(authority_request_digest(&invalid_schema).is_err());

    let claims = claims();
    let original = claims.digest().unwrap();
    let value = serde_json::to_value(&claims).unwrap();
    for field in value.as_object().unwrap().keys() {
        let mut changed = value.clone();
        changed.as_object_mut().unwrap().remove(field);
        let bytes = canonical_json(&changed).unwrap();
        let digest = bullet_wire::hash_framed_bytes("authority.claims.v1alpha1", &bytes).unwrap();
        assert_ne!(digest, original, "field {field} was not identity-bound");
    }
}

#[test]
fn malformed_claim_windows_and_keys_are_rejected_before_signing() {
    let mut invalid = claims();
    invalid.expires_at_unix_ms += 1;
    assert_eq!(
        signer()
            .sign_for_request(&invalid, &apply_request())
            .unwrap_err()
            .code(),
        "INVALID_AUTHORITY_WINDOW"
    );
    invalid = claims();
    invalid.attempt_fence = 0;
    assert_eq!(
        signer()
            .sign_for_request(&invalid, &apply_request())
            .unwrap_err()
            .code(),
        "INVALID_AUTHORITY_GENERATION"
    );
    assert!(AuthoritySigningKey::from_bytes("issuer", "key", &[0; 64]).is_err());
    assert!(AuthorityVerificationKey::from_bytes("issuer", "key", &[0; 32]).is_err());
}

#[test]
fn signing_requires_the_exact_validated_request_subject() {
    let claims = claims();
    let request = apply_request();
    let mut changed_claims = Vec::new();

    let mut changed = claims.clone();
    changed.operation = MutationOperation::Checkpoint;
    changed_claims.push(changed);
    let mut changed = claims.clone();
    changed.mutation_id = id("mut_", '0');
    changed_claims.push(changed);
    let mut changed = claims.clone();
    changed.repository_id = id("rep_", '0');
    changed_claims.push(changed);
    let mut changed = claims.clone();
    changed.workspace_id = id("wsp_", '0');
    changed_claims.push(changed);
    let mut changed = claims.clone();
    changed.workspace_generation += 1;
    changed_claims.push(changed);
    let mut changed = claims.clone();
    changed.attempt_id = id("atm_", '0');
    changed_claims.push(changed);
    let mut changed = claims.clone();
    changed.request_digest = digest(99);
    changed_claims.push(changed);

    for changed in changed_claims {
        assert_eq!(
            signer()
                .sign_for_request(&changed, &request)
                .unwrap_err()
                .code(),
            "AUTHORITY_REQUEST_BINDING_MISMATCH"
        );
    }

    let mut invalid_request = request;
    invalid_request.workspace_generation = 0;
    assert_eq!(
        signer()
            .sign_for_request(&claims, &invalid_request)
            .unwrap_err()
            .code(),
        "INVALID_AUTHORITY_REQUEST"
    );
}

#[test]
fn signing_rejects_operation_specific_claim_conflicts() {
    let clone_request = clone_request();
    let mut clone_claims = claims();
    clone_claims.operation = MutationOperation::CloneWorkspace;
    clone_claims.request_digest = authority_request_digest(&clone_request).unwrap();
    clone_claims.scope_grant_digest = clone_request.scope_grant_digest.parse().unwrap();
    clone_claims.scope_revision = clone_request.scope_grant.scope_revision;
    signer()
        .sign_for_request(&clone_claims, &clone_request)
        .unwrap();
    let mut changed = clone_claims.clone();
    changed.scope_grant_digest = digest(99);
    assert_eq!(
        signer()
            .sign_for_request(&changed, &clone_request)
            .unwrap_err()
            .code(),
        "AUTHORITY_REQUEST_BINDING_MISMATCH"
    );
    let mut changed = clone_claims;
    changed.scope_revision += 1;
    assert_eq!(
        signer()
            .sign_for_request(&changed, &clone_request)
            .unwrap_err()
            .code(),
        "AUTHORITY_REQUEST_BINDING_MISMATCH"
    );

    let dispatch_request = dispatch_request();
    let mut dispatch_claims = claims();
    dispatch_claims.operation = MutationOperation::DispatchEffect;
    dispatch_claims.request_digest = authority_request_digest(&dispatch_request).unwrap();
    assert_eq!(
        signer()
            .sign_for_request(&dispatch_claims, &dispatch_request)
            .unwrap_err()
            .code(),
        "INVALID_AUTHORITY_AUDIENCE"
    );
    dispatch_claims.audience = AuthorityAudience::EffectBroker;
    signer()
        .sign_for_request(&dispatch_claims, &dispatch_request)
        .unwrap();
    let mut changed_claims = Vec::new();
    let mut changed = dispatch_claims.clone();
    changed.policy_snapshot_id = id("cnt_", '0');
    changed_claims.push(changed);
    let mut changed = dispatch_claims.clone();
    changed.authority_epoch += 1;
    changed_claims.push(changed);
    let mut changed = dispatch_claims;
    changed.freeze_generation += 1;
    changed_claims.push(changed);
    for changed in changed_claims {
        assert_eq!(
            signer()
                .sign_for_request(&changed, &dispatch_request)
                .unwrap_err()
                .code(),
            "AUTHORITY_REQUEST_BINDING_MISMATCH"
        );
    }
}

#[test]
fn valid_signatures_over_unknown_or_noncanonical_claims_still_fail_closed() {
    let claims = claims();
    let expected = expectation(&claims);
    let mut unknown = serde_json::to_value(&claims).unwrap();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("surprise".to_owned(), serde_json::json!(true));
    let envelope = sign_raw_payload(&canonical_json(&unknown).unwrap(), "authority-signing");
    assert_eq!(
        verifier().verify(&envelope, &expected).unwrap_err().code(),
        "DOCUMENT_SCHEMA_INVALID"
    );

    let pretty = serde_json::to_vec_pretty(&claims).unwrap();
    let envelope = sign_raw_payload(&pretty, "authority-signing");
    assert_eq!(
        verifier().verify(&envelope, &expected).unwrap_err().code(),
        "NON_CANONICAL_JSON"
    );

    let payload = canonical_json(&claims).unwrap();
    let envelope = sign_raw_payload(&payload, "release-signing");
    assert_eq!(
        verifier().verify(&envelope, &expected).unwrap_err().code(),
        "INVALID_AUTHORITY_SIGNATURE"
    );
}

#[test]
fn committed_authority_golden_is_independently_verifiable() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf();
    let bytes = fs::read(root.join("fixtures/canonical/authority-golden.json")).unwrap();
    let value = bullet_wire::decode_canonical_value(&bytes).unwrap();
    let expected_hash = "4ff1ce8a4ba7a37ae705a8d2459e5a9d900abe55610f6d6984fe514cd37860df";
    assert_eq!(
        hash_canonical("authority.golden.v1alpha1", &value)
            .unwrap()
            .to_string(),
        expected_hash
    );
    assert_eq!(bullet_wire::v1alpha1::AUTHORITY_GOLDEN_HASH, expected_hash);

    let claims_json = value["claims_canonical_json"].as_str().unwrap();
    let golden_claims = decode_canonical::<AuthorityClaims>(claims_json.as_bytes()).unwrap();
    let envelope =
        serde_json::from_value::<SignedAuthorityEnvelope>(value["envelope"].clone()).unwrap();
    let public_key = hex::decode(value["public_key_hex"].as_str().unwrap()).unwrap();
    let verifier = AuthorityVerificationKey::from_bytes(
        "bullet-kernel-local",
        "authority-test-1",
        &public_key,
    )
    .unwrap();
    assert_eq!(
        verifier
            .verify(&envelope, &expectation(&golden_claims))
            .unwrap(),
        golden_claims
    );

    let permit_claims_json = value["mutation_permit_claims_canonical_json"]
        .as_str()
        .unwrap();
    let permit_claims =
        decode_canonical::<MutationPermitClaims>(permit_claims_json.as_bytes()).unwrap();
    let permit =
        serde_json::from_value::<SignedMutationPermit>(value["mutation_permit"].clone()).unwrap();
    let request = serde_json::from_value::<bullet_wire::v1alpha1::ApplyPatchRequestV1>(
        value["request"].clone(),
    )
    .unwrap();
    let permit_expectation = MutationPermitExpectation {
        audience: permit_claims.audience,
        operation: permit_claims.operation,
        authority_envelope_digest: permit_claims.authority_envelope_digest,
        authority_token_nonce: permit_claims.authority_token_nonce,
        mutation_id: permit_claims.mutation_id.clone(),
        reservation_id: permit_claims.reservation_id.clone(),
        request_digest: permit_claims.request_digest,
        repository_id: permit_claims.repository_id.clone(),
        workspace_id: permit_claims.workspace_id.clone(),
        workspace_generation: request.workspace_generation,
        attempt_id: permit_claims.attempt_id.clone(),
        attempt_fence: permit_claims.attempt_fence,
        authority_epoch: permit_claims.authority_epoch,
        freeze_generation: permit_claims.freeze_generation,
        now_unix_ms: permit_claims.not_before_unix_ms,
    };
    assert_eq!(
        verifier
            .verify_mutation_permit(&permit, &permit_expectation)
            .unwrap(),
        permit_claims
    );
    assert_eq!(
        permit.digest().unwrap().to_string(),
        value["mutation_permit_digest"].as_str().unwrap()
    );

    let settlement_request =
        serde_json::from_value::<MutationSettlementRequest>(value["settlement_request"].clone())
            .unwrap();
    settlement_request.validate_shape().unwrap();
    let settlement_result =
        serde_json::from_value::<MutationSettlementResult>(value["settlement_result"].clone())
            .unwrap();
    settlement_result.validate().unwrap();
    serde_json::from_value::<bullet_wire::v1alpha1::MutationSettlementRequestV1>(
        value["settlement_request"].clone(),
    )
    .unwrap();
    serde_json::from_value::<bullet_wire::v1alpha1::MutationSettlementResultV1>(
        value["settlement_result"].clone(),
    )
    .unwrap();
}
