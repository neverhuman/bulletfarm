use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};

use super::{ADMITTED_GIT, CommandSpec, GIT_BIN, ToolIdentity};
use crate::coord::CoordError;

pub(super) fn require_locked_version(
    id: &str,
    expected: &str,
    actual: &str,
) -> Result<(), CoordError> {
    if expected == actual {
        Ok(())
    } else {
        Err(tool_error(
            "SETUP_TOOL_SUBJECT_MISMATCH",
            id,
            format!("signed version {expected} differs from sealed tool version {actual}"),
        ))
    }
}

pub(super) fn is_numeric_triplet(value: &str) -> bool {
    let mut parts = value.split('.');
    matches!(
        (parts.next(), parts.next(), parts.next(), parts.next()),
        (Some(major), Some(minor), Some(patch), None)
            if [major, minor, patch].into_iter().all(|part| {
                !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())
            })
    )
}

pub(super) fn is_version(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+' | b'_'))
        && value
            .split('.')
            .take(2)
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

pub(super) fn required_path<'a>(
    path: Option<&'a Path>,
    label: &str,
) -> Result<&'a Path, CoordError> {
    path.ok_or_else(|| {
        tool_error(
            "SETUP_TOOL_MISSING",
            label,
            "an explicit canonical absolute path is required",
        )
    })
}

pub(super) fn tool_error(code: &'static str, label: &str, detail: impl AsRef<str>) -> CoordError {
    CoordError::new(code, format!("{label}: {}", detail.as_ref()))
}

pub(super) fn probe_path(program: &Path) -> Result<OsString, CoordError> {
    let parent = program.parent().ok_or_else(|| {
        tool_error(
            "SETUP_TOOL_PATH_NOT_CANONICAL",
            "setup tool",
            "canonical program has no parent",
        )
    })?;
    std::env::join_paths([parent, Path::new("/usr/bin")])
        .map_err(|error| tool_error("SETUP_TOOL_PATH_INVALID", "setup tool", error.to_string()))
}

pub(super) fn trusted_path<'a>(
    commands: impl IntoIterator<Item = &'a CommandSpec>,
) -> Result<OsString, CoordError> {
    let mut paths = Vec::new();
    for command in commands {
        if let Some(parent) = command.program.path.parent()
            && !paths.iter().any(|path| path == parent)
        {
            paths.push(parent.to_path_buf());
        }
    }
    let system = PathBuf::from("/usr/bin");
    if !paths.contains(&system) {
        paths.push(system);
    }
    std::env::join_paths(paths).map_err(|error| {
        tool_error(
            "SETUP_TOOL_PATH_INVALID",
            "setup toolchain",
            error.to_string(),
        )
    })
}

pub(crate) fn run_git(repo: Option<&Path>, args: &[&OsStr]) -> Result<(), CoordError> {
    admitted_git()?.run_git(repo, args)
}

pub(super) fn admitted_git() -> Result<&'static CommandSpec, CoordError> {
    if let Some(git) = ADMITTED_GIT.get() {
        return Ok(git);
    }
    let canonical = fs::canonicalize(GIT_BIN).map_err(|error| {
        tool_error(
            "SETUP_TOOL_UNAVAILABLE",
            "Git setup operation",
            format!("{GIT_BIN} cannot be resolved: {error}"),
        )
    })?;
    let candidate = CommandSpec::admit(ToolIdentity::Git, &canonical, Vec::new(), Vec::new())?;
    let _ = ADMITTED_GIT.set(candidate);
    Ok(ADMITTED_GIT
        .get()
        .expect("Git command is initialized after successful admission"))
}
