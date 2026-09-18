//! Exact-subject development fusion with atomic ignored-tree publication.

mod publish;

use std::path::Path;

use serde::Serialize;

use crate::{
    checkout::{
        admit_repository_metadata, ensure_ordinary_checkout, required_members, resolve_roots,
        verify_exact_worktree, verify_family,
    },
    coord::CoordError,
    family_lock::{self, FamilyLock},
};

const USAGE: &str = "usage: bullet-family [--root PATH] fuse --source <local|lock>";
const REPOSITORIES: &[&str] = &[
    "bullet-farm",
    "bullet-kernel",
    "bullet-git",
    "bullet-portal",
];
const DEV_SCRIPT: &str = r#"#!/usr/bin/env bash
set -euo pipefail
if [[ "$#" -ne 1 || "$1" != "build" ]]; then
  echo "usage: .fusion/dev.sh build" >&2
  exit 2
fi
fusion_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
family_root="$(cd "$fusion_dir/../.." && pwd -P)"
for repository in bullet-farm bullet-kernel bullet-git bullet-portal; do
  /bin/bash "$family_root/$repository/scripts/ci-local.sh" fast
done
"#;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Source {
    Local,
    Lock,
}

impl Source {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Lock => "lock",
        }
    }
}

#[derive(Serialize)]
struct FusionManifest {
    schema_version: &'static str,
    source: Source,
    bullet_wire_path: &'static str,
    repository: Vec<FusionRepository>,
}

#[derive(Serialize)]
struct FusionRepository {
    name: String,
    path: String,
    commit_oid: String,
    tree_oid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    jeryu_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    jeryu_slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tag: Option<String>,
}

pub fn run(
    current_dir: &Path,
    explicit_root: Option<&str>,
    args: &[String],
) -> Result<String, CoordError> {
    let source = parse_args(args)?;
    reject_symlink_root(explicit_root)?;
    let (family_root, hub_root) = resolve_roots(current_dir, explicit_root)?;
    require_canonical_family(&family_root, &hub_root)?;
    if source == Source::Local {
        require_local_members_present(&family_root)?;
    }
    crate::deps_check::run(current_dir, explicit_root, &["check".to_owned()])?;
    let manifest = match source {
        Source::Local => local_manifest(&family_root, &hub_root)?,
        Source::Lock => locked_manifest(&family_root, &hub_root)?,
    };
    let bytes = toml::to_string_pretty(&manifest)
        .map_err(|error| CoordError::new("FUSION_ENCODE_FAILED", error.to_string()))?
        .into_bytes();
    let source_bytes = format!("{}\n", source.as_str()).into_bytes();
    publish::publish(
        &hub_root,
        &[
            ("manifest.toml", bytes.as_slice(), false),
            ("source", source_bytes.as_slice(), false),
            ("dev.sh", DEV_SCRIPT.as_bytes(), true),
        ],
    )?;
    Ok(format!(
        "fused {} workspace at {}",
        source.as_str(),
        hub_root.join(".fusion").display()
    ))
}

fn require_local_members_present(family_root: &Path) -> Result<(), CoordError> {
    for &name in REPOSITORIES {
        let repo = family_root.join(name);
        match std::fs::symlink_metadata(&repo) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err(CoordError::new(
                    "FAMILY_MEMBER_MISSING",
                    format!(
                        "required canonical member {name} is absent at {}; run `bullet-family doctor --json`, then restore it through the admitted family setup before retrying fusion",
                        repo.display()
                    ),
                ));
            }
            Err(error) => return Err(CoordError::io(error)),
        }
    }
    Ok(())
}

fn parse_args(args: &[String]) -> Result<Source, CoordError> {
    match args {
        [flag, source] if flag == "--source" && source == "local" => Ok(Source::Local),
        [flag, source] if flag == "--source" && source == "lock" => Ok(Source::Lock),
        _ => Err(CoordError::new("USAGE", USAGE)),
    }
}

