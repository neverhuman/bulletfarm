//! Expected invocation subjects only. This is not an execution observation or verdict.
use std::{collections::BTreeMap, path::Path};

use serde::Serialize;

use super::{
    MEMBERS, Result,
    ci_inventory::{self, Workflow},
    git, read_manifest_at, require, store,
};
use crate::coord::CoordError;

#[derive(Clone, Serialize)]
pub(super) struct Invocation {
    pub member: &'static str,
    pub member_commit: String,
    pub member_tree: String,
    workflow_path: &'static str,
    workflow_sha256: String,
    scope: &'static str,
    job_id: &'static str,
    matrix: BTreeMap<&'static str, &'static str>,
    runner: &'static str,
    condition: &'static str,
    needs: Vec<String>,
}

#[derive(Serialize)]
struct Plan {
    schema_version: &'static str,
    purpose: &'static str,
    execution_evidence: bool,
    aggregate_commit: String,
    aggregate_tree: String,
    manifest_sha256: String,
    catalog_sha256: String,
    workflow_count: usize,
    job_definition_count: usize,
    invocation_count: usize,
    invocations: BTreeMap<String, Invocation>,
}

#[derive(Serialize)]
struct Document {
    plan: Plan,
    plan_sha256: String,
}

#[derive(Serialize)]
pub(super) struct AdmittedJob {
    pub aggregate_commit: String,
    pub aggregate_tree: String,
    pub manifest_sha256: String,
    pub catalog_sha256: String,
    pub plan_sha256: String,
    pub invocation_key: String,
    pub invocation: Invocation,
}

pub(super) fn select(aggregate: &Path, key: &str, workflows: &[Workflow]) -> Result<AdmittedJob> {
    git::checkout(aggregate)?;
    let commit = git::text(aggregate, &["rev-parse", "HEAD"])?;
    let document = document_at(aggregate, &commit, workflows)?;
    let invocation = document.plan.invocations.get(key).cloned().ok_or_else(|| {
        CoordError::new(
            "PUBLICATION_CI_INVOCATION_UNKNOWN",
            "expected invocation key required",
        )
    })?;
    git::checkout(aggregate)?;
    require(
        git::text(aggregate, &["rev-parse", "HEAD"])? == commit,
        "PUBLICATION_AGGREGATE_CHANGED",
    )?;
    Ok(AdmittedJob {
        aggregate_commit: document.plan.aggregate_commit,
        aggregate_tree: document.plan.aggregate_tree,
        manifest_sha256: document.plan.manifest_sha256,
        catalog_sha256: document.plan.catalog_sha256,
        plan_sha256: document.plan_sha256,
        invocation_key: key.into(),
        invocation,
    })
}

fn canonical(value: &impl Serialize) -> Result<Vec<u8>> {
    bullet_wire::canonical_json(value)
        .map_err(|error| CoordError::new("PUBLICATION_CI_PLAN_INVALID", error.to_string()))
}

fn key(workflow: &Workflow, job: &str, os: Option<&str>) -> String {
    let base = format!("{}:{}:{job}", workflow.member, workflow.scope);
    os.map_or(base.clone(), |os| format!("{base}:os={os}"))
}

pub(super) fn run(aggregate: &Path) -> Result<String> {
    build(aggregate, &ci_inventory::reviewed())
}

// The caller supplies trusted compiled topology; CLI never accepts catalog input.
pub(super) fn build(aggregate: &Path, workflows: &[Workflow]) -> Result<String> {
    git::checkout(aggregate)?;
    let aggregate_commit = git::text(aggregate, &["rev-parse", "HEAD"])?;
    let result = build_at(aggregate, &aggregate_commit, workflows)?;
    git::checkout(aggregate)?;
    require(
        git::text(aggregate, &["rev-parse", "HEAD"])? == aggregate_commit,
        "PUBLICATION_AGGREGATE_CHANGED",
    )?;
    Ok(result)
}

pub(super) fn stored(root: &Path, request: &str) -> Result<String> {
    stored_with(root, request, &ci_inventory::reviewed())
}

