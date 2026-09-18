use serde_json::{Value, json};

use super::*;
use crate::{
    CredentialProjectionProfileId, LaunchGrantClaims, canonical_json, decode_canonical,
    decode_canonical_value,
};

fn digest(seed: u8) -> Blake3Digest {
    Blake3Digest::from_bytes([seed; 32])
}

fn oid(seed: u8) -> GitOid {
    GitOid::Sha256(format!("{seed:02x}").repeat(32))
}

fn providers() -> [(LaunchProvider, DogfoodProviderProtocolV1); 4] {
    use DogfoodProviderProtocolV1 as P;
    use LaunchProvider as L;
    [
        (L::Claude, P::ClaudeStreamJson),
        (L::Codex, P::CodexAppServerJsonl),
        (L::Cursor, P::CursorAcp),
        (L::Agy, P::AntigravityHeadlessStructured),
    ]
}

fn enrollment(
    provider: LaunchProvider,
    protocol: DogfoodProviderProtocolV1,
) -> ProviderEnrollmentClaimsV2 {
    ProviderEnrollmentClaimsV2 {
        schema_version: DOGFOOD_SCHEMA_VERSION.to_owned(),
        issuer: "operator.example".to_owned(),
        key_id: "dogfood-enrollment-alpha".to_owned(),
        signing_purpose: PROVIDER_ENROLLMENT_SIGNING_PURPOSE.to_owned(),
        claims_domain: PROVIDER_ENROLLMENT_CLAIMS_DOMAIN.to_owned(),
        provider,
        protocol,
        runtime_passport_id: RuntimePassportId::from_digest(digest(1)),
        provider_profile_id: ProviderProfileId::from_digest(digest(2)),
        service_identity_id: PrincipalId::from_digest(digest(3)),
        credential_projection_profile_id: CredentialProjectionProfileId::from_digest(digest(4)),
        runtime_version: "v1.2.3".to_owned(),
        enrollment_generation: 2,
        activates_at_unix_ms: 1_000,
        expires_at_unix_ms: 5_000,
        revoked_at_unix_ms: None,
        egress_policy_digest: digest(5),
        tool_policy_digest: digest(6),
        budget_policy_digest: digest(7),
        endpoint_observation_digest: digest(8),
        version_observation_digest: digest(9),
        profile_observation_digest: digest(10),
        policy_snapshot_digest: digest(11),
        policy_generation: 2,
    }
}

fn intent(enrollment: &ProviderEnrollmentClaimsV2) -> DogfoodReadOnlyIntentV1 {
    DogfoodReadOnlyIntentV1 {
        schema_version: DOGFOOD_SCHEMA_VERSION.to_owned(),
        request_digest: digest(12),
        subject: DogfoodRunSubjectV1 {
            execution: DogfoodExecutionSubjectV1 {
                command_id: CommandId::from_digest(digest(13)),
                run_id: DogfoodRunId::from_digest(digest(14)),
                mission_id: MissionId::from_digest(digest(15)),
                repository_id: RepositoryId::from_digest(digest(16)),
                graph_revision_id: GraphRevisionId::from_digest(digest(17)),
                work_package_id: WorkPackageId::from_digest(digest(18)),
                variant_id: VariantId::from_digest(digest(19)),
                attempt_id: AttemptId::from_digest(digest(20)),
                attempt_fence: 3,
                runner_id: RunnerId::from_digest(digest(21)),
                runner_epoch: 4,
                authority_epoch: 5,
                freeze_generation: 6,
            },
            provider: DogfoodProviderSubjectV1 {
                provider: enrollment.provider,
                protocol: enrollment.protocol,
                provider_profile_id: enrollment.provider_profile_id.clone(),
                runtime_passport_id: enrollment.runtime_passport_id.clone(),
                provider_enrollment_id: enrollment.enrollment_id().unwrap(),
                credential_projection_id: ProviderCredentialProjectionId::from_digest(digest(22)),
            },
            repository: DogfoodRepositorySubjectV1 {
                context_snapshot_id: RepositoryContextSnapshotId::from_digest(digest(23)),
                head_oid: oid(24),
                tree_oid: oid(25),
                checkpoint_id: CheckpointId::from_digest(digest(26)),
            },
            gate_ids: vec![
                GateId::from_digest(digest(27)),
                GateId::from_digest(digest(28)),
            ],
            prompt_digest: digest(29),
            policy: DogfoodPolicySubjectV1 {
                policy_snapshot_digest: enrollment.policy_snapshot_digest,
                policy_generation: enrollment.policy_generation,
                dogfood_binding_digest: digest(30),
                tool_policy_digest: enrollment.tool_policy_digest,
                egress_policy_digest: enrollment.egress_policy_digest,
                containment_policy_digest: digest(31),
            },
            budget_reservation_id: DogfoodBudgetReservationId::from_digest(digest(32)),
            deadline_unix_ms: 4_000,
            output_schema_digest: digest(33),
        },
    }
}

