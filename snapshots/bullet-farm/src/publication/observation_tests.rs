use super::*;
use crate::publication::tests::Fixture;

#[test]
fn hosted_observation_preserves_event_and_member_identities_without_release_authority() {
    job_context_contract();
    let fixture = Fixture::new();
    let (store, request, prepared) = fixture.prepared();
    let aggregate = fixture.aggregate(&store, &prepared);
    let artifacts = fixture.temp.path().join("artifacts");
    fs::create_dir(&artifacts).unwrap();
    for name in ARTIFACTS {
        fs::write(artifacts.join(name), b"fixture artifact\n").unwrap();
    }
    let context = Context {
        event_sha: prepared.aggregate_commit.clone(),
        event_name: "pull_request".into(),
        run_id: "1234".into(),
        run_attempt: "2".into(),
        workflow_ref: "neverhuman/bulletfarm/.github/workflows/publication.yml@refs/pull/1/merge"
            .into(),
        workflow_sha: prepared.aggregate_commit.clone(),
    };
    let bytes = observe(&aggregate, &fixture.root, &artifacts, context.clone()).unwrap();
    let value: serde_json::Value = crate::publication::decode(&bytes).unwrap();
    assert_eq!(value["context"]["event_sha"], prepared.aggregate_commit);
    assert_eq!(
        value["members"]["bullet-farm"]["commit"],
        request.manifest.members["bullet-farm"].commit
    );
    assert_eq!(value["context"]["run_attempt"], "2");
    assert_eq!(value["release_authority"], false);
    assert_eq!(
        value["artifact_sha256"].as_object().unwrap().len(),
        ARTIFACTS.len()
    );
    let mut wrong = context.clone();
    wrong.event_sha = request.expected_main;
    assert!(observe(&aggregate, &fixture.root, &artifacts, wrong).is_err());
    let mut wrong = context.clone();
    wrong.run_attempt = "0".into();
    assert!(observe(&aggregate, &fixture.root, &artifacts, wrong).is_err());
    fs::remove_file(artifacts.join(ARTIFACTS[0])).unwrap();
    assert!(observe(&aggregate, &fixture.root, &artifacts, context).is_err());
}

#[test]
fn hosted_artifact_inventory_refuses_extra_missing_empty_or_symbolic_bytes() {
    job_diagnostic_contract();
    validator_mode_contract();
    let temp = tempfile::tempdir().unwrap();
    for name in ARTIFACTS {
        fs::write(temp.path().join(name), b"artifact").unwrap();
    }
    assert!(artifact_hashes(temp.path()).is_ok());
    fs::write(temp.path().join("extra"), b"extra").unwrap();
    assert!(artifact_hashes(temp.path()).is_err());
    fs::remove_file(temp.path().join("extra")).unwrap();
    fs::write(temp.path().join(ARTIFACTS[0]), b"").unwrap();
    assert!(artifact_hashes(temp.path()).is_err());
    fs::remove_file(temp.path().join(ARTIFACTS[0])).unwrap();
    std::os::unix::fs::symlink(
        temp.path().join(ARTIFACTS[1]),
        temp.path().join(ARTIFACTS[0]),
    )
    .unwrap();
    assert!(artifact_hashes(temp.path()).is_err());
}

fn validator_mode_contract() {
    use crate::publication::{ci_job, git, tests::commit};
    use std::os::unix::fs::{PermissionsExt, symlink};
    let fixture = Fixture::new();
    let member = fixture.root.join("bullet-git");
    let name = "ops/ci/artifact-check.sh";
    let path = member.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let bytes = b"#!/bin/sh\nexit 0\n";
    fs::write(&path, bytes).unwrap();
    for mode in [0o644, 0o755] {
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        let revision = commit(&member, "regular validator mode");
        assert_eq!(ci_job::validator_blob(&member, &revision).unwrap(), bytes);
        assert_eq!(git::blob(&member, &revision, name).is_ok(), mode == 0o644);
    }
    fs::remove_file(&path).unwrap();
    symlink("../../source.txt", &path).unwrap();
    let revision = commit(&member, "symbolic validator refuses");
    assert!(ci_job::validator_blob(&member, &revision).is_err());
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    fs::write(path.join("child"), bytes).unwrap();
    let revision = commit(&member, "tree validator refuses");
    assert!(ci_job::validator_blob(&member, &revision).is_err());
    fs::remove_file(path.join("child")).unwrap();
    fs::remove_dir(&path).unwrap();
    let revision = commit(&member, "missing validator refuses");
    assert!(ci_job::validator_blob(&member, &revision).is_err());
}

