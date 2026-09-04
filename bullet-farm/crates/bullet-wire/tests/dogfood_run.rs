use bullet_wire::{
    AttemptId, Blake3Digest, CheckpointId, CommandId, ContentId,
    DOGFOOD_BUDGET_SETTLEMENT_DIGEST_DOMAIN, DOGFOOD_RUN_DIGEST_DOMAIN, DOGFOOD_SCHEMA_VERSION,
    DogfoodArtifactRefV1, DogfoodBudgetReservationId, DogfoodBudgetReservationV1,
    DogfoodBudgetSettlementV1, DogfoodCleanupObservationV1, DogfoodExecutionSubjectV1,
    DogfoodPolicySubjectV1, DogfoodProcessObservationV1, DogfoodProcessStateV1,
    DogfoodProposalObservationV1, DogfoodProviderProtocolV1, DogfoodProviderSubjectV1,
    DogfoodRunArtifactsV1, DogfoodRunId, DogfoodRunSubjectV1, DogfoodRunV1, DogfoodTerminalStateV1,
    DogfoodUsageSettlementV1, GateId, GitOid, GraphRevisionId, LaunchProvider,
    MAX_DOGFOOD_PROPOSAL_ARTIFACT_BYTES, MAX_DOGFOOD_RETAINED_BYTES, MissionId, PrincipalId,
    ProviderCredentialProjectionId, ProviderEnrollmentId, ProviderProfileId,
    RepositoryContextSnapshotId, RepositoryId, RunnerId, RuntimePassportId, VariantId, WireError,
    WorkPackageId, canonical_json, decode_dogfood_run,
};
use serde_json::{Value, json};

const SAFE_MAX: u64 = 9_007_199_254_740_991;

fn digest(seed: u8) -> Blake3Digest {
    Blake3Digest::from_bytes([seed; 32])
}

fn oid(seed: u8) -> GitOid {
    GitOid::Sha256(format!("{seed:02x}").repeat(32))
}

fn subject() -> DogfoodRunSubjectV1 {
    DogfoodRunSubjectV1 {
        execution: DogfoodExecutionSubjectV1 {
            command_id: CommandId::from_digest(digest(1)),
            run_id: DogfoodRunId::from_digest(digest(2)),
            mission_id: MissionId::from_digest(digest(3)),
            repository_id: RepositoryId::from_digest(digest(4)),
            graph_revision_id: GraphRevisionId::from_digest(digest(5)),
            work_package_id: WorkPackageId::from_digest(digest(6)),
            variant_id: VariantId::from_digest(digest(7)),
            attempt_id: AttemptId::from_digest(digest(8)),
            attempt_fence: 9,
            runner_id: RunnerId::from_digest(digest(10)),
            runner_epoch: 11,
            authority_epoch: 12,
            freeze_generation: 13,
        },
        provider: DogfoodProviderSubjectV1 {
            provider: LaunchProvider::Claude,
            protocol: DogfoodProviderProtocolV1::ClaudeStreamJson,
            provider_profile_id: ProviderProfileId::from_digest(digest(14)),
            runtime_passport_id: RuntimePassportId::from_digest(digest(15)),
            provider_enrollment_id: ProviderEnrollmentId::from_digest(digest(16)),
            credential_projection_id: ProviderCredentialProjectionId::from_digest(digest(17)),
        },
        repository: bullet_wire::DogfoodRepositorySubjectV1 {
            context_snapshot_id: RepositoryContextSnapshotId::from_digest(digest(18)),
            head_oid: oid(19),
            tree_oid: oid(20),
            checkpoint_id: CheckpointId::from_digest(digest(21)),
        },
        gate_ids: vec![GateId::from_digest(digest(22))],
        prompt_digest: digest(23),
        policy: DogfoodPolicySubjectV1 {
            policy_snapshot_digest: digest(24),
            policy_generation: 25,
            dogfood_binding_digest: digest(26),
            tool_policy_digest: digest(27),
            egress_policy_digest: digest(28),
            containment_policy_digest: digest(29),
        },
        budget_reservation_id: DogfoodBudgetReservationId::from_digest(digest(30)),
        deadline_unix_ms: 5_000,
        output_schema_digest: digest(31),
    }
}

