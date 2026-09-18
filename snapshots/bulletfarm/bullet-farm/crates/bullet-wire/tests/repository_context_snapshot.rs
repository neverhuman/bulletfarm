use std::{collections::BTreeSet, str::FromStr};

use bullet_wire::{
    AttemptId, Blake3Digest, CheckpointId, CommandId, DogfoodBudgetReservationId,
    DogfoodExecutionSubjectV1, DogfoodPolicySubjectV1, DogfoodProviderProtocolV1,
    DogfoodProviderSubjectV1, DogfoodReadOnlyIntentV1, DogfoodRepositorySubjectV1, DogfoodRunId,
    DogfoodRunSubjectV1, GateId, GitOid, GraphRevisionId, LaunchProvider,
    MAX_REPOSITORY_CONTEXT_FILES, MAX_REPOSITORY_CONTEXT_TOTAL_BYTES, MissionId, PrincipalId,
    ProviderCredentialProjectionId, ProviderEnrollmentId, ProviderProfileId, RepoPath,
    RepositoryContextPostObservationV1, RepositoryContextSnapshotId, RepositoryContextSnapshotV1,
    RepositoryId, RepositoryVisibleFileV1, RunnerId, RuntimePassportId, SourceDescriptorId,
    VariantId, WireError, WorkPackageId, canonical_json,
    decode_repository_context_post_observation, decode_repository_context_snapshot,
    verify_repository_context_binding, verify_repository_context_post_observation,
};
use serde_json::{Value, json};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

fn digest(seed: u8) -> Blake3Digest {
    Blake3Digest::from_bytes([seed; 32])
}

fn oid(seed: u8) -> GitOid {
    GitOid::Sha256(format!("{seed:02x}").repeat(32))
}

fn path(value: &str) -> RepoPath {
    RepoPath::from_str(value).unwrap()
}

fn visible_file(value: &str, seed: u8, size: u64) -> RepositoryVisibleFileV1 {
    RepositoryVisibleFileV1 {
        path: path(value),
        preimage_digest: digest(seed),
        size_bytes: size,
        executable: false,
    }
}

fn snapshot() -> RepositoryContextSnapshotV1 {
    let mut snapshot = RepositoryContextSnapshotV1 {
        schema_version: "v1alpha1".into(),
        run_id: DogfoodRunId::from_digest(digest(1)),
        repository_id: RepositoryId::from_digest(digest(2)),
        source_descriptor_id: SourceDescriptorId::from_digest(digest(3)),
        attempt_id: AttemptId::from_digest(digest(4)),
        attempt_fence: 5,
        owner_principal_id: PrincipalId::from_digest(digest(6)),
        head_oid: oid(7),
        tree_oid: oid(8),
        checkpoint_id: CheckpointId::from_digest(digest(9)),
        checkpoint_digest: digest(10),
        scope_grant_digest: digest(11),
        visible_scopes: vec![path("docs"), path("src")],
        files: vec![
            visible_file("docs/readme.md", 12, 10),
            visible_file("src/lib.rs", 13, 20),
        ],
        aggregate_file_count: 0,
        aggregate_size_bytes: 0,
        visible_manifest_digest: digest(14),
        prepared_at_unix_ms: 1_000,
    };
    refresh(&mut snapshot);
    snapshot
}

fn refresh(snapshot: &mut RepositoryContextSnapshotV1) {
    snapshot.aggregate_file_count = snapshot.files.len() as u64;
    snapshot.aggregate_size_bytes = snapshot.files.iter().map(|file| file.size_bytes).sum();
    snapshot.visible_manifest_digest = snapshot.computed_visible_manifest_digest().unwrap();
}

