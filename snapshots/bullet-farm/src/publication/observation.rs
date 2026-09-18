#[cfg(test)]
#[path = "observation_tests.rs"]
mod tests;

use super::{Result, Subject, capture, encode, git, oid, read_manifest, require, store};
use crate::coord::CoordError;
use serde::Serialize;
use std::{collections::BTreeMap, fs, io::Read, path::Path};

const WORKFLOW: &str = ".github/workflows/publication.yml";
const ARTIFACTS: [&str; 8] = [
    "cargo-config.toml",
    "publication-tests.log",
    "reconstruct.log",
    "source-scan.json",
    "status.txt",
    "toolchain.txt",
    "verify.log",
    "wrapper-tests.log",
];

#[derive(Clone, Serialize)]
struct Context {
    event_sha: String,
    event_name: String,
    run_id: String,
    run_attempt: String,
    workflow_ref: String,
    workflow_sha: String,
}

impl Context {
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
        })
    }

    fn validate(&self, aggregate: &str) -> Result<()> {
        oid(&self.event_sha)?;
        oid(&self.workflow_sha)?;
        require(
            self.workflow_sha == aggregate,
            "PUBLICATION_WORKFLOW_SUBJECT_MISMATCH",
        )?;
        require(
            self.event_sha == aggregate,
            "PUBLICATION_EVENT_SUBJECT_MISMATCH",
        )?;
        require(
            ["pull_request", "push", "merge_group", "workflow_dispatch"]
                .contains(&self.event_name.as_str()),
            "PUBLICATION_EVENT_INVALID",
        )?;
        for number in [&self.run_id, &self.run_attempt] {
            require(
                !number.is_empty()
                    && number.len() <= 20
                    && !number.starts_with('0')
                    && number.bytes().all(|c| c.is_ascii_digit()),
                "PUBLICATION_RUN_CONTEXT_INVALID",
            )?;
        }
        require(
            self.workflow_ref
                .starts_with(&format!("neverhuman/bulletfarm/{WORKFLOW}@refs/"))
                && self.workflow_ref.len() <= 1024
                && !self.workflow_ref.bytes().any(|c| c.is_ascii_control()),
            "PUBLICATION_WORKFLOW_CONTEXT_INVALID",
        )
    }
}

#[derive(Serialize)]
struct Observation {
    schema_version: &'static str,
    context: Context,
    aggregate_commit: String,
    aggregate_tree: String,
    manifest_sha256: String,
    workflow_path: &'static str,
    workflow_sha256: String,
    members: BTreeMap<String, Subject>,
    artifact_sha256: BTreeMap<String, String>,
    evidence_class: &'static str,
    signed: bool,
    release_authority: bool,
}

fn artifact_hashes(root: &Path) -> Result<BTreeMap<String, String>> {
    use std::os::unix::fs::OpenOptionsExt;
    git::canonical_directory(root)?;
    let mut entries = fs::read_dir(root)
        .map_err(CoordError::io)?
        .map(|entry| {
            entry.map_err(CoordError::io).and_then(|entry| {
                entry.file_name().into_string().map_err(|_| {
                    CoordError::new("PUBLICATION_ARTIFACT_INVALID", "UTF-8 filename required")
                })
            })
        })
        .collect::<Result<Vec<_>>>()?;
    entries.sort();
    entries.retain(|entry| entry != "observation.json");
    require(
        entries.iter().map(String::as_str).eq(ARTIFACTS),
        "PUBLICATION_ARTIFACT_INVENTORY_INVALID",
    )?;
    let mut hashes = BTreeMap::new();
    for name in ARTIFACTS {
        let file = fs::OpenOptions::new()
            .read(true)
            .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK)
            .open(root.join(name))
            .map_err(CoordError::io)?;
        let metadata = file.metadata().map_err(CoordError::io)?;
        require(
            metadata.is_file() && metadata.len() > 0 && metadata.len() <= 16 * 1024 * 1024,
            "PUBLICATION_ARTIFACT_INVALID",
        )?;
        let mut bytes = Vec::new();
        file.take(16 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(CoordError::io)?;
        require(
            bytes.len() as u64 == metadata.len(),
            "PUBLICATION_ARTIFACT_CHANGED",
        )?;
        hashes.insert(name.into(), store::digest(&bytes));
    }
    Ok(hashes)
}

fn observe(aggregate: &Path, root: &Path, artifacts: &Path, context: Context) -> Result<Vec<u8>> {
    let manifest = read_manifest(aggregate)?;
    let aggregate_commit = git::text(aggregate, &["rev-parse", "HEAD"])?;
    context.validate(&aggregate_commit)?;
    require(capture(root)? == manifest, "PUBLICATION_CI_MEMBER_DRIFT")?;
    let observation = Observation {
        schema_version: "bullet.publication-ci-observation.v1",
        context,
        aggregate_commit: aggregate_commit.clone(),
        aggregate_tree: git::text(aggregate, &["rev-parse", "HEAD^{tree}"])?,
        manifest_sha256: store::digest(&git::blob(aggregate, "HEAD", super::MANIFEST)?),
        workflow_path: WORKFLOW,
        workflow_sha256: store::digest(&git::blob(aggregate, "HEAD", WORKFLOW)?),
        members: manifest.members,
        artifact_sha256: artifact_hashes(artifacts)?,
        evidence_class: "DIAGNOSTIC_ONLY",
        signed: false,
        release_authority: false,
    };
    git::checkout(aggregate)?;
    require(
        git::text(aggregate, &["rev-parse", "HEAD"])? == aggregate_commit,
        "PUBLICATION_AGGREGATE_CHANGED",
    )?;
    encode(&observation)
}

pub(super) fn write(aggregate: &Path, root: &Path, artifacts: &Path) -> Result<String> {
    let bytes = observe(aggregate, root, artifacts, Context::environment()?)?;
    store::persist(&artifacts.join("observation.json"), &bytes)?;
    Ok("wrote exact-event publication observation (unsigned diagnostic)".into())
}
