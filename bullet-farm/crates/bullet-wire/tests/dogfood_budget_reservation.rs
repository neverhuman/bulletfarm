use std::collections::BTreeSet;

use bullet_wire::{
    AttemptId, Blake3Digest, CheckpointId, CommandId, CredentialProjectionProfileId,
    DOGFOOD_BUDGET_RESERVATION_DIGEST_DOMAIN, DOGFOOD_SCHEMA_VERSION, DogfoodBudgetReservationId,
    DogfoodBudgetReservationV1, DogfoodExecutionSubjectV1, DogfoodPolicySubjectV1,
    DogfoodProviderProtocolV1, DogfoodProviderSubjectV1, DogfoodReadOnlyIntentV1,
    DogfoodRepositorySubjectV1, DogfoodRunId, DogfoodRunSubjectV1, GateId, GitOid, GraphRevisionId,
    LaunchProvider, MAX_DOGFOOD_BUDGET_CONSUME_WINDOW_MS, MissionId,
    PROVIDER_ENROLLMENT_CLAIMS_DOMAIN, PROVIDER_ENROLLMENT_SIGNING_PURPOSE, PrincipalId,
    ProviderCredentialProjectionId, ProviderEnrollmentClaimsV2, ProviderEnrollmentId,
    ProviderProfileId, RepositoryContextSnapshotId, RepositoryId, RunnerId, RuntimePassportId,
    VariantId, WireError, WorkPackageId, canonical_json, decode_dogfood_budget_reservation,
    verify_dogfood_budget_binding,
};
use serde_json::{Value, json};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

fn digest(seed: u8) -> Blake3Digest {
    Blake3Digest::from_bytes([seed; 32])
}

fn oid(seed: u8) -> GitOid {
    GitOid::Sha256(format!("{seed:02x}").repeat(32))
}

fn enrollment(provider: LaunchProvider) -> ProviderEnrollmentClaimsV2 {
    ProviderEnrollmentClaimsV2 {
        schema_version: DOGFOOD_SCHEMA_VERSION.into(),
        issuer: "operator.example".into(),
        key_id: "dogfood-enrollment-alpha".into(),
        signing_purpose: PROVIDER_ENROLLMENT_SIGNING_PURPOSE.into(),
        claims_domain: PROVIDER_ENROLLMENT_CLAIMS_DOMAIN.into(),
        provider,
        protocol: DogfoodProviderProtocolV1::required_for(provider),
        runtime_passport_id: RuntimePassportId::from_digest(digest(1)),
        provider_profile_id: ProviderProfileId::from_digest(digest(2)),
        service_identity_id: PrincipalId::from_digest(digest(3)),
        credential_projection_profile_id: CredentialProjectionProfileId::from_digest(digest(4)),
        runtime_version: "v1.2.3".into(),
        enrollment_generation: 2,
        activates_at_unix_ms: 1_000,
        expires_at_unix_ms: 50_000,
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
        schema_version: DOGFOOD_SCHEMA_VERSION.into(),
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
            gate_ids: vec![GateId::from_digest(digest(27))],
            prompt_digest: digest(28),
            policy: DogfoodPolicySubjectV1 {
                policy_snapshot_digest: enrollment.policy_snapshot_digest,
                policy_generation: enrollment.policy_generation,
                dogfood_binding_digest: digest(29),
                tool_policy_digest: enrollment.tool_policy_digest,
                egress_policy_digest: enrollment.egress_policy_digest,
                containment_policy_digest: digest(30),
            },
            budget_reservation_id: DogfoodBudgetReservationId::from_digest(digest(31)),
            deadline_unix_ms: 40_000,
            output_schema_digest: digest(32),
        },
    }
}

fn reservation(
    intent: &DogfoodReadOnlyIntentV1,
    enrollment: &ProviderEnrollmentClaimsV2,
) -> DogfoodBudgetReservationV1 {
    DogfoodBudgetReservationV1 {
        schema_version: DOGFOOD_SCHEMA_VERSION.into(),
        reservation_id: intent.subject.budget_reservation_id.clone(),
        run_id: intent.subject.execution.run_id.clone(),
        provider: enrollment.provider,
        provider_profile_id: enrollment.provider_profile_id.clone(),
        provider_enrollment_id: enrollment.enrollment_id().unwrap(),
        budget_policy_digest: enrollment.budget_policy_digest,
        reserved_at_unix_ms: 2_000,
        consume_before_unix_ms: 3_000,
        reserved_cost_micro_usd: 2_500_000,
        reserved_invocations: 1,
        reserved_wall_time_ms: 900_000,
        reserved_concurrency: 1,
    }
}