fn intent(snapshot: &RepositoryContextSnapshotV1) -> DogfoodReadOnlyIntentV1 {
    DogfoodReadOnlyIntentV1 {
        schema_version: "v1alpha1".into(),
        request_digest: digest(15),
        subject: DogfoodRunSubjectV1 {
            execution: DogfoodExecutionSubjectV1 {
                command_id: CommandId::from_digest(digest(16)),
                run_id: snapshot.run_id.clone(),
                mission_id: MissionId::from_digest(digest(17)),
                repository_id: snapshot.repository_id.clone(),
                graph_revision_id: GraphRevisionId::from_digest(digest(18)),
                work_package_id: WorkPackageId::from_digest(digest(19)),
                variant_id: VariantId::from_digest(digest(20)),
                attempt_id: snapshot.attempt_id.clone(),
                attempt_fence: snapshot.attempt_fence,
                runner_id: RunnerId::from_digest(digest(21)),
                runner_epoch: 1,
                authority_epoch: 1,
                freeze_generation: 1,
            },
            provider: DogfoodProviderSubjectV1 {
                provider: LaunchProvider::Claude,
                protocol: DogfoodProviderProtocolV1::ClaudeStreamJson,
                provider_profile_id: ProviderProfileId::from_digest(digest(22)),
                runtime_passport_id: RuntimePassportId::from_digest(digest(23)),
                provider_enrollment_id: ProviderEnrollmentId::from_digest(digest(24)),
                credential_projection_id: ProviderCredentialProjectionId::from_digest(digest(25)),
            },
            repository: DogfoodRepositorySubjectV1 {
                context_snapshot_id: snapshot.context_snapshot_id().unwrap(),
                head_oid: snapshot.head_oid.clone(),
                tree_oid: snapshot.tree_oid.clone(),
                checkpoint_id: snapshot.checkpoint_id.clone(),
            },
            gate_ids: vec![GateId::from_digest(digest(26))],
            prompt_digest: digest(27),
            policy: DogfoodPolicySubjectV1 {
                policy_snapshot_digest: digest(28),
                policy_generation: 1,
                dogfood_binding_digest: digest(29),
                tool_policy_digest: digest(30),
                egress_policy_digest: digest(31),
                containment_policy_digest: digest(32),
            },
            budget_reservation_id: DogfoodBudgetReservationId::from_digest(digest(33)),
            deadline_unix_ms: 5_000,
            output_schema_digest: digest(34),
        },
    }
}

fn post(snapshot: &RepositoryContextSnapshotV1) -> RepositoryContextPostObservationV1 {
    RepositoryContextPostObservationV1 {
        schema_version: "v1alpha1".into(),
        context_snapshot_id: snapshot.context_snapshot_id().unwrap(),
        run_id: snapshot.run_id.clone(),
        observer_principal_id: PrincipalId::from_digest(digest(35)),
        observed_at_unix_ms: 2_000,
        observed_owner_principal_id: snapshot.owner_principal_id.clone(),
        observed_head_oid: snapshot.head_oid.clone(),
        observed_tree_oid: snapshot.tree_oid.clone(),
        observed_checkpoint_id: snapshot.checkpoint_id.clone(),
        observed_checkpoint_digest: snapshot.checkpoint_digest,
        observed_visible_manifest_digest: snapshot.visible_manifest_digest,
    }
}

fn decode_value(value: &Value) -> Result<RepositoryContextSnapshotV1, WireError> {
    decode_repository_context_snapshot(&serde_jcs::to_vec(value).unwrap())
}

fn refusal<T>(result: Result<T, WireError>, expected: &'static str) {
    match result {
        Err(error) => assert_eq!(error.code(), expected, "{error}"),
        Ok(_) => panic!("expected {expected}"),
    }
}

#[test]
fn snapshot_is_content_addressed_closed_and_separate_from_post_observation() {
    let snapshot = snapshot();
    snapshot.validate().unwrap();
    let id = snapshot.context_snapshot_id().unwrap();
    assert!(id.as_str() == "rcs_ed36f87605a28dae470075d28cfde320edee8a76189e9d0c640181f5c4bc3a5c");
    let manifest = snapshot.visible_manifest_digest.to_hex();
    assert!(manifest == "1f4baab611a6340514df7690a65cca706049351178456523b6e7f4ae082f221d");
    let bytes = canonical_json(&snapshot).unwrap();
    assert_eq!(
        decode_repository_context_snapshot(&bytes).unwrap(),
        snapshot
    );
    let object = serde_json::to_value(&snapshot).unwrap();
    assert_eq!(
        object
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "aggregate_file_count".into(),
            "aggregate_size_bytes".into(),
            "attempt_fence".into(),
            "attempt_id".into(),
            "checkpoint_digest".into(),
            "checkpoint_id".into(),
            "files".into(),
            "head_oid".into(),
            "owner_principal_id".into(),
            "prepared_at_unix_ms".into(),
            "repository_id".into(),
            "run_id".into(),
            "schema_version".into(),
            "scope_grant_digest".into(),
            "source_descriptor_id".into(),
            "tree_oid".into(),
            "visible_manifest_digest".into(),
            "visible_scopes".into(),
        ])
    );
    for forbidden in "context_snapshot_id clean read_only unchanged outcome".split_whitespace() {
        assert!(!object.as_object().unwrap().contains_key(forbidden));
    }
    let post = post(&snapshot);
    let observed = post.observation_digest().unwrap();
    assert!(
        observed.to_hex() == "a5e54192a704cef280f59a280bc52427cba8a1adae60ff53b7e66127ddab8752"
    );
    verify_repository_context_post_observation(&snapshot, &post).unwrap();
    verify_repository_context_binding(&intent(&snapshot), &snapshot).unwrap();
}