fn reservation(subject: &DogfoodRunSubjectV1) -> DogfoodBudgetReservationV1 {
    DogfoodBudgetReservationV1 {
        schema_version: DOGFOOD_SCHEMA_VERSION.into(),
        reservation_id: subject.budget_reservation_id.clone(),
        run_id: subject.execution.run_id.clone(),
        provider: subject.provider.provider,
        provider_profile_id: subject.provider.provider_profile_id.clone(),
        provider_enrollment_id: subject.provider.provider_enrollment_id.clone(),
        budget_policy_digest: digest(32),
        reserved_at_unix_ms: 1_000,
        consume_before_unix_ms: 3_000,
        reserved_cost_micro_usd: 1_000,
        reserved_invocations: 1,
        reserved_wall_time_ms: 900,
        reserved_concurrency: 1,
    }
}

fn known(used: u64, reserved: u64) -> DogfoodUsageSettlementV1 {
    DogfoodUsageSettlementV1::Known {
        used,
        released: reserved.saturating_sub(used),
        overrun: used.saturating_sub(reserved),
    }
}

fn artifact(seed: u8, size_bytes: u64) -> DogfoodArtifactRefV1 {
    DogfoodArtifactRefV1 {
        digest: digest(seed),
        size_bytes,
    }
}

fn fixture() -> (DogfoodBudgetReservationV1, DogfoodRunV1) {
    let subject = subject();
    let reservation = reservation(&subject);
    let budget_settlement = DogfoodBudgetSettlementV1 {
        schema_version: DOGFOOD_SCHEMA_VERSION.into(),
        reservation_id: reservation.reservation_id.clone(),
        reservation_digest: reservation.reservation_digest().unwrap(),
        settled_at_unix_ms: 2_300,
        cost_micro_usd: known(100, reservation.reserved_cost_micro_usd),
        invocations: known(1, reservation.reserved_invocations),
        wall_time_ms: known(100, reservation.reserved_wall_time_ms),
        concurrency: known(1, reservation.reserved_concurrency),
    };
    let run = DogfoodRunV1 {
        schema_version: DOGFOOD_SCHEMA_VERSION.into(),
        subject,
        intent_id: bullet_wire::DogfoodIntentId::from_digest(digest(33)),
        launch_grant_id: bullet_wire::DogfoodGrantId::from_digest(digest(34)),
        credential_projection_digest: digest(35),
        budget_settlement,
        repository_context_post_observation_digest: digest(36),
        provider_probe_observation_digest: digest(37),
        attestor_principal_id: PrincipalId::from_digest(digest(38)),
        process: DogfoodProcessObservationV1 {
            state: DogfoodProcessStateV1::Exited { code: 0 },
            started_at_unix_ms: Some(2_100),
            ended_at_unix_ms: Some(2_200),
            observation_digest: digest(39),
        },
        artifacts: DogfoodRunArtifactsV1 {
            stdout: artifact(40, 10),
            stderr: artifact(41, 11),
            events: artifact(42, 12),
            proxy: artifact(43, 13),
            containment_receipt_digest: digest(44),
            egress_receipt_digest: digest(45),
            canary_observation_digest: digest(46),
            process_tree_observation_digest: digest(47),
            artifact_manifest_digest: digest(48),
            retained_artifacts: vec![artifact(49, 14), artifact(50, 15)],
            retained_artifact_count: 2,
            retained_artifact_size_bytes: 29,
        },
        proposal: DogfoodProposalObservationV1::Validated {
            proposal_id: ContentId::from_digest(digest(51)),
            proposal_digest: digest(52),
            artifact: artifact(53, 16),
        },
        cleanup: DogfoodCleanupObservationV1::ProvedEmpty {
            receipt_digest: digest(54),
            observed_at_unix_ms: 2_400,
        },
        attested_at_unix_ms: 2_500,
    };
    (reservation, run)
}

