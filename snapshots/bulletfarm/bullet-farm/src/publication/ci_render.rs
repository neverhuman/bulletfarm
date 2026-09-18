//! Versioned root generation, not evidence of execution or tool admission.
use std::{collections::BTreeMap, path::Path};

use super::{
    MANIFEST, MEMBERS, Manifest, Result,
    ci_inventory::{self, Workflow},
    git, require, store,
};

const PRIMARY: &str = ".github/workflows/publication.yml";
const TEMPLATE_PATH: &str = "publication/root/.github/workflows/publication.yml";
const TEMPLATE: &str = include_str!("../../publication/root/.github/workflows/publication.yml");
mod jobs;
#[cfg(test)]
mod tests;

use jobs::{required, scheduled};

const V2: &str = "bullet.publication-config.v2";

// A clean aggregate supplies only source subjects; its existing roots are not verified.
pub(super) fn preview(repo: &Path) -> Result<String> {
    git::checkout(repo)?;
    let commit = git::text(repo, &["rev-parse", "HEAD"])?;
    let manifest = super::read_source_manifest_at(repo, &commit)?;
    let files = root_files(repo, &manifest)?
        .into_iter()
        .map(|(path, bytes)| {
            String::from_utf8(bytes)
                .map(|text| (path, text))
                .map_err(|_| {
                    crate::coord::CoordError::new("PUBLICATION_ENCODING", "UTF-8 required")
                })
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    git::checkout(repo)?;
    require(
        git::text(repo, &["rev-parse", "HEAD"])? == commit,
        "PUBLICATION_AGGREGATE_CHANGED",
    )?;
    String::from_utf8(super::encode(&serde_json::json!({
        "purpose": "EXPECTED_ROOT_FILES_ONLY",
        "execution_evidence": false,
        "source_aggregate_commit": commit,
        "files": files,
    }))?)
    .map_err(|_| crate::coord::CoordError::new("PUBLICATION_ENCODING", "UTF-8 required"))
}

pub(super) fn root_files(repo: &Path, manifest: &Manifest) -> Result<BTreeMap<String, Vec<u8>>> {
    render(repo, manifest, &ci_inventory::reviewed())
}

fn render(
    repo: &Path,
    manifest: &Manifest,
    workflows: &[Workflow],
) -> Result<BTreeMap<String, Vec<u8>>> {
    manifest.validate()?;
    let hub_tree = &manifest.members["bullet-farm"].tree;
    let mut files = BTreeMap::new();
    for (target, source) in &manifest.tool_config.root_files {
        files.insert(target.clone(), git::blob(repo, hub_tree, source)?);
    }
    // Historical v1 request/commit reconstruction copies the original source bytes.
    if manifest.tool_config.schema_version != V2 {
        return Ok(files);
    }
    require(
        manifest
            .tool_config
            .root_files
            .get(PRIMARY)
            .map(String::as_str)
            == Some(TEMPLATE_PATH)
            && files.get(PRIMARY).map(Vec::as_slice) == Some(TEMPLATE.as_bytes()),
        "PUBLICATION_CI_TEMPLATE_UNSUPPORTED",
    )?;
    let catalog = ci_inventory::canonical_catalog(workflows)?;
    for member in MEMBERS {
        let tree = &manifest.members[member].tree;
        require(
            git::text(
                repo,
                &[
                    "ls-tree",
                    "-r",
                    "--name-only",
                    tree,
                    "--",
                    ".github/workflows/",
                ],
            )?
            .lines()
            .eq([ci_inventory::CI, ci_inventory::SCHEDULED]),
            "PUBLICATION_CI_WORKFLOW_INVENTORY_DRIFT",
        )?;
        for workflow in catalog.iter().filter(|w| w.member == member) {
            require(
                store::digest(&git::blob(repo, tree, workflow.path)?) == workflow.sha256,
                "PUBLICATION_CI_WORKFLOW_DIGEST_DRIFT",
            )?;
        }
    }
    files.insert(PRIMARY.into(), required(manifest, &catalog)?.into_bytes());
    for workflow in catalog.iter().filter(|w| w.scope == "SCHEDULED") {
        let target = format!(".github/workflows/{}-scheduled.yml", workflow.member);
        require(
            !files.contains_key(&target),
            "PUBLICATION_ROOT_PATH_COLLISION",
        )?;
        files.insert(target, scheduled(manifest, workflow).into_bytes());
    }
    let paths = files
        .keys()
        .map(String::as_str)
        .chain(MEMBERS)
        .chain([MANIFEST]);
    let paths = paths.map(str::to_ascii_lowercase).collect::<Vec<_>>();
    for (index, path) in paths.iter().enumerate() {
        require(
            !paths.iter().enumerate().any(|(other_index, other)| {
                index != other_index && (other == path || other.starts_with(&format!("{path}/")))
            }),
            "PUBLICATION_ROOT_PATH_COLLISION",
        )?;
    }
    Ok(files)
}