fn decode_value(value: &Value) -> Result<DogfoodBudgetReservationV1, WireError> {
    decode_dogfood_budget_reservation(&serde_jcs::to_vec(value).unwrap())
}

fn refusal<T>(result: Result<T, WireError>, expected: &'static str) {
    match result {
        Err(error) => assert_eq!(error.code(), expected, "{error}"),
        Ok(_) => panic!("expected {expected}"),
    }
}

#[test]
fn all_four_providers_round_trip_with_exact_closed_keys() {
    let providers = [
        LaunchProvider::Claude,
        LaunchProvider::Codex,
        LaunchProvider::Cursor,
        LaunchProvider::Agy,
    ];
    let mut digests = BTreeSet::new();
    for provider in providers {
        let enrollment = enrollment(provider);
        let intent = intent(&enrollment);
        let reservation = reservation(&intent, &enrollment);
        verify_dogfood_budget_binding(&reservation, &intent, &enrollment).unwrap();
        let bytes = canonical_json(&reservation).unwrap();
        assert_eq!(
            decode_dogfood_budget_reservation(&bytes).unwrap(),
            reservation
        );
        assert!(digests.insert(reservation.reservation_digest().unwrap()));
        assert!(reservation.reservation_id.as_str().starts_with("dbr_"));
        assert_eq!(
            serde_json::to_value(&reservation)
                .unwrap()
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                "budget_policy_digest".into(),
                "consume_before_unix_ms".into(),
                "provider".into(),
                "provider_enrollment_id".into(),
                "provider_profile_id".into(),
                "reservation_id".into(),
                "reserved_at_unix_ms".into(),
                "reserved_concurrency".into(),
                "reserved_cost_micro_usd".into(),
                "reserved_invocations".into(),
                "reserved_wall_time_ms".into(),
                "run_id".into(),
                "schema_version".into(),
            ])
        );
    }
    assert_eq!(digests.len(), providers.len());
    assert_eq!(
        DOGFOOD_BUDGET_RESERVATION_DIGEST_DOMAIN,
        "dogfood.budget-reservation.v1alpha1"
    );
}

#[test]
fn canonical_decode_refuses_forbidden_controls_and_non_integer_money() {
    let enrollment = enrollment(LaunchProvider::Claude);
    let intent = intent(&enrollment);
    let reservation = reservation(&intent, &enrollment);
    let text = String::from_utf8(canonical_json(&reservation).unwrap()).unwrap();
    let duplicate = text.replacen(
        "\"reserved_invocations\":1",
        "\"reserved_invocations\":1,\"reserved_invocations\":1",
        1,
    );
    assert!(decode_dogfood_budget_reservation(duplicate.as_bytes()).is_err());

    for name in [
        "state",
        "status",
        "outcome",
        "usage",
        "actual_cost",
        "settled_cost",
        "unknown_cost",
        "released_cost",
        "remaining_cost",
        "headroom",
        "reserve_class",
        "priority",
        "emergency",
        "currency",
        "provider_quota",
    ] {
        let mut changed = serde_json::to_value(&reservation).unwrap();
        changed[name] = json!("caller-control");
        assert!(decode_value(&changed).is_err(), "field {name} was accepted");
    }

    let mut changed = serde_json::to_value(&reservation).unwrap();
    changed
        .as_object_mut()
        .unwrap()
        .remove("budget_policy_digest");
    assert!(decode_value(&changed).is_err());
    for amount in [json!(1.5), json!("1.000000"), json!(-1)] {
        let mut changed = serde_json::to_value(&reservation).unwrap();
        changed["reserved_cost_micro_usd"] = amount;
        assert!(decode_value(&changed).is_err());
    }
    let exponent = text.replacen("2500000", "25e5", 1);
    assert!(decode_dogfood_budget_reservation(exponent.as_bytes()).is_err());
    let mut alias = serde_json::to_value(&reservation).unwrap();
    alias["provider"] = json!("antigravity");
    assert!(decode_value(&alias).is_err());
    let mut bad_id = serde_json::to_value(&reservation).unwrap();
    bad_id["reservation_id"] = json!("dbr_00");
    assert!(decode_value(&bad_id).is_err());
}