pub(super) fn stored_with(root: &Path, id: &str, workflows: &[Workflow]) -> Result<String> {
    use fs2::FileExt;
    store::request_id(id)?;
    git::canonical_directory(root)?;
    // Existing custody only: no Store::open initialization or load's object reconstruction.
    let lock = open_record(&root.join("lock"))?;
    lock.try_lock_exclusive().map_err(|_| {
        CoordError::new(
            "PUBLICATION_STORE_BUSY",
            "another request owns publication custody",
        )
    })?;
    let objects = root.join("objects.git");
    git::canonical_directory(&objects)?;
    require(
        git::text(&objects, &["rev-parse", "--is-bare-repository"])? == "true",
        "PUBLICATION_STORE_NOT_BARE",
    )?;
    git::metadata_admission(&objects, &objects)?;
    let request_path = root.join(format!("{id}.request.json"));
    let prepared_path = root.join(format!("{id}.prepared.json"));
    let request_bytes = record_bytes(&request_path)?;
    let prepared_bytes = record_bytes(&prepared_path)?;
    let request: store::Request = super::decode(&request_bytes)?;
    let prepared: store::Prepared = super::decode(&prepared_bytes)?;
    request.manifest.validate()?;
    super::oid(&request.expected_main)?;
    super::oid(&prepared.aggregate_commit)?;
    require(
        request.schema_version == "bullet.publication-request.v1"
            && request.request_id == id
            && prepared.schema_version == "bullet.publication-prepared.v1"
            && prepared.review_ref == format!("refs/heads/publication/{id}")
            && prepared.request_sha256 == store::digest(&request_bytes)
            && request_bytes == super::encode(&request)?
            && prepared_bytes == super::encode(&prepared)?,
        "PUBLICATION_STORE_DRIFT",
    )?;
    let manifest = read_manifest_at(&objects, &prepared.aggregate_commit)?;
    require(manifest == request.manifest, "PUBLICATION_STORE_DRIFT")?;
    let tree = git::text(
        &objects,
        &[
            "rev-parse",
            &format!("{}^{{tree}}", prepared.aggregate_commit),
        ],
    )?;
    let identity = "Bullet publication <publication@bullet.invalid> 946684800 +0000";
    let expected = format!(
        "tree {tree}\nparent {}\nauthor {identity}\ncommitter {identity}\n\nPublish exact Bullet family: {id}\n\nRequest-SHA256: {}\n",
        request.expected_main, prepared.request_sha256
    );
    require(
        git::bytes(
            &objects,
            &["cat-file", "commit", &prepared.aggregate_commit],
        )? == expected.as_bytes(),
        "PUBLICATION_STORE_DRIFT",
    )?;
    let plan = build_at(&objects, &prepared.aggregate_commit, workflows)?;
    require(
        record_bytes(&request_path)? == request_bytes
            && record_bytes(&prepared_path)? == prepared_bytes,
        "PUBLICATION_STORE_DRIFT",
    )?;
    git::canonical_directory(root)?;
    git::metadata_admission(&objects, &objects)?;
    Ok(plan)
}

fn open_record(path: &Path) -> Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK | nix::libc::O_CLOEXEC)
        .open(path)
        .map_err(CoordError::io)?;
    require(
        file.metadata().map_err(CoordError::io)?.is_file(),
        "PUBLICATION_RECORD_INVALID",
    )?;
    Ok(file)
}