fn reject_symlink_root(explicit_root: Option<&str>) -> Result<(), CoordError> {
    let Some(raw) = explicit_root else {
        return Ok(());
    };
    let metadata = std::fs::symlink_metadata(raw).map_err(CoordError::io)?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(CoordError::new(
            "INVALID_CHECKOUT",
            "fusion root must be a non-symlink directory",
        ));
    }
    Ok(())
}

fn require_canonical_family(family_root: &Path, hub_root: &Path) -> Result<(), CoordError> {
    let required = required_members(hub_root)?;
    if required != REPOSITORIES {
        return Err(CoordError::new(
            "FAMILY_MEMBER_SET_MISMATCH",
            "fusion requires the four canonical Bullet Farm repositories in canonical order",
        ));
    }
    let expected_hub = family_root.join("bullet-farm");
    if expected_hub.canonicalize().map_err(CoordError::io)? != hub_root {
        return Err(CoordError::new(
            "HUB_LOCATION_MISMATCH",
            "fusion requires bullet-farm as a direct canonical family child",
        ));
    }
    Ok(())
}

fn local_manifest(family_root: &Path, hub_root: &Path) -> Result<FusionManifest, CoordError> {
    let mut repositories = Vec::with_capacity(REPOSITORIES.len());
    for &name in REPOSITORIES {
        let repo = family_root.join(name);
        ensure_ordinary_checkout(&repo, family_root, name)?;
        admit_repository_metadata(&repo, None)?;
        verify_exact_worktree(&repo)?;
        repositories.push(repository_record(name, &repo, None)?);
    }
    debug_assert_eq!(hub_root, family_root.join("bullet-farm"));
    Ok(manifest(Source::Local, repositories))
}

fn locked_manifest(family_root: &Path, hub_root: &Path) -> Result<FusionManifest, CoordError> {
    let lock = family_lock::load(&hub_root.join("family.lock"))?;
    verify_family(family_root, hub_root, &lock)?;
    let mut repositories = Vec::with_capacity(REPOSITORIES.len());
    for &name in REPOSITORIES {
        let repo = family_root.join(name);
        repositories.push(repository_record(name, &repo, Some(&lock))?);
    }
    Ok(manifest(Source::Lock, repositories))
}

fn manifest(source: Source, repository: Vec<FusionRepository>) -> FusionManifest {
    FusionManifest {
        schema_version: "1",
        source,
        bullet_wire_path: "../crates/bullet-wire",
        repository,
    }
}

fn repository_record(
    name: &str,
    repo: &Path,
    lock: Option<&FamilyLock>,
) -> Result<FusionRepository, CoordError> {
    let (commit_oid, tree_oid) = family_lock::checkout_subject(repo)?;
    let locked = lock.and_then(|lock| lock.member(name));
    if let Some(member) = locked
        && (member.commit_oid != commit_oid || member.tree_oid != tree_oid)
    {
        return Err(CoordError::new(
            "LOCKED_SUBJECT_MISMATCH",
            format!("{name} no longer matches its verified lock subject"),
        ));
    }
    Ok(FusionRepository {
        name: name.to_owned(),
        path: relative_path(name).to_owned(),
        commit_oid,
        tree_oid,
        jeryu_url: locked.and_then(|member| member.jeryu_url.clone()),
        jeryu_slug: locked.and_then(|member| member.jeryu_slug.clone()),
        tag: lock.map(|lock| lock.tag.clone()),
    })
}

fn relative_path(name: &str) -> &'static str {
    match name {
        "bullet-farm" => "..",
        "bullet-kernel" => "../../bullet-kernel",
        "bullet-git" => "../../bullet-git",
        "bullet-portal" => "../../bullet-portal",
        _ => unreachable!("repository names are fixed"),
    }
}

#[cfg(test)]
#[path = "fuse/tests.rs"]
mod tests;
