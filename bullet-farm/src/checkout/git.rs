//! Read-only Git metadata and exact working-tree verification.

#[cfg(test)]
mod tests;

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{checkout::ensure_regular_file, coord::CoordError, process::Limits};

const MAX_GIT_CONFIG_BYTES: u64 = 64 * 1024;
const GIT_LIMITS: Limits = Limits {
    timeout: Duration::from_secs(120),
    stdout_bytes: 16 * 1024 * 1024,
    stderr_bytes: 16 * 1024 * 1024,
};
const MAX_ADMIN_ENTRIES: usize = 1_000_000;
const MAX_TRACKED_ENTRIES: usize = 1_000_000;

pub(crate) fn admit_repository_metadata(
    repo: &Path,
    expected_origin: Option<&str>,
) -> Result<(), CoordError> {
    let git_dir = repo.join(".git");
    let config = git_dir.join("config");
    ensure_regular_file(&config, "repository-local Git config")?;
    if fs::symlink_metadata(&config).map_err(CoordError::io)?.len() > MAX_GIT_CONFIG_BYTES {
        return Err(unsafe_git("repository-local Git config exceeds 64 KiB"));
    }
    for path in [git_dir.join("HEAD"), git_dir.join("index")] {
        ensure_regular_file(&path, "critical Git administrative file")?;
    }
    reject_admin_symlinks(&git_dir)?;
    for path in [
        git_dir.join("objects/info/alternates"),
        git_dir.join("objects/info/http-alternates"),
        git_dir.join("commondir"),
        git_dir.join("gitdir"),
    ] {
        match fs::symlink_metadata(&path) {
            Ok(_) => {
                return Err(unsafe_git(format!(
                    "{} redirects the repository object authority",
                    path.display()
                )));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(CoordError::io(error)),
        }
    }
    reject_effective_info_excludes(&git_dir.join("info/exclude"))?;
    let entries = run(
        repo,
        &["config", "--local", "--no-includes", "--null", "--list"],
    )?;
    let mut keys = BTreeSet::new();
    let mut origin = None;
    for entry in entries
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
    {
        let entry = std::str::from_utf8(entry)
            .map_err(|_| unsafe_git("repository-local Git config is not UTF-8"))?;
        let (key, value) = entry
            .split_once('\n')
            .ok_or_else(|| unsafe_git("repository-local Git config entry is malformed"))?;
        let key = key.to_ascii_lowercase();
        if !keys.insert(key.clone()) {
            return Err(unsafe_git(format!(
                "repository-local Git config repeats {key}"
            )));
        }
        reject_dangerous_config(&key)?;
        if key == "remote.origin.url" {
            origin = Some(value.to_owned());
        }
    }
    if let Some(expected) = expected_origin
        && origin.as_deref() != Some(expected)
    {
        return Err(unsafe_git(
            "repository origin does not match the authenticated Jeryu source",
        ));
    }
    run(
        repo,
        &[
            "fsck",
            "--strict",
            "--full",
            "--no-reflogs",
            "--no-dangling",
            "--no-progress",
        ],
    )?;
    Ok(())
}

#[cfg(unix)]
pub(crate) fn verify_exact_worktree(repo: &Path) -> Result<(), CoordError> {
    use std::os::unix::fs::PermissionsExt;

    let listing = run(repo, &["ls-tree", "-r", "-z", "HEAD"])?;
    let mut paths = BTreeSet::new();
    let mut casefolded = BTreeSet::new();
    for (index, record) in listing
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
        .enumerate()
    {
        if index >= MAX_TRACKED_ENTRIES {
            return Err(dirty("tracked entry count exceeds 1,000,000"));
        }
        let record = std::str::from_utf8(record)
            .map_err(|_| dirty("the exact tree contains a non-UTF-8 path"))?;
        let (metadata, path) = record
            .split_once('\t')
            .ok_or_else(|| dirty("Git returned a malformed tree entry"))?;
        validate_path(path)?;
        if !paths.insert(path.to_owned()) || !casefolded.insert(path.to_ascii_lowercase()) {
            return Err(dirty(format!(
                "the exact tree has a path collision at {path}"
            )));
        }
        let mut fields = metadata.split_whitespace();
        let mode = fields.next().unwrap_or_default();
        let kind = fields.next().unwrap_or_default();
        let expected_oid = fields.next().unwrap_or_default();
        if fields.next().is_some() || kind != "blob" || !matches!(mode, "100644" | "100755") {
            return Err(dirty(format!(
                "{path} has unsupported mode/type {mode} {kind}; V1 install trees require regular files"
            )));
        }
        reject_worktree_parent_symlinks(repo, path)?;
        let worktree_path = repo.join(path);
        let actual = fs::symlink_metadata(&worktree_path)
            .map_err(|_| dirty(format!("tracked path {path} is missing or unreadable")))?;
        if !actual.file_type().is_file() || actual.file_type().is_symlink() {
            return Err(dirty(format!("tracked path {path} is not a regular file")));
        }
        let executable = actual.permissions().mode() & 0o111 != 0;
        if executable != (mode == "100755") {
            return Err(dirty(format!(
                "tracked path {path} has the wrong executable mode"
            )));
        }
        let actual_oid = run_text(repo, &["hash-object", "--no-filters", "--", path])?;
        if actual_oid.trim() != expected_oid {
            return Err(dirty(format!(
                "tracked path {path} differs from the exact tree"
            )));
        }
    }
    let untracked = run(repo, &["ls-files", "--others", "--exclude-standard", "-z"])?;
    if !untracked.is_empty() {
        return Err(dirty("the checkout contains nonignored untracked paths"));
    }
    Ok(())
}

fn reject_worktree_parent_symlinks(repo: &Path, path: &str) -> Result<(), CoordError> {
    let mut current = PathBuf::from(repo);
    let mut components = path.split('/').peekable();
    while let Some(component) = components.next() {
        if components.peek().is_none() {
            break;
        }
        current.push(component);
        let metadata = fs::symlink_metadata(&current)
            .map_err(|_| dirty(format!("tracked parent {} is missing", current.display())))?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
            return Err(dirty(format!(
                "tracked parent {} is not a non-symlink directory",
                current.display()
            )));
        }
    }
    Ok(())
}

