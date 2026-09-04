use bullet_wire::{
    AttemptId, Blake3Digest, CheckpointId, CommandId, CredentialProjectionProfileId,
    DOGFOOD_LAUNCH_GRANT_CLAIMS_DOMAIN, DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE,
    DOGFOOD_SCHEMA_VERSION, DogfoodAudienceV1, DogfoodBudgetReservationId,
    DogfoodExecutionSubjectV1, DogfoodLaunchGrantClaimsV1, DogfoodOperationV1,
    DogfoodPolicySubjectV1, DogfoodProviderProtocolV1, DogfoodProviderSubjectV1,
    DogfoodReadOnlyIntentV1, DogfoodRepositorySubjectV1, DogfoodRunId, DogfoodRunSubjectV1, GateId,
    GitOid, GraphRevisionId, LaunchProvider, MissionId, PROVIDER_ENROLLMENT_CLAIMS_DOMAIN,
    PROVIDER_ENROLLMENT_SIGNING_PURPOSE, PrincipalId, ProviderCredentialProjectionId,
    ProviderEnrollmentClaimsV2, ProviderProfileId, ProviderRuntimePassportV1,
    RepositoryContextSnapshotId, RepositoryId, RunnerId, RuntimeExecutionV1, RuntimeFileRoleV1,
    RuntimeFileV1, RuntimeLoaderV1, VariantId, WireError, WorkPackageId,
    verify_dogfood_runtime_binding,
};

fn digest(seed: u8) -> Blake3Digest {
    Blake3Digest::from_bytes([seed; 32])
}

fn oid(seed: u8) -> GitOid {
    GitOid::Sha256(format!("{seed:02x}").repeat(32))
}

fn version(provider: LaunchProvider) -> &'static str {
    match provider {
        LaunchProvider::Claude => "2.1.251",
        LaunchProvider::Codex => "0.150.1",
        LaunchProvider::Cursor => "2026.08.11",
        LaunchProvider::Agy => "1.1.19",
    }
}

fn entrypoint(provider: LaunchProvider) -> &'static str {
    match provider {
        LaunchProvider::Claude => "bin/claude",
        LaunchProvider::Codex => "bin/codex",
        LaunchProvider::Cursor => "bin/cursor-agent",
        LaunchProvider::Agy => "bin/agy",
    }
}

fn passport(provider: LaunchProvider) -> ProviderRuntimePassportV1 {
    let path = entrypoint(provider);
    let version = version(provider);
    ProviderRuntimePassportV1 {
        schema_version: 1,
        provider,
        protocol: DogfoodProviderProtocolV1::required_for(provider),
        version: version.into(),
        deployment_root: format!("/usr/lib/bullet/providers/{}/{version}", provider.as_str()),
        entrypoint: path.into(),
        execution: RuntimeExecutionV1::Native {
            loader: RuntimeLoaderV1::Static,
        },
        files: vec![RuntimeFileV1 {
            path: path.into(),
            role: RuntimeFileRoleV1::Entrypoint,
            mode: 0o555,
            size: 1,
            blake3: "11".repeat(32),
        }],
        aggregate_file_count: 1,
        aggregate_size_bytes: 1,
    }
}

fn enrollment(passport: &ProviderRuntimePassportV1) -> ProviderEnrollmentClaimsV2 {
    ProviderEnrollmentClaimsV2 {
        schema_version: DOGFOOD_SCHEMA_VERSION.into(),
        issuer: "operator.example".into(),
        key_id: "dogfood-enrollment-alpha".into(),
        signing_purpose: PROVIDER_ENROLLMENT_SIGNING_PURPOSE.into(),
        claims_domain: PROVIDER_ENROLLMENT_CLAIMS_DOMAIN.into(),
        provider: passport.provider,
        protocol: passport.protocol,
        runtime_passport_id: passport.passport_id().unwrap(),
        provider_profile_id: ProviderProfileId::from_digest(digest(2)),
        service_identity_id: PrincipalId::from_digest(digest(3)),
        credential_projection_profile_id: CredentialProjectionProfileId::from_digest(digest(4)),
        runtime_version: passport.version.clone(),
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
        schema_version: DOGFOOD_SCHEMA_VERSION.into(),
        audience: DogfoodAudienceV1::DogfoodRunner,
        operation: DogfoodOperationV1::ReadOnlyPropose,
        issuer: "kernel.example".into(),
        key_id: "dogfood-launch-alpha".into(),
        signing_purpose: DOGFOOD_LAUNCH_GRANT_SIGNING_PURPOSE.into(),
        claims_domain: DOGFOOD_LAUNCH_GRANT_CLAIMS_DOMAIN.into(),
        issued_at_unix_ms: 1_900,
        not_before_unix_ms: 2_000,
        expires_at_unix_ms: 3_000,
        grant_nonce: digest(34),
        request_digest: intent.request_digest,
        intent_id: intent.intent_id().unwrap(),
        subject: intent.subject.clone(),
    }
}