fn refusal<T>(result: Result<T, WireError>, code: &'static str) -> WireError {
    match result {
        Err(error) => {
            assert_eq!(error.code(), code, "{error}");
            error
        }
        Ok(_) => panic!("expected {code}"),
    }
}

fn zero_settlement(run: &mut DogfoodRunV1, reservation: &DogfoodBudgetReservationV1) {
    run.budget_settlement.cost_micro_usd = known(0, reservation.reserved_cost_micro_usd);
    run.budget_settlement.invocations = known(0, reservation.reserved_invocations);
    run.budget_settlement.wall_time_ms = known(0, reservation.reserved_wall_time_ms);
    run.budget_settlement.concurrency = known(0, reservation.reserved_concurrency);
}

fn unknown_settlement(run: &mut DogfoodRunV1, reservation: &DogfoodBudgetReservationV1) {
    run.budget_settlement.cost_micro_usd = DogfoodUsageSettlementV1::Unknown {
        retained: reservation.reserved_cost_micro_usd,
    };
    run.budget_settlement.invocations = DogfoodUsageSettlementV1::Unknown {
        retained: reservation.reserved_invocations,
    };
    run.budget_settlement.wall_time_ms = DogfoodUsageSettlementV1::Unknown {
        retained: reservation.reserved_wall_time_ms,
    };
    run.budget_settlement.concurrency = DogfoodUsageSettlementV1::Unknown {
        retained: reservation.reserved_concurrency,
    };
}

#[test]
fn derives_all_five_states_without_a_serialized_outcome() {
    let (reservation, ready) = fixture();
    assert_eq!(
        ready.terminal_state(&reservation).unwrap(),
        DogfoodTerminalStateV1::ProposalReady
    );
    let mut failed = ready.clone();
    failed.process.state = DogfoodProcessStateV1::Exited { code: 1 };
    assert_eq!(
        failed.terminal_state(&reservation).unwrap(),
        DogfoodTerminalStateV1::Failed
    );
    let mut refused = ready.clone();
    refused.process = DogfoodProcessObservationV1 {
        state: DogfoodProcessStateV1::NotStarted,
        started_at_unix_ms: None,
        ended_at_unix_ms: None,
        observation_digest: digest(55),
    };
    refused.proposal = DogfoodProposalObservationV1::Absent;
    zero_settlement(&mut refused, &reservation);
    assert_eq!(
        refused.terminal_state(&reservation).unwrap(),
        DogfoodTerminalStateV1::RefusedBeforeSpawn
    );
    let mut unknown = ready.clone();
    unknown.process.state = DogfoodProcessStateV1::OutcomeUnknown;
    unknown.process.ended_at_unix_ms = None;
    unknown.proposal = DogfoodProposalObservationV1::Absent;
    unknown_settlement(&mut unknown, &reservation);
    assert!(unknown.budget_settlement.has_unknown_liability());
    assert_eq!(
        unknown.terminal_state(&reservation).unwrap(),
        DogfoodTerminalStateV1::OutcomeUnknown
    );
    unknown.process.started_at_unix_ms = None;
    assert_eq!(
        unknown.terminal_state(&reservation).unwrap(),
        DogfoodTerminalStateV1::OutcomeUnknown
    );
    unknown.process.ended_at_unix_ms = Some(2_200);
    assert_eq!(
        unknown.terminal_state(&reservation).unwrap(),
        DogfoodTerminalStateV1::OutcomeUnknown
    );
    let mut quarantined = ready;
    quarantined.cleanup = DogfoodCleanupObservationV1::Quarantined {
        receipt_digest: digest(56),
        residue_manifest_digest: digest(57),
        observed_at_unix_ms: 2_400,
    };
    assert_eq!(
        quarantined.terminal_state(&reservation).unwrap(),
        DogfoodTerminalStateV1::Quarantined
    );
}