#[cfg(not(unix))]
pub(crate) fn verify_exact_worktree(_repo: &Path) -> Result<(), CoordError> {
    Err(CoordError::new(
        "UNSUPPORTED_PLATFORM_CONTAINMENT",
        "exact working-tree byte/mode verification is unavailable on this platform",
    ))
}

/// Walk one member's Git administrative tree.
///
/// Every I/O failure here is a refusal that NAMES the exact offending path, not
/// a bare `COORD_IO_FAILED`. A node this walk cannot read (mode-000 CI
/// quarantine directories are the observed case) could hide a symlink that
/// redirects administrative authority, so the member's admission stops
/// fail-closed at that node with `UNREADABLE_GIT_METADATA`. The caller reports
/// the member by name and keeps walking every other member; nothing about this
/// member is thereafter reported as safe or clean.
fn reject_admin_symlinks(root: &Path) -> Result<(), CoordError> {
    let metadata = fs::symlink_metadata(root).map_err(|error| unreadable_git(root, &error))?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(unsafe_git(format!(
            "{} is not a regular Git administrative directory",
            root.display()
        )));
    }
    let mut stack = vec![PathBuf::from(root)];
    let mut seen = 0usize;
    while let Some(directory) = stack.pop() {
        let listing =
            fs::read_dir(&directory).map_err(|error| unreadable_git(&directory, &error))?;
        for entry in listing {
            let entry = entry.map_err(|error| unreadable_git(&directory, &error))?;
            seen = seen.saturating_add(1);
            if seen > MAX_ADMIN_ENTRIES {
                return Err(unsafe_git(
                    "Git administrative tree exceeds 1,000,000 entries",
                ));
            }
            let path = entry.path();
            let metadata =
                fs::symlink_metadata(&path).map_err(|error| unreadable_git(&path, &error))?;
            if metadata.file_type().is_symlink() {
                return Err(unsafe_git(format!(
                    "{} redirects Git administrative authority",
                    path.display()
                )));
            }
            if metadata.file_type().is_dir() {
                stack.push(path);
            }
        }
    }
    Ok(())
}