#[test]
fn manifest_paths_counts_sizes_and_document_are_independently_bounded() {
    let base = snapshot();
    let mut invalid = base.clone();
    invalid.visible_scopes.clear();
    refusal(invalid.validate(), "REPOSITORY_CONTEXT_INVALID");
    invalid = base.clone();
    invalid.files.clear();
    refresh(&mut invalid);
    refusal(invalid.validate(), "REPOSITORY_CONTEXT_INVALID");
    for scopes in [
        vec![path("src"), path("docs")],
        vec![path("src"), path("src/lib")],
        vec![path("SRC"), path("src")],
        vec![path("SRC/STRASSE"), path("src/straße")],
    ] {
        invalid = base.clone();
        invalid.visible_scopes = scopes;
        refresh(&mut invalid);
        refusal(invalid.validate(), "REPOSITORY_CONTEXT_INVALID");
    }
    invalid = base.clone();
    invalid.files.swap(0, 1);
    refresh(&mut invalid);
    refusal(invalid.validate(), "REPOSITORY_CONTEXT_INVALID");
    invalid = base.clone();
    invalid.files = vec![
        visible_file("src/FILE.rs", 40, 1),
        visible_file("src/file.rs", 41, 1),
    ];
    refresh(&mut invalid);
    refusal(invalid.validate(), "REPOSITORY_CONTEXT_INVALID");
    invalid = base.clone();
    invalid.visible_scopes = vec![path("src")];
    invalid.files = vec![
        visible_file("src/STRASSE", 40, 1),
        visible_file("src/straße", 41, 1),
    ];
    refresh(&mut invalid);
    refusal(invalid.validate(), "REPOSITORY_CONTEXT_INVALID");
    invalid = base.clone();
    invalid.files = vec![visible_file("outside.txt", 42, 1)];
    refresh(&mut invalid);
    refusal(invalid.validate(), "REPOSITORY_CONTEXT_INVALID");

    invalid = base.clone();
    invalid.aggregate_file_count += 1;
    refusal(invalid.validate(), "REPOSITORY_CONTEXT_INVALID");
    invalid = base.clone();
    invalid.files[0].size_bytes = MAX_REPOSITORY_CONTEXT_TOTAL_BYTES + 1;
    refresh(&mut invalid);
    refusal(invalid.validate(), "REPOSITORY_CONTEXT_INVALID");
    invalid = base.clone();
    invalid.visible_manifest_digest = digest(43);
    refusal(invalid.validate(), "REPOSITORY_CONTEXT_MANIFEST_MISMATCH");

    let mut boundary = base.clone();
    boundary.files = vec![visible_file(
        "docs/archive.bin",
        43,
        MAX_REPOSITORY_CONTEXT_TOTAL_BYTES,
    )];
    refresh(&mut boundary);
    boundary.validate().unwrap();
    let unsafe_integer_mutations: [fn(&mut RepositoryContextSnapshotV1); 3] = [
        |value| value.aggregate_file_count = MAX_SAFE_INTEGER + 1,
        |value| value.aggregate_size_bytes = MAX_SAFE_INTEGER + 1,
        |value| value.files[0].size_bytes = MAX_SAFE_INTEGER + 1,
    ];
    for mutate in unsafe_integer_mutations {
        invalid = base.clone();
        mutate(&mut invalid);
        refusal(invalid.validate(), "REPOSITORY_CONTEXT_INVALID");
    }

    invalid = base.clone();
    invalid.visible_scopes = (0..129)
        .map(|index| path(&format!("s{index:03}")))
        .collect();
    invalid.files = vec![visible_file("s000/file", 43, 0)];
    refresh(&mut invalid);
    refusal(invalid.validate(), "REPOSITORY_CONTEXT_INVALID");

    let mut disclosure = base.clone();
    disclosure.visible_scopes = vec![path("src")];
    disclosure.files = (0..129)
        .map(|index| visible_file(&format!("src/file-{index:04}"), 44, 0))
        .collect();
    refresh(&mut disclosure);
    disclosure.validate().unwrap();

    let mut maximum = disclosure.clone();
    maximum.files = (0..MAX_REPOSITORY_CONTEXT_FILES)
        .map(|index| visible_file(&format!("src/file-{index:04}"), 45, 0))
        .collect();
    refresh(&mut maximum);
    maximum.validate().unwrap();
    invalid = maximum;
    invalid.files.push(visible_file("src/file-4096", 45, 0));
    refresh(&mut invalid);
    refusal(invalid.validate(), "REPOSITORY_CONTEXT_INVALID");

    invalid = base.clone();
    invalid.visible_scopes = vec![path("src")];
    invalid.files = (0..2_052)
        .map(|index| visible_file(&format!("src/sum-{index:04}"), 45, MAX_SAFE_INTEGER))
        .collect();
    invalid.aggregate_file_count = invalid.files.len() as u64;
    invalid.aggregate_size_bytes = 0;
    refusal(invalid.validate(), "REPOSITORY_CONTEXT_INVALID");

    invalid = disclosure;
    invalid.files = (0..300)
        .map(|index| visible_file(&format!("src/{index:04}-{}", "a".repeat(4_070)), 46, 0))
        .collect();
    refresh(&mut invalid);
    refusal(invalid.validate(), "REPOSITORY_CONTEXT_INVALID");
}