fn grant(intent: &DogfoodReadOnlyIntentV1) -> DogfoodLaunchGrantClaimsV1 {
    DogfoodLaunchGrantClaimsV1 {
        schema_version: DOGFOOD_SCHEMA_VERSION.to_owned(),
        audience: DogfoodAudienceV1::DogfoodRunner,
        operation: DogfoodOperationV1::ReadOnlyPropose,
        issuer: "kernel.example".to_owned(),
        key_id: "dogfood-launch-alpha".to_owned(),
        signing_purpose: DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE.to_owned(),
        claims_domain: DOGFOOD_LAUNCH_GRANT_CLAIMS_DOMAIN.to_owned(),
        issued_at_unix_ms: 1_900,
        not_before_unix_ms: 2_000,
        expires_at_unix_ms: 3_000,
        grant_nonce: digest(34),
        request_digest: intent.request_digest,
        intent_id: intent.intent_id().unwrap(),
        subject: intent.subject.clone(),
    }
}

fn fixture() -> (
    ProviderEnrollmentClaimsV2,
    DogfoodReadOnlyIntentV1,
    DogfoodLaunchGrantClaimsV1,
) {
    let enrollment = enrollment(
        LaunchProvider::Claude,
        DogfoodProviderProtocolV1::ClaudeStreamJson,
    );
    let intent = intent(&enrollment);
    let grant = grant(&intent);
    (enrollment, intent, grant)
}

fn refusal<T>(result: Result<T, WireError>, expected: &'static str) {
    match result {
        Err(error) => assert_eq!(error.code(), expected, "{error}"),
        Ok(_) => panic!("expected {expected}"),
    }
}

fn canonical_value(value: &Value) -> Vec<u8> {
    canonical_json(value).unwrap()
}

fn assert_id(first: impl ToString, second: impl ToString, prefix: &str) {
    let first = first.to_string();
    assert_eq!(first, second.to_string());
    assert_eq!(first.len(), 68);
    assert!(first.starts_with(prefix));
}

fn rebind(
    enrollment: &ProviderEnrollmentClaimsV2,
    intent: &mut DogfoodReadOnlyIntentV1,
    grant: &mut DogfoodLaunchGrantClaimsV1,
) {
    intent.subject.provider.provider_enrollment_id = enrollment.enrollment_id().unwrap();
    grant.intent_id = intent.intent_id().unwrap();
    grant.subject = intent.subject.clone();
}

#[test]
fn all_four_provider_subjects_round_trip_and_bind() {
    for (provider, protocol) in providers() {
        let enrollment = enrollment(provider, protocol);
        let intent = intent(&enrollment);
        let grant = grant(&intent);
        let intent_bytes = canonical_json(&intent).unwrap();
        let grant_bytes = canonical_json(&grant).unwrap();
        let enrollment_bytes = canonical_json(&enrollment).unwrap();

        assert_eq!(
            decode_dogfood_read_only_intent(&intent_bytes).unwrap(),
            intent
        );
        assert_eq!(
            decode_dogfood_launch_grant_claims(&grant_bytes).unwrap(),
            grant
        );
        assert_eq!(
            decode_provider_enrollment_claims(&enrollment_bytes).unwrap(),
            enrollment
        );
        verify_dogfood_subjects(&grant, &intent, &enrollment).unwrap();
        assert_id(
            intent.intent_id().unwrap(),
            intent.intent_id().unwrap(),
            "dfi_",
        );
        assert_id(grant.grant_id().unwrap(), grant.grant_id().unwrap(), "dfg_");
        assert_id(
            enrollment.enrollment_id().unwrap(),
            enrollment.enrollment_id().unwrap(),
            "pen_",
        );
        let subject = &intent.subject;
        for (actual, prefix) in [
            (subject.execution.run_id.as_str(), "dfr_"),
            (subject.provider.runtime_passport_id.as_str(), "rtp_"),
            (subject.provider.credential_projection_id.as_str(), "pcp_"),
            (subject.repository.context_snapshot_id.as_str(), "rcs_"),
            (subject.budget_reservation_id.as_str(), "dbr_"),
            (enrollment.credential_projection_profile_id.as_str(), "cpp_"),
        ] {
            assert!(actual.starts_with(prefix));
        }
        let intent_wire = String::from_utf8(intent_bytes).unwrap();
        assert!(intent_wire.contains(&format!("\"provider\":\"{}\"", provider.as_str())));
    }
}