fn fixture(
    provider: LaunchProvider,
) -> (
    ProviderRuntimePassportV1,
    ProviderEnrollmentClaimsV2,
    DogfoodReadOnlyIntentV1,
    DogfoodLaunchGrantClaimsV1,
) {
    let passport = passport(provider);
    let enrollment = enrollment(&passport);
    let intent = intent(&enrollment);
    let grant = grant(&intent);
    (passport, enrollment, intent, grant)
}

fn rebind(
    enrollment: &ProviderEnrollmentClaimsV2,
    intent: &mut DogfoodReadOnlyIntentV1,
    grant: &mut DogfoodLaunchGrantClaimsV1,
) {
    let provider = &mut intent.subject.provider;
    provider.provider = enrollment.provider;
    provider.protocol = enrollment.protocol;
    provider.provider_profile_id = enrollment.provider_profile_id.clone();
    provider.runtime_passport_id = enrollment.runtime_passport_id.clone();
    provider.provider_enrollment_id = enrollment.enrollment_id().unwrap();
    intent.subject.policy.policy_snapshot_digest = enrollment.policy_snapshot_digest;
    intent.subject.policy.policy_generation = enrollment.policy_generation;
    intent.subject.policy.egress_policy_digest = enrollment.egress_policy_digest;
    intent.subject.policy.tool_policy_digest = enrollment.tool_policy_digest;
    grant.intent_id = intent.intent_id().unwrap();
    grant.subject = intent.subject.clone();
}

fn refusal(result: Result<(), WireError>, code: &'static str) {
    let error = result.unwrap_err();
    assert_eq!(error.code(), code, "{error}");
}

#[test]
fn all_four_exact_runtime_bodies_close_the_unsigned_subject_graph() {
    for provider in [
        LaunchProvider::Claude,
        LaunchProvider::Codex,
        LaunchProvider::Cursor,
        LaunchProvider::Agy,
    ] {
        let (passport, enrollment, intent, grant) = fixture(provider);
        verify_dogfood_runtime_binding(&grant, &intent, &enrollment, &passport).unwrap();
        assert!(enrollment.service_identity_id.as_str().starts_with("pri_"));
    }
}

#[test]
fn malformed_and_internally_substituted_passports_keep_structural_codes() {
    let (mut subject, enrollment, intent, grant) = fixture(LaunchProvider::Claude);
    subject.deployment_root = "/tmp/claude".into();
    refusal(
        verify_dogfood_runtime_binding(&grant, &intent, &enrollment, &subject),
        "RUNTIME_PASSPORT_MALFORMED",
    );

    subject = passport(LaunchProvider::Claude);
    subject.protocol = DogfoodProviderProtocolV1::CodexAppServerJsonl;
    refusal(
        verify_dogfood_runtime_binding(&grant, &intent, &enrollment, &subject),
        "RUNTIME_PASSPORT_PROTOCOL_MISMATCH",
    );
}

#[test]
fn changed_body_id_refuses_before_enrollment_semantics() {
    let (mut passport, enrollment, intent, grant) = fixture(LaunchProvider::Claude);
    passport.version = "2.1.252".into();
    passport.deployment_root = "/usr/lib/bullet/providers/claude/2.1.252".into();
    refusal(
        verify_dogfood_runtime_binding(&grant, &intent, &enrollment, &passport),
        "RUNTIME_PASSPORT_ID_MISMATCH",
    );
}

#[test]
fn reissued_enrollment_cannot_contradict_runtime_provider_protocol_or_version() {
    let (passport, mut enrollment, mut intent, mut grant) = fixture(LaunchProvider::Claude);
    enrollment.runtime_version = "2.1.252".into();
    rebind(&enrollment, &mut intent, &mut grant);
    refusal(
        verify_dogfood_runtime_binding(&grant, &intent, &enrollment, &passport),
        "PROVIDER_ENROLLMENT_RUNTIME_MISMATCH",
    );

    enrollment.provider = LaunchProvider::Codex;
    enrollment.protocol = DogfoodProviderProtocolV1::CodexAppServerJsonl;
    enrollment.runtime_version = passport.version.clone();
    rebind(&enrollment, &mut intent, &mut grant);
    refusal(
        verify_dogfood_runtime_binding(&grant, &intent, &enrollment, &passport),
        "PROVIDER_ENROLLMENT_RUNTIME_MISMATCH",
    );
}
