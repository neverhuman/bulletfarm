//! Exact, read-only verification of an installed split-family checkout.

mod git;

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{
    coord::CoordError,
    family_lock::{self, FamilyLock, LockedMember},
};

const USAGE: &str = "usage: bullet-family [--root PATH] checkout verify";
const ALLOWED_SIGNERS: &str = "release/allowed_signers";

pub(crate) use self::git::{admit_repository_metadata, verify_exact_worktree};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HubManifest {
    schema_version: String,
    family: String,
    umbrella_repo: String,
    required_repos: Vec<String>,
}

pub fn run(
    current_dir: &Path,
    explicit_root: Option<&str>,
    args: &[String],
) -> Result<String, CoordError> {
    if args != ["verify"] {
        return Err(CoordError::new("USAGE", USAGE));
    }
    let (family_root, hub_root) = resolve_roots(current_dir, explicit_root)?;
    let lock = family_lock::load(&hub_root.join("family.lock"))?;
    verify_family(&family_root, &hub_root, &lock)?;
    Ok(format!(
        "checkout verify: {} members match {}",
        lock.member.len() + 1,
        lock.tag
    ))
}

pub(crate) fn resolve_roots(
    current_dir: &Path,
    explicit_root: Option<&str>,
) -> Result<(PathBuf, PathBuf), CoordError> {
    if let Some(raw) = explicit_root {
        let root = PathBuf::from(raw).canonicalize().map_err(CoordError::io)?;
        let (family_root, hub_root) = if is_hub(&root) {
            let parent = root.parent().ok_or_else(|| {
                CoordError::new("INVALID_ROOT", "hub checkout has no family parent")
            })?;
            (parent.to_path_buf(), root)
        } else {
            (root.clone(), root.join("bullet-farm"))
        };
        if !is_hub(&hub_root) {
            return Err(CoordError::new(
                "HUB_CHECKOUT_NOT_FOUND",
                format!("{} has no Bullet Farm hub checkout", family_root.display()),
            ));
        }
        return Ok((family_root, hub_root));
    }
    let hub_root = crate::doctor::discover_hub(current_dir, None)?;
    let family_root = hub_root
        .parent()
        .ok_or_else(|| CoordError::new("INVALID_ROOT", "hub checkout has no family parent"))?
        .to_path_buf();
    Ok((family_root, hub_root))
}

pub(crate) fn required_members(hub_root: &Path) -> Result<Vec<String>, CoordError> {
    let bytes = fs::read(hub_root.join("repos.manifest.toml")).map_err(CoordError::io)?;
    if bytes.len() > 1024 * 1024 {
        return Err(CoordError::new(
            "INVALID_FAMILY_MANIFEST",
            "hub manifest exceeds the 1 MiB admission limit",
        ));
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| CoordError::new("INVALID_FAMILY_MANIFEST", "hub manifest is not UTF-8"))?;
    let manifest: HubManifest = toml::from_str(text)
        .map_err(|error| CoordError::new("INVALID_FAMILY_MANIFEST", error.to_string()))?;
    if manifest.schema_version != "1.2.0"
        || manifest.family != "bullet-farm"
        || manifest.umbrella_repo != "bullet-farm"
    {
        return Err(CoordError::new(
            "UNSUPPORTED_SCHEMA",
            "hub manifest is not the supported Bullet Farm 1.2.0 schema",
        ));
    }
    Ok(manifest.required_repos)
}

pub(crate) fn verify_family(
    family_root: &Path,
    hub_root: &Path,
    lock: &FamilyLock,
) -> Result<(), CoordError> {
    let required = required_members(hub_root)?;
    lock.validate_required_members(&required)?;
    if lock.family != "bullet-farm" {
        return Err(CoordError::new(
            "INVALID_FAMILY_LOCK",
            "lock does not describe the Bullet Farm family",
        ));
    }
    let allowed_signers = hub_root.join(ALLOWED_SIGNERS);
    ensure_regular_file(&allowed_signers, "allowed signers")?;
    verify_hub(lock, hub_root, family_root, &allowed_signers)?;
    for member in &lock.member {
        let repo = family_root.join(&member.name);
        verify_member(member, &repo, family_root, &allowed_signers)?;
    }
    Ok(())
}

pub(crate) fn verify_hub(
    lock: &FamilyLock,
    hub_root: &Path,
    family_root: &Path,
    allowed_signers: &Path,
) -> Result<(), CoordError> {
    ensure_ordinary_checkout(hub_root, family_root, "bullet-farm")?;
    admit_repository_metadata(hub_root, None)?;
    family_lock::verify_hub_checkout(lock, hub_root, allowed_signers)?;
    verify_exact_worktree(hub_root)
}

pub(crate) fn verify_member(
    member: &LockedMember,
    repo: &Path,
    family_root: &Path,
    allowed_signers: &Path,
) -> Result<(), CoordError> {
    ensure_ordinary_checkout(repo, family_root, &member.name)?;
    admit_repository_metadata(repo, member.jeryu_url.as_deref())?;
    family_lock::verify_locked_checkout(member, repo, allowed_signers)?;
    verify_exact_worktree(repo)
}

pub(crate) fn ensure_ordinary_checkout(
    repo: &Path,
    family_root: &Path,
    member: &str,
) -> Result<(), CoordError> {
    ensure_regular_directory(repo, member)?;
    ensure_regular_directory(&repo.join(".git"), &format!("{member}/.git"))?;
    let root = family_root.canonicalize().map_err(CoordError::io)?;
    let checkout = repo.canonicalize().map_err(CoordError::io)?;
    if checkout.parent() != Some(root.as_path()) {
        return Err(CoordError::new(
            "CHECKOUT_ESCAPE",
            format!(
                "{} does not resolve to a direct family child",
                repo.display()
            ),
        ));
    }
    Ok(())
}

pub(crate) fn ensure_regular_directory(path: &Path, label: &str) -> Result<(), CoordError> {
    let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(CoordError::new(
            "INVALID_CHECKOUT",
            format!("{label} must be a non-symlink directory"),
        ));
    }
    Ok(())
}

pub(crate) fn ensure_regular_file(path: &Path, label: &str) -> Result<(), CoordError> {
    let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(CoordError::new(
            "INVALID_CHECKOUT",
            format!("{label} must be a non-symlink regular file"),
        ));
    }
    Ok(())
}

fn is_hub(path: &Path) -> bool {
    path.join("Cargo.toml").is_file()
        && path.join("family.lock").is_file()
        && path.join("repos.manifest.toml").is_file()
}
