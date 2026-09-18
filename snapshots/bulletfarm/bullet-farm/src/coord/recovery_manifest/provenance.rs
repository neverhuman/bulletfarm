use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Seek, SeekFrom, Write},
    path::Path,
    process::Command,
    time::Duration,
};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::RecoveryProvenanceCommand;
use crate::{
    coord::{CoordError, RecoveryBootstrapProvenanceV1},
    process::{Limits, run_bounded, run_bounded_with_input_file},
};

pub(super) mod archive;
mod tool;
const GIT_BIN: &str = "/usr/bin/git";
const GIT_LIMITS: Limits = Limits {
    timeout: Duration::from_secs(120),
    stdout_bytes: 16 * 1024 * 1024,
    stderr_bytes: 1024 * 1024,
};
const ARCHIVE_LIMITS: Limits = Limits {
    timeout: Duration::from_secs(120),
    stdout_bytes: archive::MAX_ARCHIVE_BYTES,
    stderr_bytes: 1024 * 1024,
};
const BLOB_LIMITS: Limits = Limits {
    timeout: Duration::from_secs(120),
    stdout_bytes: archive::MAX_ARCHIVE_BYTES + 2 * 1024 * 1024,
    stderr_bytes: 1024 * 1024,
};
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct TreeBlob {
    pub(super) mode: u32,
    pub(super) oid: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct RepositoryObservation {
    object_format: String,
    head_oid: String,
    tree_oid: String,
    tree_files: BTreeMap<String, TreeBlob>,
}
pub(super) fn produce(
    family_root: &Path,
    command: &RecoveryProvenanceCommand,
) -> Result<RecoveryBootstrapProvenanceV1, CoordError> {
    crate::coord::validate_commit_oid(&command.bootstrap_commit_oid)?;
    let checkout = family_root.join("bullet-farm");
    let output = tool::OutputGuard::admit(&command.output, &checkout)?;
    super::require_normalized_absolute(&command.source_archive_output, "source archive output")?;
    if command.source_archive_output == command.output
        || command.source_archive_output.starts_with(&checkout)
        || command.source_archive_output.parent() != Some(output.parent_path())
    {
        return Err(invalid(
            "source archive output must be a distinct sibling outside the family checkout",
        ));
    }
    let self_executable = tool::SelfExecutable::observe()?;
    let mut git_tool = tool::RetainedTool::admit(Path::new(GIT_BIN), "git")?;
    let mut cargo = tool::RetainedTool::admit(&command.cargo_bin, "cargo")?;
    let mut rustc = tool::RetainedTool::admit(&command.rustc_bin, "rustc")?;
    let repository = crate::coord::git::verify_repository(family_root, "bullet-farm")?;
    let before = observe_repository(&mut git_tool, &checkout, &command.bootstrap_commit_oid)?;
    let first_archive = git(
        &mut git_tool,
        &checkout,
        &[
            "archive".to_owned(),
            "--format=tar".to_owned(),
            command.bootstrap_commit_oid.clone(),
        ],
        "recovery provenance Git archive",
        ARCHIVE_LIMITS,
    )?;
    let second_archive = git(
        &mut git_tool,
        &checkout,
        &[
            "archive".to_owned(),
            "--format=tar".to_owned(),
            command.bootstrap_commit_oid.clone(),
        ],
        "recovery provenance repeated Git archive",
        ARCHIVE_LIMITS,
    )?;
    if first_archive != second_archive {
        return Err(changed(
            "Git archive bytes changed across independent bounded runs",
        ));
    }
    drop(second_archive);
    let archive = archive::inspect(
        &first_archive,
        &command.bootstrap_commit_oid,
        &before.tree_files,
    )?;
    let archive_sha256 = format!("sha256:{:x}", Sha256::digest(&first_archive));
    let cargo_version = tool::probe_version(&mut cargo, "cargo")?;
    let rustc_version = tool::probe_version(&mut rustc, "rustc")?;
    let rust_toolchain_channel = parse_toolchain_channel(&archive.rust_toolchain)?;
    if cargo_version.host != rustc_version.host
        || cargo_version.release != rustc_version.release
        || rustc_version.release != rust_toolchain_channel
    {
        return Err(invalid(
            "Cargo/Rustc host and release must agree with rust-toolchain.toml",
        ));
    }
    let provenance = RecoveryBootstrapProvenanceV1::from_observations(
        before.head_oid.clone(),
        before.tree_oid.clone(),
        archive_sha256,
        archive.cargo_lock_sha256.clone(),
        archive.source_files.clone(),
        (rustc_version.rendered, cargo_version.rendered),
        self_executable.facts(),
    )?;
    preflight_document(&provenance)?;
    let mut blob_input = tempfile::tempfile_in(output.parent_path()).map_err(CoordError::io)?;
    for blob in before.tree_files.values() {
        writeln!(blob_input, "{}", blob.oid).map_err(CoordError::io)?;
    }
    blob_input.flush().map_err(CoordError::io)?;
    blob_input
        .seek(SeekFrom::Start(0))
        .map_err(CoordError::io)?;
    let blob_batch = git_with_input(
        &mut git_tool,
        &checkout,
        &["cat-file".to_owned(), "--batch".to_owned()],
        "recovery provenance Git blob batch",
        BLOB_LIMITS,
        blob_input,
    )?;
    archive::verify_blob_batch(&blob_batch, &before.tree_files, &archive.source_files)?;
    drop(blob_batch);
    drop(archive);
    let after = observe_repository(&mut git_tool, &checkout, &command.bootstrap_commit_oid)?;
    if before != after {
        return Err(changed(
            "Git HEAD, tree, object format, or inventory changed during provenance production",
        ));
    }
    repository.revalidate(family_root)?;
    git_tool.revalidate()?;
    cargo.revalidate()?;
    rustc.revalidate()?;
    self_executable.revalidate()?;
    output.revalidate_absent()?;
    if let Err(write_error) = crate::coord::sealed::write_raw(
        &command.source_archive_output,
        &first_archive,
        archive::MAX_ARCHIVE_BYTES as u64,
    ) {
        let existing = crate::coord::sealed::read_raw(
            &command.source_archive_output,
            archive::MAX_ARCHIVE_BYTES as u64,
        )
        .map_err(|_| write_error)?;
        if existing != first_archive {
            return Err(changed(
                "existing source archive differs from regenerated bytes",
            ));
        }
    }
    output.revalidate_absent()?;
    crate::coord::sealed::write(output.path(), &provenance)?;
    output.verify_published()?;
    Ok(provenance)
}

fn observe_repository(
    git_tool: &mut tool::RetainedTool,
    checkout: &Path,
    expected_commit: &str,
) -> Result<RepositoryObservation, CoordError> {
    reject_info_attributes(checkout)?;
    let status = git(
        git_tool,
        checkout,
        &[
            "status".to_owned(),
            "--porcelain=v2".to_owned(),
            "-z".to_owned(),
            "--untracked-files=all".to_owned(),
        ],
        "recovery provenance Git status",
        GIT_LIMITS,
    )?;
    if !status.is_empty() {
        return Err(CoordError::new(
            "DIRTY_SOURCE",
            "bullet-farm has tracked, staged, or untracked changes",
        ));
    }
    let replacements = git(
        git_tool,
        checkout,
        &["for-each-ref".to_owned(), "refs/replace".to_owned()],
        "recovery provenance Git replacement-ref check",
        GIT_LIMITS,
    )?;
    if !replacements.is_empty() {
        return Err(invalid("Git replacement refs are not admitted"));
    }
    let object_format = git_line(
        git_tool,
        checkout,
        &["rev-parse".to_owned(), "--show-object-format".to_owned()],
        "recovery provenance Git object format",
    )?;
    let object_width = match object_format.as_str() {
        "sha1" => 40,
        "sha256" => 64,
        _ => return Err(invalid("Git repository uses an unsupported object format")),
    };
    if expected_commit.len() != object_width {
        return Err(invalid(
            "bootstrap commit width differs from the repository object format",
        ));
    }
    let head_oid = git_line(
        git_tool,
        checkout,
        &[
            "rev-parse".to_owned(),
            "--verify".to_owned(),
            "HEAD^{commit}".to_owned(),
        ],
        "recovery provenance Git HEAD",
    )?;
    validate_oid(&head_oid, object_width)?;
    if head_oid != expected_commit {
        return Err(invalid(
            "bootstrap commit must equal the clean checkout's exact HEAD",
        ));
    }
    let commit_spec = format!("{expected_commit}^{{commit}}");
    let existence = git(
        git_tool,
        checkout,
        &["cat-file".to_owned(), "-e".to_owned(), commit_spec],
        "recovery provenance Git commit existence",
        GIT_LIMITS,
    )?;
    if !existence.is_empty() {
        return Err(invalid("Git commit existence probe emitted output"));
    }
    let tree_oid = git_line(
        git_tool,
        checkout,
        &[
            "rev-parse".to_owned(),
            "--verify".to_owned(),
            format!("{expected_commit}^{{tree}}"),
        ],
        "recovery provenance Git tree",
    )?;
    validate_oid(&tree_oid, object_width)?;
    let tree = git(
        git_tool,
        checkout,
        &[
            "ls-tree".to_owned(),
            "-rz".to_owned(),
            "--full-tree".to_owned(),
            expected_commit.to_owned(),
        ],
        "recovery provenance Git tree inventory",
        GIT_LIMITS,
    )?;
    Ok(RepositoryObservation {
        object_format,
        head_oid,
        tree_oid,
        tree_files: parse_tree(&tree, object_width)?,
    })
}

fn reject_info_attributes(checkout: &Path) -> Result<(), CoordError> {
    match fs::symlink_metadata(checkout.join(".git/info/attributes")) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(invalid(
            "ambient .git/info/attributes is not admitted for provenance production",
        )),
        Err(error) => Err(CoordError::io(error)),
    }
}

