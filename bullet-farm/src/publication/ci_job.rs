//! Bootstrap validation of member diagnostics; no nested job execution claim.
use std::{collections::BTreeMap, fs, io::Read, path::Path, process::Command, time::Duration};

use serde::{Deserialize, Serialize};

use super::{Result, capture, ci_inventory, ci_plan, decode, git, oid, require, store};
use crate::{coord::CoordError, process};

pub(super) const KEY: &str = "bullet-farm:REQUIRED:source_scan";
const WORKFLOW: &str = ".github/workflows/publication.yml";
const ROOT_JOB: &str = "publication_integrity";
const OBSERVATION: &str = ".ci-artifacts/observations/source-scan.json";
const VALIDATOR: &str = "ops/ci/artifact-check.sh";

#[derive(Clone, Serialize)]
pub(super) struct Hosted {
    pub event_sha: String,
    pub event_name: String,
    pub run_id: String,
    pub run_attempt: String,
    pub workflow_ref: String,
    pub workflow_sha: String,
    pub job: String,
}

impl Hosted {
    fn environment() -> Result<Self> {
        require(
            std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true"),
            "PUBLICATION_HOSTED_CONTEXT_REQUIRED",
        )?;
        let env = |name| {
            std::env::var(name)
                .map_err(|_| CoordError::new("PUBLICATION_RUN_CONTEXT_MISSING", name))
        };
        Ok(Self {
            event_sha: env("GITHUB_SHA")?,
            event_name: env("GITHUB_EVENT_NAME")?,
            run_id: env("GITHUB_RUN_ID")?,
            run_attempt: env("GITHUB_RUN_ATTEMPT")?,
            workflow_ref: env("GITHUB_WORKFLOW_REF")?,
            workflow_sha: env("GITHUB_WORKFLOW_SHA")?,
            job: env("GITHUB_JOB")?,
        })
    }

    fn validate(&self, aggregate: &str) -> Result<()> {
        oid(&self.event_sha)?;
        oid(&self.workflow_sha)?;
        require(
            self.event_sha == aggregate && self.workflow_sha == aggregate,
            "PUBLICATION_CI_EVENT_SUBJECT_MISMATCH",
        )?;
        // The current template has these three events; it executes no nested lane.
        require(
            ["pull_request", "push", "workflow_dispatch"].contains(&self.event_name.as_str())
                && self.job == ROOT_JOB,
            "PUBLICATION_CI_BOOTSTRAP_CONTEXT_UNSUPPORTED",
        )?;
        for value in [&self.run_id, &self.run_attempt] {
            require(
                !value.is_empty()
                    && value.len() <= 20
                    && !value.starts_with('0')
                    && value.bytes().all(|c| c.is_ascii_digit()),
                "PUBLICATION_RUN_CONTEXT_INVALID",
            )?;
        }
        let prefix = format!("neverhuman/bulletfarm/{WORKFLOW}@");
        let suffix = self.workflow_ref.strip_prefix(&prefix).unwrap_or_default();
        require(
            self.workflow_ref.len() <= 1024
                && !suffix.bytes().any(|c| c.is_ascii_control())
                && match self.event_name.as_str() {
                    "pull_request" => suffix
                        .strip_prefix("refs/pull/")
                        .and_then(|s| s.strip_suffix("/merge"))
                        .is_some_and(|s| {
                            !s.is_empty()
                                && !s.starts_with('0')
                                && s.bytes().all(|c| c.is_ascii_digit())
                        }),
                    "push" => suffix == "refs/heads/main",
                    "workflow_dispatch" => {
                        suffix.starts_with("refs/heads/") && suffix.len() > "refs/heads/".len()
                    }
                    _ => false,
                },
            "PUBLICATION_WORKFLOW_CONTEXT_INVALID",
        )
    }
}

#[derive(Serialize)]
pub(super) struct Context {
    schema_version: &'static str,
    purpose: &'static str,
    execution_evidence: bool,
    hosted: Hosted,
    subject: ci_plan::AdmittedJob,
    root_workflow_path: &'static str,
    root_workflow_sha256: String,
}