#[test]
fn settlement_conserves_every_dimension_and_unknown_retains_liability() {
    let (reservation, mut run) = fixture();
    run.budget_settlement
        .validate_against(&reservation)
        .unwrap();
    run.budget_settlement.cost_micro_usd = known(1_100, 1_000);
    run.budget_settlement
        .validate_against(&reservation)
        .unwrap();
    assert_eq!(
        run.terminal_state(&reservation).unwrap(),
        DogfoodTerminalStateV1::Failed
    );
    run.budget_settlement.cost_micro_usd = DogfoodUsageSettlementV1::Known {
        used: 100,
        released: 899,
        overrun: 0,
    };
    refusal(
        run.budget_settlement.validate_against(&reservation),
        "DOGFOOD_RUN_SETTLEMENT_MISMATCH",
    );
    run.budget_settlement.cost_micro_usd = DogfoodUsageSettlementV1::Unknown { retained: 999 };
    refusal(
        run.budget_settlement.validate_against(&reservation),
        "DOGFOOD_RUN_SETTLEMENT_MISMATCH",
    );
    run.budget_settlement.cost_micro_usd = DogfoodUsageSettlementV1::Known {
        used: SAFE_MAX + 1,
        released: 0,
        overrun: 0,
    };
    refusal(
        run.budget_settlement.validate(),
        "DOGFOOD_BUDGET_SETTLEMENT_INVALID",
    );
    assert_eq!(
        DOGFOOD_BUDGET_SETTLEMENT_DIGEST_DOMAIN,
        "dogfood.budget-settlement.v1alpha1"
    );
}