pub(super) fn parse_tree(
    bytes: &[u8],
    object_width: usize,
) -> Result<BTreeMap<String, TreeBlob>, CoordError> {
    if bytes.is_empty() || bytes.last() != Some(&0) {
        return Err(invalid("Git tree inventory lacks its terminal NUL"));
    }
    let mut files = BTreeMap::new();
    for record in bytes[..bytes.len() - 1].split(|byte| *byte == 0) {
        let tab = record
            .iter()
            .position(|byte| *byte == b'\t')
            .ok_or_else(|| invalid("Git tree inventory entry lacks its path separator"))?;
        let header = std::str::from_utf8(&record[..tab])
            .map_err(|_| invalid("Git tree inventory header is not UTF-8"))?;
        let fields = header.split(' ').collect::<Vec<_>>();
        if fields.len() != 3 || fields[1] != "blob" {
            return Err(invalid(
                "bootstrap tree contains a symlink, submodule, or non-blob entry",
            ));
        }
        let mode = match fields[0] {
            "100644" => 0o664,
            "100755" => 0o775,
            _ => return Err(invalid("bootstrap tree has an unadmitted blob mode")),
        };
        validate_oid(fields[2], object_width)?;
        let path = archive::repository_path(&record[tab + 1..], false)?;
        if files
            .insert(
                path,
                TreeBlob {
                    mode,
                    oid: fields[2].to_owned(),
                },
            )
            .is_some()
        {
            return Err(invalid("Git tree inventory repeats a repository path"));
        }
        if files.len() > 8_192 {
            return Err(invalid("bootstrap tree exceeds 8,192 source files"));
        }
    }
    Ok(files)
}