fn reject_effective_info_excludes(path: &Path) -> Result<(), CoordError> {
    match fs::read_to_string(path) {
        Ok(text)
            if text
                .lines()
                .all(|line| line.trim().is_empty() || line.trim_start().starts_with('#')) =>
        {
            Ok(())
        }
        Ok(_) => Err(unsafe_git(
            "repository-local info/exclude may hide untracked paths",
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(CoordError::io(error)),
    }
}

fn reject_dangerous_config(key: &str) -> Result<(), CoordError> {
    let exact = matches!(
        key,
        "core.worktree"
            | "core.fsmonitor"
            | "core.attributesfile"
            | "core.excludesfile"
            | "core.sshcommand"
            | "core.gitproxy"
            | "core.pager"
            | "diff.external"
            | "extensions.worktreeconfig"
    );
    let executable_prefix = [
        "filter.",
        "include.",
        "includeif.",
        "alias.",
        "pager.",
        "fsck.",
    ]
    .iter()
    .any(|prefix| key.starts_with(prefix));
    let executable_suffix = (key.starts_with("diff.") && key.ends_with(".command"))
        || (key.starts_with("merge.") && key.ends_with(".driver"));
    let lazy_remote = (key.starts_with("extensions.") && key != "extensions.objectformat")
        || key.ends_with(".promisor")
        || key.ends_with(".partialclonefilter");
    if exact || executable_prefix || executable_suffix || lazy_remote {
        Err(unsafe_git(format!(
            "repository-local Git config key {key} is forbidden during verification"
        )))
    } else {
        Ok(())
    }
}

fn validate_path(path: &str) -> Result<(), CoordError> {
    if path.is_empty()
        || path.len() > 4096
        || !path.is_ascii()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path.bytes().any(|byte| byte.is_ascii_control())
        || path.split('/').any(|part| {
            part.is_empty() || matches!(part, "." | "..") || part.eq_ignore_ascii_case(".git")
        })
    {
        return Err(dirty(format!(
            "the exact tree contains unsafe path {path:?}"
        )));
    }
    Ok(())
}

fn run_text(repo: &Path, args: &[&str]) -> Result<String, CoordError> {
    String::from_utf8(run(repo, args)?)
        .map_err(|_| CoordError::new("INVALID_GIT_OUTPUT", "Git emitted non-UTF-8 metadata"))
}

fn run(repo: &Path, args: &[&str]) -> Result<Vec<u8>, CoordError> {
    let output = run_after_verify(repo, args, || Ok(()))?;
    if !output.status.success() {
        return Err(CoordError::new(
            "GIT_VERIFICATION_FAILED",
            format!("Git verification failed in {}", repo.display()),
        ));
    }
    Ok(output.stdout)
}

fn run_after_verify(
    repo: &Path,
    args: &[&str],
    after_verify: impl FnOnce() -> Result<(), CoordError>,
) -> Result<std::process::Output, CoordError> {
    let mut admitted_args = Vec::with_capacity(18 + args.len());
    admitted_args.extend_from_slice(&[
        "-c",
        "core.fsmonitor=false",
        "-c",
        "core.attributesFile=/dev/null",
        "-c",
        "core.excludesFile=/dev/null",
        "-c",
        "core.filemode=true",
        "-c",
        "core.symlinks=true",
        "-c",
        "core.ignorecase=false",
        "-c",
        "core.autocrlf=false",
        "-c",
        "core.untrackedCache=false",
    ]);
    admitted_args.extend_from_slice(args);
    crate::family_lock::run_admitted_git_after_verify(
        repo,
        &admitted_args,
        GIT_LIMITS,
        "Git checkout verification",
        after_verify,
    )
}

fn unsafe_git(reason: impl Into<String>) -> CoordError {
    CoordError::new("UNSAFE_GIT_METADATA", reason)
}

/// A node of a Git administrative tree that could not be read. Distinct from
/// `UNSAFE_GIT_METADATA` (proven hostile) and from `COORD_IO_FAILED` (an
/// anonymous I/O failure): this code says which member path defeated the walk,
/// and it is never success.
fn unreadable_git(path: &Path, error: &std::io::Error) -> CoordError {
    CoordError::new(
        "UNREADABLE_GIT_METADATA",
        format!(
            "{} could not be read while verifying Git administrative metadata ({error}); \
             an unreadable node is never treated as clean or safe",
            path.display()
        ),
    )
}

fn dirty(reason: impl Into<String>) -> CoordError {
    CoordError::new("DIRTY_CHECKOUT", reason)
}