#[test]
fn decoder_refuses_noncanonical_forbidden_and_malformed_members() {
    let snapshot = snapshot();
    let text = String::from_utf8(canonical_json(&snapshot).unwrap()).unwrap();
    let duplicate = text.replacen(
        "\"attempt_fence\":5",
        "\"attempt_fence\":5,\"attempt_fence\":5",
        1,
    );
    assert!(decode_repository_context_snapshot(duplicate.as_bytes()).is_err());
    assert!(decode_repository_context_snapshot(format!(" {text}").as_bytes()).is_err());
    for name in "clean read_only unchanged pass outcome host_root workdir mount_path uid gid mode credential holdout caller_exclusions".split_whitespace() {
        let mut changed = serde_json::to_value(&snapshot).unwrap();
        changed[name] = json!(true);
        assert!(decode_value(&changed).is_err(), "field {name} was accepted");
    }
    let mut missing = serde_json::to_value(&snapshot).unwrap();
    missing
        .as_object_mut()
        .unwrap()
        .remove("source_descriptor_id");
    assert!(decode_value(&missing).is_err());
    for raw in [
        ".git/config",
        "../outside",
        "/absolute",
        "src\\file",
        "src/a:b",
        "src/./file",
        "src/trailing ",
        "src/e\u{301}",
    ] {
        let mut bad_path = serde_json::to_value(&snapshot).unwrap();
        bad_path["files"][0]["path"] = json!(raw);
        assert!(
            decode_value(&bad_path).is_err(),
            "path {raw:?} was accepted"
        );
    }
    let mut mixed = serde_json::to_value(&snapshot).unwrap();
    mixed["tree_oid"] = json!(format!("sha1:{}", "a".repeat(40)));
    refusal(decode_value(&mixed), "REPOSITORY_CONTEXT_INVALID");
    let mut bad_id = serde_json::to_value(&snapshot).unwrap();
    bad_id["checkpoint_id"] = json!("ckp_short");
    assert!(decode_value(&bad_id).is_err());
    for raw in ["deadbeef", "sha1:ABCDEF", "sha256:00"] {
        let mut bad_oid = serde_json::to_value(&snapshot).unwrap();
        bad_oid["head_oid"] = json!(raw);
        assert!(decode_value(&bad_oid).is_err(), "OID {raw:?} was accepted");
    }
    let mut nested = serde_json::to_value(&snapshot).unwrap();
    nested["files"][0]["caller_state"] = json!("clean");
    assert!(decode_value(&nested).is_err());
    let mut post_value = serde_json::to_value(post(&snapshot)).unwrap();
    post_value["unchanged"] = json!(true);
    assert!(
        decode_repository_context_post_observation(&serde_jcs::to_vec(&post_value).unwrap())
            .is_err()
    );
}