#[test]
fn decoder_is_canonical_bounded_and_recursively_closed() {
    let (_, run) = fixture();
    let bytes = canonical_json(&run).unwrap();
    assert_eq!(decode_dogfood_run(&bytes).unwrap(), run);
    assert_eq!(DOGFOOD_RUN_DIGEST_DOMAIN, "dogfood.run.v1alpha1");
    assert_eq!(
        String::from_utf8(bytes.clone()).unwrap(),
        r#"{"artifacts":{"artifact_manifest_digest":"3030303030303030303030303030303030303030303030303030303030303030","canary_observation_digest":"2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e","containment_receipt_digest":"2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c","egress_receipt_digest":"2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d","events":{"digest":"2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a","size_bytes":12},"process_tree_observation_digest":"2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f","proxy":{"digest":"2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b","size_bytes":13},"retained_artifact_count":2,"retained_artifact_size_bytes":29,"retained_artifacts":[{"digest":"3131313131313131313131313131313131313131313131313131313131313131","size_bytes":14},{"digest":"3232323232323232323232323232323232323232323232323232323232323232","size_bytes":15}],"stderr":{"digest":"2929292929292929292929292929292929292929292929292929292929292929","size_bytes":11},"stdout":{"digest":"2828282828282828282828282828282828282828282828282828282828282828","size_bytes":10}},"attested_at_unix_ms":2500,"attestor_principal_id":"pri_2626262626262626262626262626262626262626262626262626262626262626","budget_settlement":{"concurrency":{"knowledge":"known","overrun":0,"released":0,"used":1},"cost_micro_usd":{"knowledge":"known","overrun":0,"released":900,"used":100},"invocations":{"knowledge":"known","overrun":0,"released":0,"used":1},"reservation_digest":"7ac48388ddceffa20604f8c5b7d82e452107005511b589427941951f2d3ad8c9","reservation_id":"dbr_1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e","schema_version":"v1alpha1","settled_at_unix_ms":2300,"wall_time_ms":{"knowledge":"known","overrun":0,"released":800,"used":100}},"cleanup":{"kind":"proved_empty","observed_at_unix_ms":2400,"receipt_digest":"3636363636363636363636363636363636363636363636363636363636363636"},"credential_projection_digest":"2323232323232323232323232323232323232323232323232323232323232323","intent_id":"dfi_2121212121212121212121212121212121212121212121212121212121212121","launch_grant_id":"dfg_2222222222222222222222222222222222222222222222222222222222222222","process":{"ended_at_unix_ms":2200,"observation_digest":"2727272727272727272727272727272727272727272727272727272727272727","started_at_unix_ms":2100,"state":{"code":0,"kind":"exited"}},"proposal":{"artifact":{"digest":"3535353535353535353535353535353535353535353535353535353535353535","size_bytes":16},"kind":"validated","proposal_digest":"3434343434343434343434343434343434343434343434343434343434343434","proposal_id":"cnt_3333333333333333333333333333333333333333333333333333333333333333"},"provider_probe_observation_digest":"2525252525252525252525252525252525252525252525252525252525252525","repository_context_post_observation_digest":"2424242424242424242424242424242424242424242424242424242424242424","schema_version":"v1alpha1","subject":{"budget_reservation_id":"dbr_1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e","deadline_unix_ms":5000,"execution":{"attempt_fence":9,"attempt_id":"atm_0808080808080808080808080808080808080808080808080808080808080808","authority_epoch":12,"command_id":"cmd_0101010101010101010101010101010101010101010101010101010101010101","freeze_generation":13,"graph_revision_id":"grf_0505050505050505050505050505050505050505050505050505050505050505","mission_id":"mis_0303030303030303030303030303030303030303030303030303030303030303","repository_id":"rep_0404040404040404040404040404040404040404040404040404040404040404","run_id":"dfr_0202020202020202020202020202020202020202020202020202020202020202","runner_epoch":11,"runner_id":"run_0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a0a","variant_id":"var_0707070707070707070707070707070707070707070707070707070707070707","work_package_id":"wpk_0606060606060606060606060606060606060606060606060606060606060606"},"gate_ids":["gat_1616161616161616161616161616161616161616161616161616161616161616"],"output_schema_digest":"1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f","policy":{"containment_policy_digest":"1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d","dogfood_binding_digest":"1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a","egress_policy_digest":"1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c1c","policy_generation":25,"policy_snapshot_digest":"1818181818181818181818181818181818181818181818181818181818181818","tool_policy_digest":"1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b"},"prompt_digest":"1717171717171717171717171717171717171717171717171717171717171717","provider":{"credential_projection_id":"pcp_1111111111111111111111111111111111111111111111111111111111111111","protocol":"claude_stream_json","provider":"claude","provider_enrollment_id":"pen_1010101010101010101010101010101010101010101010101010101010101010","provider_profile_id":"prf_0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e","runtime_passport_id":"rtp_0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f"},"repository":{"checkpoint_id":"ckp_1515151515151515151515151515151515151515151515151515151515151515","context_snapshot_id":"rcs_1212121212121212121212121212121212121212121212121212121212121212","head_oid":"sha256:1313131313131313131313131313131313131313131313131313131313131313","tree_oid":"sha256:1414141414141414141414141414141414141414141414141414141414141414"}}}"#
    );
    assert_eq!(
        run.digest().unwrap().to_hex(),
        "2b6f25d694b280a5fcb4be6ab24f1e8d88c2bf03f4fe08df349a377ba7746636"
    );
    assert_eq!(
        String::from_utf8(canonical_json(&run.budget_settlement).unwrap()).unwrap(),
        r#"{"concurrency":{"knowledge":"known","overrun":0,"released":0,"used":1},"cost_micro_usd":{"knowledge":"known","overrun":0,"released":900,"used":100},"invocations":{"knowledge":"known","overrun":0,"released":0,"used":1},"reservation_digest":"7ac48388ddceffa20604f8c5b7d82e452107005511b589427941951f2d3ad8c9","reservation_id":"dbr_1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e","schema_version":"v1alpha1","settled_at_unix_ms":2300,"wall_time_ms":{"knowledge":"known","overrun":0,"released":800,"used":100}}"#
    );
    assert_eq!(
        run.budget_settlement.digest().unwrap().to_hex(),
        "e19ac00a147864ade628ddf4e237ecfb4ead47d829ddd99122b680bd32677a56"
    );
    let value = serde_json::to_value(&run).unwrap();
    for field in "status success outcome eligibility pass authority signature key nonce credential token candidate evidence release trusted_time provider_output stdout_text path home workdir environment argv"
        .split_whitespace()
    {
        let mut hostile = value.clone();
        hostile[field] = json!(true);
        refusal(
            decode_dogfood_run(&serde_jcs::to_vec(&hostile).unwrap()),
            "DOGFOOD_RUN_INVALID",
        );
    }
    let mut nested = value.clone();
    nested["budget_settlement"]["cost_micro_usd"]["headroom"] = json!(1);
    refusal(
        decode_dogfood_run(&serde_jcs::to_vec(&nested).unwrap()),
        "DOGFOOD_RUN_INVALID",
    );
    let mut missing = value;
    missing.as_object_mut().unwrap().remove("cleanup");
    refusal(
        decode_dogfood_run(&serde_jcs::to_vec(&missing).unwrap()),
        "DOGFOOD_RUN_INVALID",
    );
    refusal(
        decode_dogfood_run(b"{ \"schema_version\": \"v1alpha1\" }"),
        "DOGFOOD_RUN_INVALID",
    );
    refusal(
        decode_dogfood_run(br#"{"schema_version":"v1alpha1","schema_version":"v1alpha1"}"#),
        "DOGFOOD_RUN_INVALID",
    );
    let exact = vec![b' '; 64 * 1024];
    let over = vec![b' '; 64 * 1024 + 1];
    assert!(
        !refusal(decode_dogfood_run(&exact), "DOGFOOD_RUN_INVALID")
            .to_string()
            .contains("exceeds")
    );
    assert!(
        refusal(decode_dogfood_run(&over), "DOGFOOD_RUN_INVALID")
            .to_string()
            .contains("exceeds")
    );
}

#[test]
fn impossible_process_cleanup_and_artifact_shapes_refuse() {
    let (reservation, run) = fixture();
    let mut invalid = run.clone();
    invalid.process.state = DogfoodProcessStateV1::OutcomeUnknown;
    invalid.process.ended_at_unix_ms = None;
    refusal(invalid.validate(), "DOGFOOD_RUN_PROCESS_MISMATCH");
    unknown_settlement(&mut invalid, &reservation);
    refusal(invalid.validate(), "DOGFOOD_RUN_PROCESS_MISMATCH");
    invalid.proposal = DogfoodProposalObservationV1::Absent;
    invalid.validate().unwrap();

    for state in [
        DogfoodProcessStateV1::Exited { code: 256 },
        DogfoodProcessStateV1::Signaled { signal: 0 },
        DogfoodProcessStateV1::Signaled { signal: 256 },
    ] {
        let mut invalid = run.clone();
        invalid.process.state = state;
        refusal(invalid.validate(), "DOGFOOD_RUN_INVALID");
    }
    let mut invalid = run.clone();
    invalid.process.started_at_unix_ms = None;
    refusal(invalid.validate(), "DOGFOOD_RUN_INVALID");
    let mut invalid = run.clone();
    invalid.attested_at_unix_ms = 2_399;
    refusal(invalid.validate(), "DOGFOOD_RUN_TIME_MISMATCH");
    let mut invalid = run.clone();
    invalid.artifacts.stdout.size_bytes = 1024 * 1024 + 1;
    refusal(invalid.validate(), "DOGFOOD_RUN_INVALID");
    let mut proposal = run.clone();
    if let DogfoodProposalObservationV1::Validated { artifact, .. } = &mut proposal.proposal {
        artifact.size_bytes = MAX_DOGFOOD_PROPOSAL_ARTIFACT_BYTES;
    }
    proposal.validate().unwrap();
    if let DogfoodProposalObservationV1::Validated { artifact, .. } = &mut proposal.proposal {
        artifact.size_bytes += 1;
    }
    refusal(proposal.validate(), "DOGFOOD_RUN_INVALID");
    proposal.proposal = DogfoodProposalObservationV1::Rejected {
        artifact: artifact(59, MAX_DOGFOOD_RETAINED_BYTES + 1),
    };
    refusal(proposal.validate(), "DOGFOOD_RUN_INVALID");

    let mut exact = run.clone();
    exact.artifacts.stdout.size_bytes = 1024 * 1024;
    exact.artifacts.retained_artifacts = vec![artifact(60, 32 * 1024 * 1024)];
    exact.artifacts.retained_artifact_count = 1;
    exact.artifacts.retained_artifact_size_bytes = 32 * 1024 * 1024;
    exact.validate().unwrap();
    exact.artifacts.retained_artifacts[0].size_bytes += 1;
    exact.artifacts.retained_artifact_size_bytes += 1;
    refusal(exact.validate(), "DOGFOOD_RUN_INVALID");

    let mut too_many = run.clone();
    too_many.artifacts.retained_artifacts = (60..=124).map(|seed| artifact(seed, 1)).collect();
    too_many.artifacts.retained_artifact_count = 65;
    too_many.artifacts.retained_artifact_size_bytes = 65;
    refusal(too_many.validate(), "DOGFOOD_RUN_INVALID");
    let mut duplicate = run;
    duplicate.artifacts.retained_artifacts = vec![artifact(60, 1), artifact(60, 1)];
    duplicate.artifacts.retained_artifact_size_bytes = 2;
    refusal(duplicate.validate(), "DOGFOOD_RUN_INVALID");
}

#[test]
fn valid_fields_are_digest_bound_and_exact_artifact_edges_are_admitted() {
    let (_, run) = fixture();
    let baseline = run.digest().unwrap();
    let strings = "/intent_id /launch_grant_id /credential_projection_digest /repository_context_post_observation_digest /provider_probe_observation_digest /attestor_principal_id /subject/execution/command_id /subject/execution/run_id /subject/execution/mission_id /subject/execution/repository_id /subject/execution/graph_revision_id /subject/execution/work_package_id /subject/execution/variant_id /subject/execution/attempt_id /subject/execution/runner_id /subject/provider/provider_profile_id /subject/provider/runtime_passport_id /subject/provider/provider_enrollment_id /subject/provider/credential_projection_id /subject/repository/context_snapshot_id /subject/repository/head_oid /subject/repository/tree_oid /subject/repository/checkpoint_id /subject/gate_ids/0 /subject/prompt_digest /subject/policy/policy_snapshot_digest /subject/policy/dogfood_binding_digest /subject/policy/tool_policy_digest /subject/policy/egress_policy_digest /subject/policy/containment_policy_digest /subject/budget_reservation_id /subject/output_schema_digest /budget_settlement/reservation_id /budget_settlement/reservation_digest /process/observation_digest /artifacts/stdout/digest /artifacts/stderr/digest /artifacts/events/digest /artifacts/proxy/digest /artifacts/containment_receipt_digest /artifacts/egress_receipt_digest /artifacts/canary_observation_digest /artifacts/process_tree_observation_digest /artifacts/artifact_manifest_digest /artifacts/retained_artifacts/0/digest /artifacts/retained_artifacts/1/digest /proposal/proposal_id /proposal/proposal_digest /proposal/artifact/digest /cleanup/receipt_digest";
    for pointer in strings.split_whitespace() {
        let mut value = serde_json::to_value(&run).unwrap();
        let leaf = value.pointer_mut(pointer).unwrap();
        let text = leaf.as_str().unwrap();
        let suffix = if text.ends_with('0') { "1" } else { "0" };
        *leaf = Value::String(format!("{}{suffix}", &text[..text.len() - 1]));
        let changed = decode_dogfood_run(&serde_jcs::to_vec(&value).unwrap()).unwrap();
        assert_ne!(changed.digest().unwrap(), baseline);
    }
    let numbers = "/subject/execution/attempt_fence /subject/execution/runner_epoch /subject/execution/authority_epoch /subject/execution/freeze_generation /subject/deadline_unix_ms /subject/policy/policy_generation /budget_settlement/settled_at_unix_ms /budget_settlement/cost_micro_usd/used /budget_settlement/cost_micro_usd/released /budget_settlement/cost_micro_usd/overrun /budget_settlement/invocations/used /budget_settlement/invocations/released /budget_settlement/invocations/overrun /budget_settlement/wall_time_ms/used /budget_settlement/wall_time_ms/released /budget_settlement/wall_time_ms/overrun /budget_settlement/concurrency/used /budget_settlement/concurrency/released /budget_settlement/concurrency/overrun /process/started_at_unix_ms /process/ended_at_unix_ms /process/state/code /artifacts/stdout/size_bytes /artifacts/stderr/size_bytes /artifacts/events/size_bytes /artifacts/proxy/size_bytes /proposal/artifact/size_bytes /cleanup/observed_at_unix_ms /attested_at_unix_ms";
    for pointer in numbers.split_whitespace() {
        let mut value = serde_json::to_value(&run).unwrap();
        let leaf = value.pointer_mut(pointer).unwrap();
        *leaf = json!(leaf.as_u64().unwrap() + 1);
        let changed = decode_dogfood_run(&serde_jcs::to_vec(&value).unwrap()).unwrap();
        assert_ne!(changed.digest().unwrap(), baseline);
    }
    let mut variants = Vec::new();
    let mut changed = run.clone();
    changed.budget_settlement.cost_micro_usd =
        DogfoodUsageSettlementV1::Unknown { retained: 1_000 };
    variants.push(changed);
    let mut changed = run.clone();
    changed.process.state = DogfoodProcessStateV1::TimedOut;
    variants.push(changed);
    let mut changed = run.clone();
    changed.proposal = DogfoodProposalObservationV1::Rejected {
        artifact: artifact(53, 16),
    };
    variants.push(changed);
    let mut changed = run.clone();
    changed.cleanup = DogfoodCleanupObservationV1::Quarantined {
        receipt_digest: digest(54),
        residue_manifest_digest: digest(90),
        observed_at_unix_ms: 2_400,
    };
    variants.push(changed);
    for changed in variants {
        assert_ne!(changed.digest().unwrap(), baseline);
    }

    let mut exact = run.clone();
    exact.artifacts.retained_artifacts = (60..124).map(|seed| artifact(seed, 1)).collect();
    exact.artifacts.retained_artifact_count = 64;
    exact.artifacts.retained_artifact_size_bytes = 64;
    exact.validate().unwrap();
    exact.artifacts.retained_artifacts.swap(0, 1);
    refusal(exact.validate(), "DOGFOOD_RUN_INVALID");

    let mut mismatch = run.clone();
    mismatch.artifacts.retained_artifact_count = 3;
    refusal(mismatch.validate(), "DOGFOOD_RUN_INVALID");
    let mut invalid_time = run;
    invalid_time.budget_settlement.settled_at_unix_ms = 0;
    refusal(invalid_time.validate(), "DOGFOOD_BUDGET_SETTLEMENT_INVALID");
}