fn git_line(
    git_tool: &mut tool::RetainedTool,
    checkout: &Path,
    args: &[String],
    label: &str,
) -> Result<String, CoordError> {
    let bytes = git(git_tool, checkout, args, label, GIT_LIMITS)?;
    if bytes.len() < 2
        || bytes.last() != Some(&b'\n')
        || bytes[..bytes.len() - 1]
            .iter()
            .any(|byte| !byte.is_ascii() || byte.is_ascii_control())
    {
        return Err(invalid(format!(
            "{label} must emit one nonempty ASCII line"
        )));
    }
    String::from_utf8(bytes[..bytes.len() - 1].to_vec())
        .map_err(|_| invalid(format!("{label} is not UTF-8")))
}

fn git(
    git_tool: &mut tool::RetainedTool,
    checkout: &Path,
    args: &[String],
    label: &str,
    limits: Limits,
) -> Result<Vec<u8>, CoordError> {
    run_git(git_tool, checkout, args, label, limits, None)
}

fn git_with_input(
    git_tool: &mut tool::RetainedTool,
    checkout: &Path,
    args: &[String],
    label: &str,
    limits: Limits,
    input: File,
) -> Result<Vec<u8>, CoordError> {
    run_git(git_tool, checkout, args, label, limits, Some(input))
}

