use std::{fs, path::Path, process::Command, time::Duration};

use crate::{coord::CoordError, process};

use super::{Result, require};

pub(super) fn command(repo: &Path) -> Command {
    let mut command = Command::new("/usr/bin/git");
    for secret in [
        "BULLET_PUBLICATION_TOKEN",
        "GH_TOKEN",
        "GITHUB_TOKEN",
        "GH_ENTERPRISE_TOKEN",
        "GITHUB_ENTERPRISE_TOKEN",
    ] {
        command.env_remove(secret);
    }
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    command
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("LC_ALL", "C")
        .args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
        ])
        .args([
            "-c",
            "protocol.ext.allow=never",
            "-c",
            "core.attributesFile=/dev/null",
        ])
        .args(["-c", "core.fsync=all", "-c", "core.fsyncMethod=fsync"])
        .arg("-C")
        .arg(repo);
    command
}

pub(super) fn execute(command: &mut Command, input: Option<&[u8]>) -> Result<Vec<u8>> {
    let limits = process::Limits {
        timeout: Duration::from_secs(300),
        stdout_bytes: 16 * 1024 * 1024,
        stderr_bytes: 1024 * 1024,
    };
    let result = if let Some(bytes) = input {
        use std::io::{Seek, Write};
        let mut file = tempfile::tempfile().map_err(CoordError::io)?;
        file.write_all(bytes).map_err(CoordError::io)?;
        file.rewind().map_err(CoordError::io)?;
        process::run_bounded_with_input_file(command, "publication git", limits, file)?.output
    } else {
        process::run_bounded(command, "publication git", limits)?
    };
    require(result.status.success(), "PUBLICATION_GIT_FAILED")?;
    Ok(result.stdout)
}

pub(super) fn bytes(repo: &Path, args: &[&str]) -> Result<Vec<u8>> {
    execute(command(repo).args(args), None)
}

pub(super) fn text(repo: &Path, args: &[&str]) -> Result<String> {
    String::from_utf8(bytes(repo, args)?)
        .map(|value| value.trim_end().to_owned())
        .map_err(|_| CoordError::new("PUBLICATION_GIT_OUTPUT_INVALID", "expected UTF-8"))
}

pub(super) fn canonical_directory(path: &Path) -> Result<()> {
    require(path.is_absolute(), "PUBLICATION_ROOT_NOT_ABSOLUTE")?;
    require(
        path.canonicalize().map_err(CoordError::io)? == path,
        "PUBLICATION_PATH_SUBSTITUTED",
    )?;
    require(path.is_dir(), "PUBLICATION_DIRECTORY_REQUIRED")
}

pub(super) fn checkout(path: &Path) -> Result<()> {
    canonical_directory(path)?;
    let metadata = path.join(".git");
    require(
        fs::symlink_metadata(&metadata)
            .map_err(CoordError::io)?
            .is_dir(),
        "PUBLICATION_ORDINARY_CHECKOUT_REQUIRED",
    )?;
    require(
        text(path, &["rev-parse", "--show-toplevel"])? == path.to_string_lossy(),
        "PUBLICATION_CHECKOUT_ROOT_MISMATCH",
    )?;
    metadata_admission(path, &metadata)?;
    let listing = text(path, &["ls-files", "-v"])?;
    require(
        listing.lines().all(|line| line.starts_with("H ")),
        "PUBLICATION_HIDDEN_INDEX_FLAGS",
    )?;
    require(
        !text(path, &["ls-tree", "-r", "HEAD"])?
            .lines()
            .any(|line| line.starts_with("160000 ")),
        "PUBLICATION_GITLINK_UNSUPPORTED",
    )?;
    require(
        bytes(path, &["status", "--porcelain=v1", "--untracked-files=all"])?.is_empty(),
        "PUBLICATION_DIRTY_CHECKOUT",
    )?;
    crate::checkout::verify_exact_worktree(path)
}

pub(super) fn absent(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        _ => Err(CoordError::new(
            "PUBLICATION_EXTERNAL_OBJECTS",
            "forbidden metadata exists or is unreadable",
        )),
    }
}

pub(super) fn metadata_admission(path: &Path, metadata: &Path) -> Result<()> {
    for file in ["config", "HEAD"] {
        require(
            fs::symlink_metadata(metadata.join(file))
                .map_err(CoordError::io)?
                .is_file(),
            "PUBLICATION_METADATA_SUBSTITUTED",
        )?;
    }
    require(
        text(path, &["rev-parse", "--is-shallow-repository"])? == "false",
        "PUBLICATION_SHALLOW_HISTORY",
    )?;
    for forbidden in [
        "objects/info/alternates",
        "objects/info/http-alternates",
        "info/grafts",
        "commondir",
        "gitdir",
        "shallow",
    ] {
        absent(&metadata.join(forbidden))?;
    }
    for directory in ["objects", "refs"] {
        canonical_directory(&metadata.join(directory))?;
    }
    let config = text(path, &["config", "--local", "--no-includes", "--list"])?;
    require(
        !config.lines().any(|line| {
            // Canonical source checkouts disable their retired push transport.
            // Preserve this exact denial; no protocol enablement is admitted.
            if line == "protocol.blocked-push.allow=never" {
                return false;
            }
            let key = line
                .split('=')
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase();
            [
                "include",
                "filter.",
                "url.",
                "credential.",
                "http.",
                "protocol.",
            ]
            .iter()
            .any(|prefix| key.starts_with(prefix))
                || [
                    "core.sshcommand",
                    "core.gitproxy",
                    "core.worktree",
                    "extensions.worktreeconfig",
                    "extensions.partialclone",
                ]
                .contains(&key.as_str())
                || key.ends_with(".promisor")
        }),
        "PUBLICATION_GIT_CONFIG_UNADMITTED",
    )?;
    require(
        text(
            path,
            &["for-each-ref", "--format=%(refname)", "refs/replace/"],
        )?
        .is_empty(),
        "PUBLICATION_REPLACEMENT_OBJECTS",
    )?;
    Ok(())
}

pub(super) fn blob(repo: &Path, revision: &str, path: &str) -> Result<Vec<u8>> {
    let listing = text(repo, &["ls-tree", revision, "--", path])?;
    require(
        listing.starts_with("100644 blob ")
            && listing.ends_with(&format!("\t{path}"))
            && listing.lines().count() == 1,
        "PUBLICATION_REGULAR_TEMPLATE_REQUIRED",
    )?;
    bytes(repo, &["show", &format!("{revision}:{path}")])
}