#[test]
fn canonical_and_recursive_schema_hostiles_refuse() {
    let (_, intent, _) = fixture();
    let bytes = canonical_json(&intent).unwrap();
    let mut top = serde_json::to_value(&intent).unwrap();
    top.as_object_mut()
        .unwrap()
        .insert("a_unknown".to_owned(), json!(true));
    refusal(
        decode_dogfood_read_only_intent(&canonical_value(&top)),
        "DOCUMENT_SCHEMA_INVALID",
    );

    let mut nested = serde_json::to_value(&intent).unwrap();
    nested["subject"]["provider"]
        .as_object_mut()
        .unwrap()
        .insert("b_unknown".to_owned(), json!(true));
    refusal(
        decode_dogfood_read_only_intent(&canonical_value(&nested)),
        "DOCUMENT_SCHEMA_INVALID",
    );
    refusal(
        decode_dogfood_read_only_intent(
            br#"{"schema_version":"v1alpha1","schema_version":"v1alpha1"}"#,
        ),
        "DUPLICATE_JSON_KEY",
    );
    let mut spaced = vec![b' '];
    spaced.extend_from_slice(&bytes);
    refusal(
        decode_dogfood_read_only_intent(&spaced),
        "NON_CANONICAL_JSON",
    );
    refusal(
        decode_dogfood_read_only_intent(br#"{"x":9007199254740992}"#),
        "UNSAFE_JSON_INTEGER",
    );

    let mut missing = serde_json::to_value(&intent).unwrap();
    missing["subject"]
        .as_object_mut()
        .unwrap()
        .remove("output_schema_digest");
    refusal(
        decode_dogfood_read_only_intent(&canonical_value(&missing)),
        "DOCUMENT_SCHEMA_INVALID",
    );
}

#[test]
fn provider_alias_protocol_and_typed_id_substitution_refuse() {
    let (_, intent, _) = fixture();
    let mut alias = serde_json::to_value(&intent).unwrap();
    alias["subject"]["provider"]["provider"] = json!("antigravity");
    refusal(
        decode_dogfood_read_only_intent(&canonical_value(&alias)),
        "DOCUMENT_SCHEMA_INVALID",
    );
    let mut legacy = serde_json::to_value(&intent).unwrap();
    legacy["subject"]["provider"]["protocol"] = json!("codex_exec_json");
    refusal(
        decode_dogfood_read_only_intent(&canonical_value(&legacy)),
        "DOCUMENT_SCHEMA_INVALID",
    );
    let mut malformed_id = serde_json::to_value(&intent).unwrap();
    malformed_id["subject"]["repository"]["context_snapshot_id"] = json!("rcs_00");
    refusal(
        decode_dogfood_read_only_intent(&canonical_value(&malformed_id)),
        "DOCUMENT_SCHEMA_INVALID",
    );
    let mut wrong_pair = intent;
    wrong_pair.subject.provider.protocol = DogfoodProviderProtocolV1::CursorAcp;
    refusal(
        decode_dogfood_read_only_intent(&canonical_json(&wrong_pair).unwrap()),
        "DOGFOOD_PROVIDER_PROTOCOL_MISMATCH",
    );
}

#[test]
fn shape_gate_time_and_deadline_bounds_refuse() {
    let (mut enrollment, mut intent, mut grant) = fixture();
    intent.subject.gate_ids.clear();
    refusal(intent.validate(), "DOGFOOD_INTENT_INVALID");
    let (_, mut intent, _) = fixture();
    intent.subject.gate_ids[1] = intent.subject.gate_ids[0].clone();
    refusal(intent.validate(), "DOGFOOD_INTENT_INVALID");
    let (_, mut intent, _) = fixture();
    intent.subject.execution.attempt_fence = 0;
    refusal(intent.validate(), "DOGFOOD_INTENT_INVALID");
    let (_, mut intent, _) = fixture();
    intent.subject.execution.runner_epoch = MAX_SAFE_INTEGER + 1;
    refusal(intent.validate(), "DOGFOOD_INTENT_INVALID");

    grant.issued_at_unix_ms = grant.not_before_unix_ms + 1;
    refusal(grant.validate(), "DOGFOOD_GRANT_INVALID");
    let (_, _, mut grant) = fixture();
    grant.expires_at_unix_ms = grant.not_before_unix_ms + MAX_DOGFOOD_GRANT_TTL_MS + 1;
    grant.subject.deadline_unix_ms = grant.expires_at_unix_ms;
    refusal(grant.validate(), "DOGFOOD_GRANT_INVALID");
    let (_, _, mut grant) = fixture();
    grant.subject.deadline_unix_ms = grant.expires_at_unix_ms - 1;
    refusal(grant.validate(), "DOGFOOD_GRANT_INVALID");

    enrollment.enrollment_generation = 0;
    refusal(enrollment.validate(), "PROVIDER_ENROLLMENT_INVALID");
    let (mut enrollment, _, _) = fixture();
    enrollment.revoked_at_unix_ms = Some(enrollment.activates_at_unix_ms - 1);
    refusal(enrollment.validate(), "PROVIDER_ENROLLMENT_INVALID");
}

#[test]
fn every_run_subject_identity_is_exactly_bound() {
    let (enrollment, intent, grant) = fixture();
    let mutations: &[fn(&mut DogfoodRunSubjectV1)] = &[
        |s| s.execution.command_id = CommandId::from_digest(digest(90)),
        |s| s.execution.run_id = DogfoodRunId::from_digest(digest(91)),
        |s| s.execution.mission_id = MissionId::from_digest(digest(92)),
        |s| s.execution.repository_id = RepositoryId::from_digest(digest(93)),
        |s| s.execution.graph_revision_id = GraphRevisionId::from_digest(digest(94)),
        |s| s.execution.work_package_id = WorkPackageId::from_digest(digest(95)),
        |s| s.execution.variant_id = VariantId::from_digest(digest(96)),
        |s| s.execution.attempt_id = AttemptId::from_digest(digest(97)),
        |s| s.execution.attempt_fence += 1,
        |s| s.execution.runner_id = RunnerId::from_digest(digest(98)),
        |s| s.execution.runner_epoch += 1,
        |s| s.execution.authority_epoch += 1,
        |s| s.execution.freeze_generation += 1,
        |s| {
            s.provider.provider = LaunchProvider::Codex;
            s.provider.protocol = DogfoodProviderProtocolV1::CodexAppServerJsonl;
        },
        |s| s.provider.provider_profile_id = ProviderProfileId::from_digest(digest(99)),
        |s| s.provider.runtime_passport_id = RuntimePassportId::from_digest(digest(100)),
        |s| s.provider.provider_enrollment_id = ProviderEnrollmentId::from_digest(digest(101)),
        |s| {
            s.provider.credential_projection_id =
                ProviderCredentialProjectionId::from_digest(digest(102));
        },
        |s| {
            s.repository.context_snapshot_id = RepositoryContextSnapshotId::from_digest(digest(103))
        },
        |s| s.repository.head_oid = oid(104),
        |s| s.repository.tree_oid = oid(105),
        |s| s.repository.checkpoint_id = CheckpointId::from_digest(digest(106)),
        |s| s.gate_ids = vec![GateId::from_digest(digest(107))],
        |s| s.prompt_digest = digest(108),
        |s| s.policy.policy_snapshot_digest = digest(109),
        |s| s.policy.policy_generation += 1,
        |s| s.policy.dogfood_binding_digest = digest(110),
        |s| s.policy.tool_policy_digest = digest(111),
        |s| s.policy.egress_policy_digest = digest(112),
        |s| s.policy.containment_policy_digest = digest(113),
        |s| s.budget_reservation_id = DogfoodBudgetReservationId::from_digest(digest(114)),
        |s| s.deadline_unix_ms += 1,
        |s| s.output_schema_digest = digest(115),
    ];
    for mutate in mutations {
        let mut changed = grant.clone();
        mutate(&mut changed.subject);
        refusal(
            verify_dogfood_subjects(&changed, &intent, &enrollment),
            "DOGFOOD_GRANT_SUBJECT_MISMATCH",
        );
    }
    let mut changed = grant.clone();
    changed.request_digest = digest(116);
    refusal(
        verify_dogfood_subjects(&changed, &intent, &enrollment),
        "DOGFOOD_GRANT_SUBJECT_MISMATCH",
    );
}

#[test]
fn enrollment_identity_and_full_grant_window_are_bound() {
    let (enrollment, intent, grant) = fixture();
    let mutations: &[fn(&mut ProviderEnrollmentClaimsV2)] = &[
        |e| e.issuer = "other.operator".to_owned(),
        |e| e.key_id = "other-key".to_owned(),
        |e| {
            e.provider = LaunchProvider::Codex;
            e.protocol = DogfoodProviderProtocolV1::CodexAppServerJsonl;
        },
        |e| e.runtime_passport_id = RuntimePassportId::from_digest(digest(120)),
        |e| e.provider_profile_id = ProviderProfileId::from_digest(digest(121)),
        |e| e.service_identity_id = PrincipalId::from_digest(digest(122)),
        |e| {
            e.credential_projection_profile_id =
                CredentialProjectionProfileId::from_digest(digest(123));
        },
        |e| e.runtime_version = "v1.2.4".to_owned(),
        |e| e.enrollment_generation += 1,
        |e| e.activates_at_unix_ms += 1,
        |e| e.expires_at_unix_ms -= 1,
        |e| e.revoked_at_unix_ms = Some(4_000),
        |e| e.egress_policy_digest = digest(124),
        |e| e.tool_policy_digest = digest(125),
        |e| e.budget_policy_digest = digest(126),
        |e| e.endpoint_observation_digest = digest(127),
        |e| e.version_observation_digest = digest(128),
        |e| e.profile_observation_digest = digest(129),
        |e| e.policy_snapshot_digest = digest(130),
        |e| e.policy_generation += 1,
    ];
    for mutate in mutations {
        let mut changed = enrollment.clone();
        mutate(&mut changed);
        refusal(
            verify_dogfood_subjects(&grant, &intent, &changed),
            "PROVIDER_ENROLLMENT_SUBJECT_MISMATCH",
        );
    }

    for mutate in [
        (|e: &mut ProviderEnrollmentClaimsV2| e.activates_at_unix_ms = 2_001)
            as fn(&mut ProviderEnrollmentClaimsV2),
        |e| e.expires_at_unix_ms = 2_999,
        |e| e.revoked_at_unix_ms = Some(3_000),
    ] {
        let mut changed = enrollment.clone();
        let mut rebound_intent = intent.clone();
        let mut rebound_grant = grant.clone();
        mutate(&mut changed);
        rebind(&changed, &mut rebound_intent, &mut rebound_grant);
        refusal(
            verify_dogfood_subjects(&rebound_grant, &rebound_intent, &changed),
            "PROVIDER_ENROLLMENT_SUBJECT_MISMATCH",
        );
    }
    let mut boundary = enrollment;
    let mut boundary_intent = intent;
    let mut boundary_grant = grant;
    boundary.activates_at_unix_ms = boundary_grant.not_before_unix_ms;
    boundary.expires_at_unix_ms = boundary_grant.expires_at_unix_ms;
    rebind(&boundary, &mut boundary_intent, &mut boundary_grant);
    verify_dogfood_subjects(&boundary_grant, &boundary_intent, &boundary).unwrap();
}

#[test]
fn live_shapes_and_authority_outputs_are_absent() {
    let (_, intent, grant) = fixture();
    let golden = decode_canonical_value(include_bytes!(
        "../../../../fixtures/canonical/launch-grant-golden.json"
    ))
    .unwrap();
    let live_claims = golden["claims_canonical_json"].as_str().unwrap().as_bytes();
    refusal(
        decode_dogfood_launch_grant_claims(live_claims),
        "DOCUMENT_SCHEMA_INVALID",
    );
    refusal(
        decode_canonical::<LaunchGrantClaims>(&canonical_json(&grant).unwrap()),
        "DOCUMENT_SCHEMA_INVALID",
    );

    let wire = String::from_utf8(canonical_json(&(intent, grant)).unwrap()).unwrap();
    for forbidden in [
        "\"paseto\":",
        "\"outcome\":",
        "\"release_eligibility\":",
        "\"executable_path\":",
        "\"credential_bytes\":",
        "\"signature\":",
        "\"eligibility\":",
        "\"filesystem_path\":",
        "\"provider_spawn\":",
        "\"requested_runner_id\":",
        "\"requested_outcome\":",
    ] {
        assert!(
            !wire.contains(forbidden),
            "forbidden authority/output field {forbidden}"
        );
    }
}