#[test]
fn identity_binding_and_post_comparison_derive_every_result() {
    let base = snapshot();
    let expected = base.context_snapshot_id().unwrap();
    let mutations: [fn(&mut RepositoryContextSnapshotV1); 11] = [
        |v| v.run_id = DogfoodRunId::from_digest(digest(50)),
        |v| v.repository_id = RepositoryId::from_digest(digest(51)),
        |v| v.source_descriptor_id = SourceDescriptorId::from_digest(digest(52)),
        |v| v.attempt_id = AttemptId::from_digest(digest(53)),
        |v| v.attempt_fence += 1,
        |v| v.owner_principal_id = PrincipalId::from_digest(digest(54)),
        |v| v.head_oid = oid(55),
        |v| v.tree_oid = oid(56),
        |v| v.checkpoint_id = CheckpointId::from_digest(digest(57)),
        |v| v.checkpoint_digest = digest(58),
        |v| v.scope_grant_digest = digest(59),
    ];
    for mutate in mutations {
        let mut changed = base.clone();
        mutate(&mut changed);
        assert_ne!(changed.context_snapshot_id().unwrap(), expected);
    }
    let mut changed = base.clone();
    changed.files[0].preimage_digest = digest(60);
    refresh(&mut changed);
    assert_ne!(changed.context_snapshot_id().unwrap(), expected);
    let file_mutations: [fn(&mut RepositoryContextSnapshotV1); 3] = [
        |value| value.files[0].path = path("docs/guide.md"),
        |value| value.files[0].size_bytes += 1,
        |value| value.files[0].executable = true,
    ];
    for mutate in file_mutations {
        changed = base.clone();
        mutate(&mut changed);
        refresh(&mut changed);
        assert_ne!(changed.context_snapshot_id().unwrap(), expected);
    }
    changed = base.clone();
    changed.visible_scopes[0] = path("guides");
    changed.files[0].path = path("guides/readme.md");
    refresh(&mut changed);
    assert_ne!(changed.context_snapshot_id().unwrap(), expected);
    changed = base.clone();
    changed.prepared_at_unix_ms += 1;
    assert_ne!(changed.context_snapshot_id().unwrap(), expected);

    changed = base.clone();
    changed.attempt_fence = 0;
    refusal(changed.validate(), "REPOSITORY_CONTEXT_INVALID");
    changed = base.clone();
    changed.attempt_fence = MAX_SAFE_INTEGER + 1;
    refusal(changed.validate(), "REPOSITORY_CONTEXT_INVALID");
    changed = base.clone();
    changed.prepared_at_unix_ms = MAX_SAFE_INTEGER + 1;
    refusal(changed.validate(), "REPOSITORY_CONTEXT_INVALID");

    let mut bound_intent = intent(&base);
    bound_intent.subject.repository.context_snapshot_id =
        RepositoryContextSnapshotId::from_digest(digest(60));
    refusal(
        verify_repository_context_binding(&bound_intent, &base),
        "REPOSITORY_CONTEXT_ID_MISMATCH",
    );
    let intent_mutations: [fn(&mut DogfoodReadOnlyIntentV1); 7] = [
        |value| value.subject.execution.run_id = DogfoodRunId::from_digest(digest(61)),
        |value: &mut DogfoodReadOnlyIntentV1| {
            value.subject.execution.repository_id = RepositoryId::from_digest(digest(62));
        },
        |value| value.subject.execution.attempt_id = AttemptId::from_digest(digest(63)),
        |value| value.subject.execution.attempt_fence += 1,
        |value| value.subject.repository.head_oid = oid(64),
        |value| value.subject.repository.tree_oid = oid(65),
        |value| value.subject.repository.checkpoint_id = CheckpointId::from_digest(digest(66)),
    ];
    for mutate in intent_mutations {
        let mut changed_intent = intent(&base);
        mutate(&mut changed_intent);
        refusal(
            verify_repository_context_binding(&changed_intent, &base),
            "REPOSITORY_CONTEXT_SUBJECT_MISMATCH",
        );
    }

    let post = post(&base);
    let post_digest = post.observation_digest().unwrap();
    let mut observer_changed = post.clone();
    observer_changed.observer_principal_id = PrincipalId::from_digest(digest(64));
    assert_ne!(observer_changed.observation_digest().unwrap(), post_digest);
    let post_mutations: [fn(&mut RepositoryContextPostObservationV1); 8] = [
        |value: &mut RepositoryContextPostObservationV1| value.observed_at_unix_ms = 999,
        |value| value.run_id = DogfoodRunId::from_digest(digest(65)),
        |value| value.observed_owner_principal_id = PrincipalId::from_digest(digest(66)),
        |value| value.observed_head_oid = oid(67),
        |value| value.observed_tree_oid = oid(68),
        |value| value.observed_checkpoint_id = CheckpointId::from_digest(digest(69)),
        |value| value.observed_checkpoint_digest = digest(70),
        |value| value.observed_visible_manifest_digest = digest(71),
    ];
    for mutate in post_mutations {
        let mut changed_post = post.clone();
        mutate(&mut changed_post);
        refusal(
            verify_repository_context_post_observation(&base, &changed_post),
            "REPOSITORY_CONTEXT_POST_MISMATCH",
        );
    }
    let mut wrong_id = post;
    wrong_id.context_snapshot_id = RepositoryContextSnapshotId::from_digest(digest(72));
    refusal(
        verify_repository_context_post_observation(&base, &wrong_id),
        "REPOSITORY_CONTEXT_POST_MISMATCH",
    );
}
