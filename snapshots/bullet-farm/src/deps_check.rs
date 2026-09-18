//! Reject path dependencies that escape their owning repository.

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;
use toml::Value;

use crate::{coord::CoordError, doctor::discover_hub};

const USAGE: &str = "usage: bullet-family [--root PATH] deps check";
const SKIP_DIRS: &[&str] = &[".git", "target", "node_modules", ".fusion"];

#[derive(Deserialize)]
struct Manifest {
    required_repos: Vec<String>,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Violation {
    manifest: PathBuf,
    dependency: String,
    path: String,
    reason: &'static str,
}

pub fn run(
    current_dir: &Path,
    explicit_root: Option<&str>,
    args: &[String],
) -> Result<String, CoordError> {
    if args != ["check"] {
        return Err(CoordError::new("USAGE", USAGE));
    }
    let hub = discover_hub(current_dir, explicit_root)?;
    let repos = repositories(&hub)?;
    let mut violations = Vec::new();
    for repo in &repos {
        scan_repository(repo, &mut violations)?;
    }
    violations.sort();
    if !violations.is_empty() {
        let detail = violations
            .iter()
            .map(|violation| {
                format!(
                    "{}: dependency {} path {:?} {}",
                    violation.manifest.display(),
                    violation.dependency,
                    violation.path,
                    violation.reason
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        return Err(CoordError::new("FORBIDDEN_PATH_DEPENDENCY", detail));
    }
    let scope = hub
        .parent()
        .filter(|parent| parent.join("repos.manifest.toml").is_file())
        .unwrap_or(&hub);
    Ok(format!("path-deps: ok ({})", scope.display()))
}

fn repositories(hub: &Path) -> Result<Vec<PathBuf>, CoordError> {
    let Some(family) = hub
        .parent()
        .filter(|parent| parent.join("repos.manifest.toml").is_file())
    else {
        return Ok(vec![hub.to_path_buf()]);
    };
    let text = fs::read_to_string(family.join("repos.manifest.toml")).map_err(CoordError::io)?;
    let manifest: Manifest = toml::from_str(&text)
        .map_err(|error| CoordError::new("INVALID_FAMILY_MANIFEST", error.to_string()))?;
    let mut repos = Vec::new();
    for name in manifest.required_repos {
        validate_repo_name(&name)?;
        let repo = family.join(&name);
        if !repo.join(".git").is_dir() {
            return Err(CoordError::new(
                "FAMILY_MEMBER_MISSING",
                format!("{} is not an ordinary Git checkout", repo.display()),
            ));
        }
        repos.push(repo);
    }
    repos.sort();
    Ok(repos)
}

fn scan_repository(repo: &Path, violations: &mut Vec<Violation>) -> Result<(), CoordError> {
    let repo = repo.canonicalize().map_err(CoordError::io)?;
    let mut manifests = Vec::new();
    collect_manifests(&repo, &mut manifests)?;
    manifests.sort();
    for manifest in manifests {
        let text = fs::read_to_string(&manifest).map_err(CoordError::io)?;
        let value: Value = toml::from_str(&text).map_err(|error| {
            CoordError::new(
                "INVALID_CARGO_MANIFEST",
                format!("{}: {error}", manifest.display()),
            )
        })?;
        inspect_tables(&repo, &manifest, &value, violations)?;
    }
    Ok(())
}

fn collect_manifests(directory: &Path, found: &mut Vec<PathBuf>) -> Result<(), CoordError> {
    if directory.join("Cargo.toml").is_file() {
        found.push(directory.join("Cargo.toml"));
    }
    let mut children = fs::read_dir(directory)
        .map_err(CoordError::io)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(CoordError::io)?;
    children.sort_by_key(std::fs::DirEntry::file_name);
    for child in children {
        let file_type = child.file_type().map_err(CoordError::io)?;
        let name = child.file_name();
        let name = name.to_string_lossy();
        if file_type.is_dir() && !SKIP_DIRS.contains(&name.as_ref()) {
            collect_manifests(&child.path(), found)?;
        }
    }
    Ok(())
}

fn inspect_tables(
    repo: &Path,
    manifest: &Path,
    value: &Value,
    violations: &mut Vec<Violation>,
) -> Result<(), CoordError> {
    let Some(table) = value.as_table() else {
        return Ok(());
    };
    for (key, nested) in table {
        if is_dependency_section(key)
            && let Some(dependencies) = nested.as_table()
        {
            for (name, specification) in dependencies {
                if let Some(path) = specification
                    .as_table()
                    .and_then(|fields| fields.get("path"))
                    .and_then(Value::as_str)
                    && let Some(reason) = path_violation(repo, manifest, path)?
                {
                    violations.push(Violation {
                        manifest: manifest.to_path_buf(),
                        dependency: name.clone(),
                        path: path.to_owned(),
                        reason,
                    });
                }
            }
        }
        inspect_tables(repo, manifest, nested, violations)?;
    }
    Ok(())
}

fn path_violation(
    repo: &Path,
    manifest: &Path,
    raw: &str,
) -> Result<Option<&'static str>, CoordError> {
    let path = Path::new(raw);
    if path.is_absolute()
        || raw.contains('\\')
        || raw.as_bytes().get(1).is_some_and(|byte| *byte == b':')
    {
        return Ok(Some("is absolute or platform-ambiguous"));
    }
    let parent = manifest.parent().ok_or_else(|| {
        CoordError::new(
            "INVALID_CARGO_MANIFEST",
            "Cargo.toml has no parent directory",
        )
    })?;
    let Some(normalized) = normalize(parent.join(path)) else {
        return Ok(Some("escapes the filesystem root"));
    };
    if !normalized.starts_with(repo) {
        return Ok(Some("escapes its repository"));
    }
    let mut existing = normalized.as_path();
    while !existing.exists() {
        existing = existing.parent().ok_or_else(|| {
            CoordError::new("INVALID_DEPENDENCY_PATH", "path has no existing ancestor")
        })?;
    }
    let target = existing.canonicalize().map_err(CoordError::io)?;
    if !target.starts_with(repo) {
        return Ok(Some("resolves through a symlink outside its repository"));
    }
    Ok(None)
}

fn normalize(path: PathBuf) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
        }
    }
    Some(normalized)
}

fn is_dependency_section(key: &str) -> bool {
    matches!(
        key,
        "dependencies" | "dev-dependencies" | "build-dependencies"
    )
}

fn validate_repo_name(name: &str) -> Result<(), CoordError> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(CoordError::new(
            "INVALID_FAMILY_MANIFEST",
            format!("invalid repository name {name:?}"),
        ));
    }
    Ok(())
}