fn record_bytes(path: &Path) -> Result<Vec<u8>> {
    use std::io::Read;
    let file = open_record(path)?;
    let length = file.metadata().map_err(CoordError::io)?.len();
    require(
        length > 0 && length <= 1024 * 1024,
        "PUBLICATION_RECORD_INVALID",
    )?;
    let mut bytes = Vec::new();
    file.take(1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    require(bytes.len() as u64 == length, "PUBLICATION_RECORD_INVALID")?;
    Ok(bytes)
}

fn build_at(aggregate: &Path, aggregate_commit: &str, workflows: &[Workflow]) -> Result<String> {
    String::from_utf8(canonical(&document_at(
        aggregate,
        aggregate_commit,
        workflows,
    )?)?)
    .map_err(|_| CoordError::new("PUBLICATION_ENCODING", "UTF-8 required"))
}

fn document_at(
    aggregate: &Path,
    aggregate_commit: &str,
    workflows: &[Workflow],
) -> Result<Document> {
    let catalog = ci_inventory::canonical_catalog(workflows)?;
    super::oid(aggregate_commit)?;
    let manifest = read_manifest_at(aggregate, aggregate_commit)?;
    for member in MEMBERS {
        let subject = &manifest.members[member];
        require(
            git::text(aggregate, &["cat-file", "-t", &subject.commit])? == "commit"
                && git::text(
                    aggregate,
                    &["rev-parse", &format!("{}^{{tree}}", subject.commit)],
                )? == subject.tree,
            "PUBLICATION_CI_SOURCE_TREE_MISMATCH",
        )?;
        let expected =
            [ci_inventory::CI, ci_inventory::SCHEDULED].map(|path| format!("{member}/{path}"));
        let actual = git::text(
            aggregate,
            &[
                "ls-tree",
                "-r",
                "--name-only",
                aggregate_commit,
                "--",
                &format!("{member}/.github/workflows/"),
            ],
        )?;
        require(
            actual.lines().eq(expected.iter().map(String::as_str)),
            "PUBLICATION_CI_WORKFLOW_INVENTORY_DRIFT",
        )?;
    }
    let mut invocations = BTreeMap::new();
    for workflow in &catalog {
        let path = format!("{}/{}", workflow.member, workflow.path);
        require(
            store::digest(&git::blob(aggregate, aggregate_commit, &path)?) == workflow.sha256,
            "PUBLICATION_CI_WORKFLOW_DIGEST_DRIFT",
        )?;
        let subject = &manifest.members[workflow.member];
        for job in &workflow.jobs {
            let needs = workflow
                .jobs
                .iter()
                .filter(|other| job.needs.contains(&other.id))
                .flat_map(|other| {
                    if other.os.is_empty() {
                        vec![key(workflow, other.id, None)]
                    } else {
                        other
                            .os
                            .iter()
                            .map(|os| key(workflow, other.id, Some(os)))
                            .collect()
                    }
                })
                .collect::<Vec<_>>();
            let cells = if job.os.is_empty() {
                vec![None]
            } else {
                job.os.iter().copied().map(Some).collect()
            };
            for os in cells {
                let invocation = Invocation {
                    member: workflow.member,
                    member_commit: subject.commit.clone(),
                    member_tree: subject.tree.clone(),
                    workflow_path: workflow.path,
                    workflow_sha256: workflow.sha256.clone(),
                    scope: workflow.scope,
                    job_id: job.id,
                    matrix: os
                        .map(|value| BTreeMap::from([("os", value)]))
                        .unwrap_or_default(),
                    runner: os.unwrap_or(job.runner),
                    condition: if job.always { "always()" } else { "success()" },
                    needs: needs.clone(),
                };
                require(
                    invocations
                        .insert(key(workflow, job.id, os), invocation)
                        .is_none(),
                    "PUBLICATION_CI_INVOCATION_DUPLICATE",
                )?;
            }
        }
    }
    let plan = Plan {
        schema_version: "bullet.publication-ci-plan.v1",
        purpose: "EXPECTED_INVOCATIONS_ONLY",
        execution_evidence: false,
        aggregate_tree: git::text(
            aggregate,
            &["rev-parse", &format!("{aggregate_commit}^{{tree}}")],
        )?,
        manifest_sha256: store::digest(&git::blob(aggregate, aggregate_commit, super::MANIFEST)?),
        aggregate_commit: aggregate_commit.into(),
        catalog_sha256: store::digest(&canonical(&catalog)?),
        workflow_count: catalog.len(),
        job_definition_count: catalog.iter().map(|w| w.jobs.len()).sum(),
        invocation_count: invocations.len(),
        invocations,
    };
    require(
        plan.invocation_count
            == catalog
                .iter()
                .flat_map(|w| &w.jobs)
                .map(|j| j.os.len().max(1))
                .sum::<usize>(),
        "PUBLICATION_CI_INVOCATION_INVENTORY_INVALID",
    )?;
    Ok(Document {
        plan_sha256: store::digest(&canonical(&plan)?),
        plan,
    })
}
