use std::{path::Path, process::Command, time::Duration};

use super::{CoordError, validate_path, validate_repo_name};
use crate::process::{Limits, run_bounded};

const GIT_BIN: &str = "/usr/bin/git";
const GIT_LIMITS: Limits = Limits {
    timeout: Duration::from_secs(120),
    stdout_bytes: 16 * 1024 * 1024,
    stderr_bytes: 16 * 1024 * 1024,
};

pub(super) fn commit_paths(
    family_root: &Path,
    repo: &str,
    commit_oid: &str,
) -> Result<Vec<String>, CoordError> {
    validate_repo_name(repo)?;
    let repo_root = family_root.join(repo);
    if !repo_root.join(".git").is_dir() {
        return Err(CoordError::new(
            "REPOSITORY_NOT_FOUND",
            format!("{} is not a Git repository", repo_root.display()),
        ));
    }
    git(
        &repo_root,
        &["cat-file", "-e", &format!("{commit_oid}^{{commit}}")],
    )?;
    let output = git(
        &repo_root,
        &[
            "diff-tree",
            "--root",
            "--no-commit-id",
            "--name-only",
            "-z",
            "-r",
            commit_oid,
        ],
    )?;
    if !output.is_empty() && output.last() != Some(&0) {
        return Err(CoordError::new(
            "INVALID_GIT_OUTPUT",
            "Git leaf-path output lacks its terminal NUL",
        ));
    }
    let mut actual = output
        .strip_suffix(&[0])
        .unwrap_or_default()
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| {
            let path = std::str::from_utf8(path).map_err(|_| {
                CoordError::new("INVALID_GIT_OUTPUT", "Git emitted a non-UTF-8 leaf path")
            })?;
            validate_path(path).map_err(|error| {
                CoordError::new(
                    "INVALID_GIT_OUTPUT",
                    format!("Git emitted an invalid leaf path: {error}"),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    actual.sort();
    actual.dedup();
    Ok(actual)
}

fn git(repo_root: &Path, args: &[&str]) -> Result<Vec<u8>, CoordError> {
    let output = run_bounded(
        Command::new(GIT_BIN)
            .arg("-C")
            .arg(repo_root)
            .args(args)
            .env_clear()
            .env("LC_ALL", "C"),
        "Git coordination receipt check",
        GIT_LIMITS,
    )?;
    if !output.status.success() {
        return Err(CoordError::new(
            "COMMIT_NOT_FOUND",
            format!(
                "Git could not resolve the receipted commit in {}",
                repo_root.display()
            ),
        ));
    }
    Ok(output.stdout)
}