fn job_fixture() -> (
    Fixture,
    Vec<crate::publication::ci_inventory::Workflow>,
    std::path::PathBuf,
    crate::publication::ci_job::Hosted,
    std::path::PathBuf,
) {
    use crate::publication::{ci_inventory, ci_job, git, tests::commit};
    let fixture = Fixture::new();
    let mut catalog = ci_inventory::reviewed();
    for workflow in &mut catalog {
        let repo = fixture.root.join(workflow.member);
        fs::create_dir_all(repo.join(".github/workflows")).unwrap();
        let bytes = b"name: diagnostic fixture\n";
        fs::write(repo.join(workflow.path), bytes).unwrap();
        workflow.sha256 = store::digest(bytes);
    }
    let hub = fixture.root.join("bullet-farm");
    fs::write(
        hub.join("publication/root/.github/workflows/publication.yml"),
        include_bytes!("../../publication/root/.github/workflows/publication.yml"),
    )
    .unwrap();
    // Exercise the real semantic validator and its complete source-scan dependency set.
    for (name, bytes) in [
        (
            "ops/ci/artifact-check.sh",
            include_bytes!("../../ops/ci/artifact-check.sh").as_slice(),
        ),
        (
            "ops/ci/lib.sh",
            include_bytes!("../../ops/ci/lib.sh").as_slice(),
        ),
        (
            "ops/ci/artifact-path.sh",
            include_bytes!("../../ops/ci/artifact-path.sh").as_slice(),
        ),
        (
            "ops/ci/tool-version.sh",
            include_bytes!("../../ops/ci/tool-version.sh").as_slice(),
        ),
        (
            "ops/ci/toolchain-pins.sh",
            include_bytes!("../../ops/ci/toolchain-pins.sh").as_slice(),
        ),
        (
            "ops/ci/rust-toolchain-boundary.sh",
            include_bytes!("../../ops/ci/rust-toolchain-boundary.sh").as_slice(),
        ),
        (
            "ops/ci/strict-json.sh",
            include_bytes!("../../ops/ci/strict-json.sh").as_slice(),
        ),
        (
            ".node-version",
            include_bytes!("../../.node-version").as_slice(),
        ),
        (
            ".npm-version",
            include_bytes!("../../.npm-version").as_slice(),
        ),
    ] {
        let path = hub.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    for name in crate::publication::MEMBERS {
        commit(&fixture.root.join(name), "diagnostic subject fixture");
    }
    let (store, request, prepared) = fixture.prepared();
    let aggregate = fixture.aggregate(&store, &prepared);
    for subject in request.manifest.members.values() {
        git::execute(
            git::command(&aggregate)
                .arg("fetch")
                .arg(&store.objects)
                .arg(&subject.commit),
            None,
        )
        .unwrap();
    }
    let hosted = ci_job::Hosted {
        event_sha: prepared.aggregate_commit.clone(),
        event_name: "pull_request".into(),
        run_id: "1234".into(),
        run_attempt: "2".into(),
        workflow_ref: "neverhuman/bulletfarm/.github/workflows/publication.yml@refs/pull/1/merge"
            .into(),
        workflow_sha: prepared.aggregate_commit,
        job: "publication_integrity".into(),
    };
    let artifacts = fixture.temp.path().join("member-diagnostics");
    fs::create_dir_all(artifacts.join(".ci-artifacts/observations")).unwrap();
    let member = &request.manifest.members["bullet-farm"];
    let observation = serde_json::json!({
        "schema_version":"bullet.ci-observation.v1", "repository":"bullet-farm",
        "commit_oid": member.commit, "tree_oid":member.tree, "clean":true,
        "commands":["bash scripts/ci-doctor.sh source-scan", "bash ops/ci/source-scan.sh"],
        "tool_versions":{"git":"git version 2.43.0", "gitleaks":"8.21.2", "python":"Python 3.12.3"},
        "outcomes":[{"lane":"source-scan", "status":"PASS", "exit_code":0}],
        "artifact_hashes":[], "signed":false, "evidence_class":"DIAGNOSTIC_ONLY"
    });
    fs::write(
        artifacts.join(".ci-artifacts/observations/source-scan.json"),
        crate::publication::encode(&observation).unwrap(),
    )
    .unwrap();
    (fixture, catalog, aggregate, hosted, artifacts)
}

fn job_context_contract() {
    use crate::publication::{ci_job, ci_plan};
    let (fixture, catalog, aggregate, hosted, _artifacts) = job_fixture();
    let prior_event = std::env::var_os("GITHUB_SHA");
    let admitted = ci_job::admit(
        &aggregate,
        &fixture.root,
        ci_job::KEY,
        hosted.clone(),
        &catalog,
    )
    .unwrap();
    let value = serde_json::to_value(admitted).unwrap();
    assert_eq!(value["purpose"], "BOOTSTRAP_MEMBER_DIAGNOSTIC_VALIDATION");
    assert_eq!(value["execution_evidence"], false);
    assert_eq!(value["hosted"]["event_sha"], hosted.event_sha);
    assert_eq!(value["hosted"]["job"], "publication_integrity");
    assert_eq!(value["subject"]["invocation_key"], ci_job::KEY);
    assert_eq!(
        value["subject"]["invocation"]["matrix"],
        serde_json::json!({})
    );
    assert_ne!(
        value["subject"]["invocation"]["member_commit"],
        hosted.event_sha
    );
    let plan: serde_json::Value =
        crate::publication::decode(ci_plan::build(&aggregate, &catalog).unwrap().as_bytes())
            .unwrap();
    assert_eq!(value["subject"]["plan_sha256"], plan["plan_sha256"]);
    assert_eq!(
        value["root_workflow_sha256"],
        store::digest(include_bytes!(
            "../../publication/root/.github/workflows/publication.yml"
        ))
    );
    for key in [
        "bullet-git:REQUIRED:fast",
        "bullet-git:SCHEDULED:source_scan",
        "bullet-git:REQUIRED:source_scan:os=ubuntu-24.04",
        "bullet-farm:REQUIRED:fast",
        "bullet-farm:SCHEDULED:source_scan",
        "bullet-farm:REQUIRED:source_scan:os=ubuntu-24.04",
        "invented",
    ] {
        let error = ci_job::admit(&aggregate, &fixture.root, key, hosted.clone(), &catalog)
            .err()
            .expect("unsupported key refused");
        assert_eq!(error.code(), "PUBLICATION_CI_JOB_ADAPTER_UNSUPPORTED");
    }
    for field in [
        "event",
        "workflow",
        "job",
        "attempt",
        "run",
        "event-name",
        "ref",
        "merge-group",
    ] {
        let mut wrong = hosted.clone();
        match field {
            "event" => wrong.event_sha = "0".repeat(40),
            "workflow" => wrong.workflow_sha = "0".repeat(40),
            "job" => wrong.job = "invented".into(),
            "attempt" => wrong.run_attempt = "0".into(),
            "run" => wrong.run_id = "1\n2".into(),
            "event-name" => wrong.event_name = "pull_request_target".into(),
            "ref" => {
                wrong.workflow_ref =
                    "neverhuman/other/.github/workflows/publication.yml@refs/pull/1/merge".into()
            }
            _ => wrong.event_name = "merge_group".into(),
        }
        assert!(
            ci_job::admit(&aggregate, &fixture.root, ci_job::KEY, wrong, &catalog).is_err(),
            "{field}"
        );
    }
    // Git uses its own actual member identity and hosted job, never the Hub job.
    let mut git_hosted = hosted.clone();
    git_hosted.job = "bullet_git_source_scan".into();
    let git_context = ci_job::admit(
        &aggregate,
        &fixture.root,
        ci_job::GIT_KEY,
        git_hosted.clone(),
        &catalog,
    )
    .unwrap();
    let git_value = serde_json::to_value(git_context).unwrap();
    assert_eq!(git_value["purpose"], "MEMBER_DIAGNOSTIC_VALIDATION");
    assert_eq!(git_value["hosted"]["event_sha"], hosted.event_sha);
    assert_eq!(git_value["subject"]["invocation_key"], ci_job::GIT_KEY);
    assert_eq!(git_value["subject"]["invocation"]["member"], "bullet-git");
    assert_eq!(
        git_value["subject"]["invocation"]["matrix"],
        serde_json::json!({})
    );
    assert_ne!(
        git_value["subject"]["invocation"]["member_commit"],
        hosted.event_sha
    );
    assert_eq!(git_value["subject"]["plan_sha256"], plan["plan_sha256"]);
    assert!(
        ci_job::admit(
            &aggregate,
            &fixture.root,
            ci_job::KEY,
            git_hosted.clone(),
            &catalog,
        )
        .is_err()
    );
    assert!(
        ci_job::admit(
            &aggregate,
            &fixture.root,
            ci_job::GIT_KEY,
            hosted.clone(),
            &catalog,
        )
        .is_err()
    );
    for invalid in ["event", "attempt", "workflow"] {
        let mut wrong = git_hosted.clone();
        match invalid {
            "event" => wrong.event_sha = "0".repeat(40),
            "attempt" => wrong.run_attempt = "0".into(),
            _ => wrong.workflow_sha = "0".repeat(40),
        }
        assert!(
            ci_job::admit(&aggregate, &fixture.root, ci_job::GIT_KEY, wrong, &catalog,).is_err(),
            "{invalid}"
        );
    }
    // Actual Git validator/lane execution belongs to the explicit family wrapper
    // fixture; standalone Hub unit tests must not read mutable sibling checkouts.
    // The compiled CLI catalog rejects this fixture even with valid hosted context.
    assert!(
        ci_job::admit(
            &aggregate,
            &fixture.root,
            ci_job::KEY,
            hosted.clone(),
            &crate::publication::ci_inventory::reviewed()
        )
        .is_err()
    );
    fs::write(fixture.root.join("bullet-git/source.txt"), "changed").unwrap();
    assert!(ci_job::admit(&aggregate, &fixture.root, ci_job::KEY, hosted, &catalog).is_err());
    assert_eq!(std::env::var_os("GITHUB_SHA"), prior_event);
}

fn job_diagnostic_contract() {
    use crate::publication::ci_job;
    let (fixture, catalog, aggregate, hosted, artifacts) = job_fixture();
    let observe = || {
        ci_job::observe(
            &aggregate,
            &fixture.root,
            ci_job::KEY,
            &artifacts,
            hosted.clone(),
            &catalog,
        )
    };
    let path = artifacts.join(".ci-artifacts/observations/source-scan.json");
    let original = fs::read(&path).unwrap();
    let bytes = observe().expect("actual semantic validator accepts source-scan diagnostic");
    let value = bullet_wire::decode_canonical_value(bytes.as_bytes()).unwrap();
    assert_eq!(value["execution_evidence"], false);
    assert_eq!(value["purpose"], "VALIDATED_MEMBER_DIAGNOSTIC_ONLY");
    assert_eq!(value["release_authority"], false);
    assert_eq!(value["signed"], false);
    assert!(
        value.get("status").is_none(),
        "validation is not a CI run verdict"
    );
    assert_eq!(value["member_observation_sha256"], store::digest(&original));
    assert_eq!(value["validation"]["exit_code"], 0);
    assert_eq!(
        value["test_inventory_kind"],
        "SOURCE_SCAN_HAS_NO_TEST_SUITE"
    );
    assert_eq!(value["selected_tests"], serde_json::json!([]));
    assert_eq!(bytes, observe().unwrap());
    assert_eq!(fs::read(&path).unwrap(), original);
    for variant in [
        "commit",
        "tree",
        "failure",
        "skip",
        "tool",
        "extra-key",
        "commands",
        "artifact",
    ] {
        let mut changed: serde_json::Value = crate::publication::decode(&original).unwrap();
        match variant {
            "commit" => changed["commit_oid"] = hosted.event_sha.clone().into(),
            "tree" => changed["tree_oid"] = "0".repeat(40).into(),
            "failure" => {
                changed["outcomes"][0]["status"] = "FAIL".into();
                changed["outcomes"][0]["exit_code"] = 1.into();
            }
            "skip" => changed["outcomes"][0]["status"] = "SKIPPED".into(),
            "tool" => changed["tool_versions"]["gitleaks"] = "8.21.3".into(),
            "extra-key" => changed["authority"] = true.into(),
            "commands" => changed["commands"] = serde_json::json!(["echo passed"]),
            _ => {
                changed["artifact_hashes"] =
                    serde_json::json!([{"path":".ci-artifacts/fake","sha256":"0".repeat(64)}])
            }
        }
        fs::write(&path, crate::publication::encode(&changed).unwrap()).unwrap();
        assert!(observe().is_err(), "{variant}");
    }
    for bytes in [
        b"".to_vec(),
        b"{\"x\":1,\"x\":2}".to_vec(),
        vec![b' '; 1024 * 1024 + 1],
    ] {
        fs::write(&path, bytes).unwrap();
        assert!(observe().is_err());
    }
    fs::remove_file(&path).unwrap();
    assert!(observe().is_err());
    let external = fixture.temp.path().join("external.json");
    fs::write(&external, &original).unwrap();
    std::os::unix::fs::symlink(&external, &path).unwrap();
    assert!(observe().is_err());
    fs::remove_file(&path).unwrap();
    fs::write(&path, &original).unwrap();
    for extra in [
        "extra",
        ".ci-artifacts/extra",
        ".ci-artifacts/observations/extra",
    ] {
        let path = artifacts.join(extra);
        fs::write(&path, b"extra").unwrap();
        assert!(observe().is_err());
        fs::remove_file(path).unwrap();
    }
    assert_eq!(observe().unwrap(), bytes);
}