#[test]
fn numeric_and_time_bounds_refuse_unsafe_or_ambiguous_reservations() {
    let enrollment = enrollment(LaunchProvider::Claude);
    let intent = intent(&enrollment);
    let base = reservation(&intent, &enrollment);
    let invalid: [fn(&mut DogfoodBudgetReservationV1); 10] = [
        |v| v.reserved_at_unix_ms = v.consume_before_unix_ms,
        |v| v.consume_before_unix_ms = v.reserved_at_unix_ms - 1,
        |v| v.consume_before_unix_ms = v.reserved_at_unix_ms + 15_001,
        |v| v.reserved_cost_micro_usd = 0,
        |v| v.reserved_wall_time_ms = 0,
        |v| v.reserved_invocations = 0,
        |v| v.reserved_invocations = 2,
        |v| v.reserved_concurrency = 0,
        |v| v.reserved_concurrency = 2,
        |v| v.reserved_cost_micro_usd = MAX_SAFE_INTEGER + 1,
    ];
    for mutate in invalid {
        let mut changed = base.clone();
        mutate(&mut changed);
        refusal(changed.validate(), "DOGFOOD_BUDGET_RESERVATION_INVALID");
    }

    let mut boundary = base.clone();
    boundary.consume_before_unix_ms =
        boundary.reserved_at_unix_ms + MAX_DOGFOOD_BUDGET_CONSUME_WINDOW_MS;
    boundary.validate().unwrap();
    boundary.consume_before_unix_ms = MAX_SAFE_INTEGER;
    boundary.reserved_at_unix_ms = MAX_SAFE_INTEGER - 1;
    refusal(boundary.validate(), "DOGFOOD_BUDGET_RESERVATION_INVALID");
}

#[test]
fn digest_and_subject_binding_cover_every_mutable_identity() {
    let enrollment = enrollment(LaunchProvider::Claude);
    let intent = intent(&enrollment);
    let base = reservation(&intent, &enrollment);
    let expected = base.reservation_digest().unwrap();
    let valid_mutations: [fn(&mut DogfoodBudgetReservationV1); 9] = [
        |v| v.reservation_id = DogfoodBudgetReservationId::from_digest(digest(40)),
        |v| v.run_id = DogfoodRunId::from_digest(digest(41)),
        |v| v.provider = LaunchProvider::Codex,
        |v| v.provider_profile_id = ProviderProfileId::from_digest(digest(42)),
        |v| v.provider_enrollment_id = ProviderEnrollmentId::from_digest(digest(43)),
        |v| v.budget_policy_digest = digest(44),
        |v| v.reserved_at_unix_ms += 1,
        |v| v.consume_before_unix_ms += 1,
        |v| v.reserved_cost_micro_usd += 1,
    ];
    for mutate in valid_mutations {
        let mut changed = base.clone();
        mutate(&mut changed);
        assert_ne!(changed.reservation_digest().unwrap(), expected);
    }
    let mut changed = base.clone();
    changed.reserved_wall_time_ms += 1;
    assert_ne!(changed.reservation_digest().unwrap(), expected);

    verify_dogfood_budget_binding(&base, &intent, &enrollment).unwrap();
    let mut wrong_id = base.clone();
    wrong_id.reservation_id = DogfoodBudgetReservationId::from_digest(digest(45));
    refusal(
        verify_dogfood_budget_binding(&wrong_id, &intent, &enrollment),
        "DOGFOOD_BUDGET_RESERVATION_ID_MISMATCH",
    );
    for mutate in valid_mutations.into_iter().skip(1).take(5) {
        let mut changed = base.clone();
        mutate(&mut changed);
        refusal(
            verify_dogfood_budget_binding(&changed, &intent, &enrollment),
            "DOGFOOD_BUDGET_SUBJECT_MISMATCH",
        );
    }
}