fn run_git(
    git_tool: &mut tool::RetainedTool,
    checkout: &Path,
    args: &[String],
    label: &str,
    limits: Limits,
    input: Option<File>,
) -> Result<Vec<u8>, CoordError> {
    git_tool.revalidate()?;
    let mut command = Command::new(git_tool.proc_path());
    command
        .arg("-C")
        .arg(checkout)
        .args([
            "--no-optional-locks",
            "--no-replace-objects",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.untrackedCache=false",
            "-c",
            "core.attributesFile=/dev/null",
            "-c",
            "core.excludesFile=/dev/null",
            "-c",
            "maintenance.auto=false",
            "-c",
            "gc.auto=0",
        ])
        .args(args)
        .env_clear()
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_ATTR_NOSYSTEM", "1")
        .env("GIT_NO_LAZY_FETCH", "1")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_TERMINAL_PROMPT", "0");
    let output = match input {
        Some(input) => run_bounded_with_input_file(&mut command, label, limits, input)?.output,
        None => run_bounded(&mut command, label, limits)?,
    };
    git_tool.revalidate()?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(invalid(format!(
            "{label} failed or emitted unexpected stderr"
        )));
    }
    Ok(output.stdout)
}

fn validate_oid(value: &str, width: usize) -> Result<(), CoordError> {
    if value.len() != width
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid("Git emitted a malformed object ID"));
    }
    Ok(())
}

#[derive(Deserialize)]
struct RustToolchainDocument {
    toolchain: RustToolchain,
}

#[derive(Deserialize)]
struct RustToolchain {
    channel: String,
}

fn parse_toolchain_channel(bytes: &[u8]) -> Result<String, CoordError> {
    let text =
        std::str::from_utf8(bytes).map_err(|_| invalid("rust-toolchain.toml is not UTF-8"))?;
    let document: RustToolchainDocument = toml::from_str(text)
        .map_err(|error| invalid(format!("invalid rust-toolchain.toml: {error}")))?;
    let channel = document.toolchain.channel;
    if channel.is_empty()
        || channel.len() > 128
        || !channel.is_ascii()
        || channel.chars().any(char::is_control)
    {
        return Err(invalid("rust-toolchain.toml channel is malformed"));
    }
    Ok(channel)
}

fn preflight_document(value: &RecoveryBootstrapProvenanceV1) -> Result<(), CoordError> {
    let bytes = bullet_wire::canonical_json(value)
        .map_err(|error| invalid(format!("cannot preflight provenance document: {error}")))?;
    if bytes.len() > bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES {
        return Err(invalid(
            "recovery provenance document exceeds the sealed canonical byte bound",
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn test_preflight_document(
    value: &RecoveryBootstrapProvenanceV1,
) -> Result<(), CoordError> {
    preflight_document(value)
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_PRODUCTION", reason)
}

fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_SUBJECT_CHANGED", reason)
}