pub(super) fn admit(
    aggregate: &Path,
    root: &Path,
    key: &str,
    hosted: Hosted,
    catalog: &[ci_inventory::Workflow],
) -> Result<Context> {
    require(key == KEY, "PUBLICATION_CI_JOB_ADAPTER_UNSUPPORTED")?;
    let subject = ci_plan::select(aggregate, key, catalog)?;
    hosted.validate(&subject.aggregate_commit)?;
    let reference = hosted
        .workflow_ref
        .split_once('@')
        .expect("validated workflow ref")
        .1;
    git::bytes(aggregate, &["check-ref-format", reference])?;
    require(
        capture(root)? == super::read_manifest_at(aggregate, &subject.aggregate_commit)?,
        "PUBLICATION_CI_MEMBER_DRIFT",
    )?;
    git::checkout(aggregate)?;
    require(
        git::text(aggregate, &["rev-parse", "HEAD"])? == subject.aggregate_commit,
        "PUBLICATION_AGGREGATE_CHANGED",
    )?;
    Ok(Context {
        schema_version: "bullet.publication-ci-job-context.v1",
        purpose: "BOOTSTRAP_MEMBER_DIAGNOSTIC_VALIDATION",
        execution_evidence: false,
        root_workflow_sha256: store::digest(&git::blob(
            aggregate,
            &subject.aggregate_commit,
            WORKFLOW,
        )?),
        root_workflow_path: WORKFLOW,
        hosted,
        subject,
    })
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct MemberObservation {
    schema_version: String,
    repository: String,
    commit_oid: String,
    tree_oid: String,
    clean: bool,
    commands: Vec<String>,
    tool_versions: BTreeMap<String, String>,
    outcomes: Vec<Outcome>,
    artifact_hashes: Vec<Artifact>,
    signed: bool,
    evidence_class: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Outcome {
    lane: String,
    status: String,
    exit_code: u8,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Artifact {
    path: String,
    sha256: String,
}

fn member_observation(bytes: &[u8], context: &Context) -> Result<MemberObservation> {
    let observation: MemberObservation = decode(bytes)?;
    let row = &context.subject.invocation;
    require(
        observation.schema_version == "bullet.ci-observation.v1"
            && observation.repository == row.member
            && observation.commit_oid == row.member_commit
            && observation.tree_oid == row.member_tree
            && observation.clean
            && !observation.signed
            && observation.evidence_class == "DIAGNOSTIC_ONLY"
            && observation.commands
                == [
                    "bash scripts/ci-doctor.sh source-scan",
                    "bash ops/ci/source-scan.sh",
                ]
            && observation.artifact_hashes.is_empty()
            && matches!(observation.outcomes.as_slice(), [outcome]
            if outcome.lane == "source-scan" && outcome.status == "PASS" && outcome.exit_code == 0),
        "PUBLICATION_CI_MEMBER_DIAGNOSTIC_INVALID",
    )?;
    Ok(observation)
}

fn read_regular(path: &Path) -> Result<Vec<u8>> {
    use std::os::unix::fs::OpenOptionsExt;
    let file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK)
        .open(path)
        .map_err(CoordError::io)?;
    let metadata = file.metadata().map_err(CoordError::io)?;
    require(
        metadata.is_file() && metadata.len() > 0 && metadata.len() <= 1024 * 1024,
        "PUBLICATION_CI_DIAGNOSTIC_FILE_INVALID",
    )?;
    let mut bytes = Vec::new();
    file.take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    require(
        bytes.len() as u64 == metadata.len(),
        "PUBLICATION_CI_DIAGNOSTIC_CHANGED",
    )?;
    Ok(bytes)
}

fn diagnostic_bytes(root: &Path) -> Result<Vec<u8>> {
    for (directory, leaf) in [
        (root.to_path_buf(), ".ci-artifacts"),
        (root.join(".ci-artifacts"), "observations"),
        (root.join(".ci-artifacts/observations"), "source-scan.json"),
    ] {
        git::canonical_directory(&directory)?;
        let entries = fs::read_dir(&directory)
            .map_err(CoordError::io)?
            .take(2)
            .map(|entry| entry.map(|entry| entry.file_name()).map_err(CoordError::io))
            .collect::<Result<Vec<_>>>()?;
        require(
            entries == [std::ffi::OsString::from(leaf)],
            "PUBLICATION_CI_DIAGNOSTIC_INVENTORY_INVALID",
        )?;
    }
    read_regular(&root.join(OBSERVATION))
}

#[derive(Serialize)]
struct Validation {
    program: &'static str,
    script_path: &'static str,
    script_sha256: String,
    exit_code: i32,
    stdout_sha256: String,
    stderr_sha256: String,
}

fn validate_member(root: &Path, artifacts: &Path, context: &Context) -> Result<Validation> {
    let member = root.join(context.subject.invocation.member);
    let script = member.join(VALIDATOR);
    let script_sha256 = store::digest(&read_regular(&script)?);
    require(
        script_sha256
            == store::digest(&git::blob(
                &member,
                &context.subject.invocation.member_commit,
                VALIDATOR,
            )?),
        "PUBLICATION_CI_VALIDATOR_CHANGED",
    )?;
    // Source checks bound this cooperative validation interval. They do not
    // isolate a hostile same-UID writer swapping the script or its sourced
    // dependencies during execution, and confer no execution authority.
    let mut command = Command::new("/usr/bin/bash");
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("LC_ALL", "C")
        .env("TZ", "UTC")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .current_dir(&member)
        .arg(&script)
        .arg("source-scan")
        .arg(&context.subject.invocation.member_commit)
        .arg(artifacts)
        .arg("atomic");
    let result = process::run_bounded(
        &mut command,
        "member source-scan diagnostic validation",
        process::Limits {
            timeout: Duration::from_secs(30),
            stdout_bytes: 64 * 1024,
            stderr_bytes: 64 * 1024,
        },
    )?;
    require(
        result.status.success(),
        "PUBLICATION_CI_MEMBER_VALIDATION_FAILED",
    )?;
    require(
        store::digest(&read_regular(&script)?) == script_sha256,
        "PUBLICATION_CI_VALIDATOR_CHANGED",
    )?;
    Ok(Validation {
        program: "/usr/bin/bash",
        script_path: VALIDATOR,
        script_sha256,
        exit_code: 0,
        stdout_sha256: store::digest(&result.stdout),
        stderr_sha256: store::digest(&result.stderr),
    })
}

#[derive(Serialize)]
struct Observation {
    schema_version: &'static str,
    purpose: &'static str,
    execution_evidence: bool,
    context: Context,
    member_observation_path: &'static str,
    member_observation_sha256: String,
    member_observation: MemberObservation,
    validation: Validation,
    test_inventory_kind: &'static str,
    selected_tests: Vec<String>,
    completed_tests: Vec<String>,
    signed: bool,
    release_authority: bool,
    evidence_class: &'static str,
}

pub(super) fn observe(
    aggregate: &Path,
    root: &Path,
    key: &str,
    artifacts: &Path,
    hosted: Hosted,
    catalog: &[ci_inventory::Workflow],
) -> Result<String> {
    let context = admit(aggregate, root, key, hosted.clone(), catalog)?;
    require(
        !artifacts.starts_with(aggregate) && !artifacts.starts_with(root),
        "PUBLICATION_CI_DIAGNOSTICS_MUST_BE_EXTERNAL",
    )?;
    let bytes = diagnostic_bytes(artifacts)?;
    let member_observation = member_observation(&bytes, &context)?;
    let validation = validate_member(root, artifacts, &context)?;
    require(
        diagnostic_bytes(artifacts)? == bytes,
        "PUBLICATION_CI_DIAGNOSTIC_CHANGED",
    )?;
    let final_context = admit(aggregate, root, key, hosted, catalog)?;
    require(
        canonical(&final_context)? == canonical(&context)?,
        "PUBLICATION_CI_CONTEXT_CHANGED",
    )?;
    canonical(&Observation {
        schema_version: "bullet.publication-ci-job-observation.v1",
        purpose: "VALIDATED_MEMBER_DIAGNOSTIC_ONLY",
        execution_evidence: false,
        context,
        member_observation_path: OBSERVATION,
        member_observation_sha256: store::digest(&bytes),
        member_observation,
        validation,
        // A source scanner has no test suite. Empty lists cannot stand in for test execution.
        test_inventory_kind: "SOURCE_SCAN_HAS_NO_TEST_SUITE",
        selected_tests: vec![],
        completed_tests: vec![],
        signed: false,
        release_authority: false,
        evidence_class: "DIAGNOSTIC_ONLY",
    })
}

fn canonical(value: &impl Serialize) -> Result<String> {
    String::from_utf8(
        bullet_wire::canonical_json(value).map_err(|error| {
            CoordError::new("PUBLICATION_CI_JOB_JSON_INVALID", error.to_string())
        })?,
    )
    .map_err(|_| CoordError::new("PUBLICATION_ENCODING", "UTF-8 required"))
}

pub(super) fn run(
    aggregate: &Path,
    root: &Path,
    key: &str,
    artifacts: Option<&Path>,
) -> Result<String> {
    let hosted = Hosted::environment()?;
    let catalog = ci_inventory::reviewed();
    match artifacts {
        Some(artifacts) => observe(aggregate, root, key, artifacts, hosted, &catalog),
        None => canonical(&admit(aggregate, root, key, hosted, &catalog)?),
    }
}
