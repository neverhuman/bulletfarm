//! Exact Git subjects, family membership, and pinned build toolchain.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde::Deserialize;

use super::{admitted_absolute_dir, invalid};
use crate::{
    coord::CoordError,
    process::{Limits, run_bounded},
};

const GIT_BIN: &str = "/usr/bin/git";
const TOOL_LIMITS: Limits = Limits {
    timeout: std::time::Duration::from_secs(120),
    stdout_bytes: 4 * 1024 * 1024,
    stderr_bytes: 4 * 1024 * 1024,
};
const MAX_LOCK_BYTES: u64 = 1024 * 1024;

/// One clean family member bound to its exact committed subject.
#[derive(Clone, Debug)]
pub(super) struct MemberSubject {
    pub(super) name: String,
    pub(super) path: PathBuf,
    pub(super) commit_oid: String,
    pub(super) tree_oid: String,
}

/// The exact executables and versions this build ran.
#[derive(Clone, Debug)]
pub(super) struct Toolchain {
    pub(super) git: PathBuf,
    pub(super) git_version: String,
    pub(super) cargo: PathBuf,
    pub(super) cargo_version: String,
    pub(super) rustc: PathBuf,
    pub(super) rustc_version: String,
    pub(super) node: PathBuf,
    pub(super) node_version: String,
    pub(super) npm: PathBuf,
    pub(super) npm_version: String,
}

pub(super) struct LockSubject {
    pub(super) schema_version: String,
    pub(super) tag: String,
    pub(super) release_signing_identity: String,
}

#[derive(Deserialize)]
struct LockProbe {
    schema_version: String,
    family: String,
    tag: String,
    member: Vec<LockProbeMember>,
}

#[derive(Deserialize)]
struct LockProbeMember {
    release_signing_identity: String,
}

#[derive(Deserialize)]
struct ManifestProbe {
    required_repos: Vec<String>,
}

pub(super) fn admit_toolchain() -> Result<Toolchain, CoordError> {
    let git = admitted_executable(Path::new(GIT_BIN), "git")?;
    let cargo = resolve("cargo")?;
    let rustc = resolve("rustc")?;
    let node = resolve("node")?;
    let npm = resolve("npm")?;
    Ok(Toolchain {
        git_version: version(&git, &["--version"], "git")?,
        cargo_version: version(&cargo, &["-V"], "cargo")?,
        rustc_version: version(&rustc, &["-V"], "rustc")?,
        node_version: version(&node, &["--version"], "node")?,
        npm_version: version(&npm, &["--version"], "npm")?,
        git,
        cargo,
        rustc,
        node,
        npm,
    })
}

pub(super) fn admit_family(
    family_root: &Path,
    tools: &Toolchain,
) -> Result<Vec<MemberSubject>, CoordError> {
    let manifest_path = family_root.join("repos.manifest.toml");
    let bytes = bounded_read(&manifest_path, MAX_LOCK_BYTES)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| invalid("repos.manifest.toml must contain valid UTF-8"))?;
    let manifest: ManifestProbe = toml::from_str(text)
        .map_err(|error| invalid(format!("invalid repos.manifest.toml: {error}")))?;
    if manifest.required_repos.is_empty() || manifest.required_repos.len() > 16 {
        return Err(invalid(
            "repos.manifest.toml must name between one and sixteen required repositories",
        ));
    }
    let mut names = manifest.required_repos.clone();
    names.sort();
    names.dedup();
    if names.len() != manifest.required_repos.len() {
        return Err(invalid("repos.manifest.toml repeats a required repository"));
    }
    if !names.iter().any(|name| name == "bullet-farm") {
        return Err(invalid("repos.manifest.toml omits the bullet-farm hub"));
    }
    names
        .into_iter()
        .map(|name| admit_member(family_root, &name, tools))
        .collect()
}

fn admit_member(
    family_root: &Path,
    name: &str,
    tools: &Toolchain,
) -> Result<MemberSubject, CoordError> {
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(invalid(format!("family member name {name:?} is malformed")));
    }
    let path = admitted_absolute_dir(&family_root.join(name), name)?;
    let git_dir = path.join(".git");
    let metadata = fs::symlink_metadata(&git_dir).map_err(CoordError::io)?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(invalid(format!(
            "{name} must be an ordinary checkout with a real .git directory, never a linked worktree"
        )));
    }
    let top = git(tools, &path, &["rev-parse", "--show-toplevel"])?;
    if Path::new(&top) != path {
        return Err(invalid(format!("{name} resolves to Git top-level {top}")));
    }
    let algorithm = git(tools, &path, &["rev-parse", "--show-object-format"])?;
    if !matches!(algorithm.as_str(), "sha1" | "sha256") {
        return Err(invalid(format!("{name} uses object format {algorithm:?}")));
    }
    let status = git_bytes(
        tools,
        &path,
        &["status", "--porcelain=v2", "-z", "--untracked-files=all"],
    )?;
    if !status.is_empty() {
        return Err(CoordError::new(
            "DIRTY_SOURCE",
            format!(
                "{name} has tracked, untracked, or index changes; a release archive is only ever \
                 built from an exact committed subject"
            ),
        ));
    }
    let commit = git(tools, &path, &["rev-parse", "--verify", "HEAD^{commit}"])?;
    let tree = git(tools, &path, &["rev-parse", "--verify", "HEAD^{tree}"])?;
    let width = if algorithm == "sha1" { 40 } else { 64 };
    for oid in [&commit, &tree] {
        if oid.len() != width || !oid.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(invalid(format!("{name} returned a malformed Git OID")));
        }
    }
    Ok(MemberSubject {
        name: name.to_owned(),
        path,
        commit_oid: format!("{algorithm}:{commit}"),
        tree_oid: format!("{algorithm}:{tree}"),
    })
}

pub(super) fn read_lock(path: &Path) -> Result<LockSubject, CoordError> {
    let bytes = bounded_read(path, MAX_LOCK_BYTES)?;
    let text =
        std::str::from_utf8(&bytes).map_err(|_| invalid("family.lock must contain valid UTF-8"))?;
    let lock: LockProbe =
        toml::from_str(text).map_err(|error| invalid(format!("invalid family.lock: {error}")))?;
    if lock.family != "bullet-farm" {
        return Err(invalid("family.lock does not bind the bullet-farm family"));
    }
    crate::release::schema::validate_tag(&lock.tag)?;
    let mut identities = lock
        .member
        .iter()
        .map(|member| member.release_signing_identity.as_str())
        .collect::<Vec<_>>();
    identities.sort_unstable();
    identities.dedup();
    match identities.as_slice() {
        [identity] => Ok(LockSubject {
            schema_version: lock.schema_version,
            tag: lock.tag,
            release_signing_identity: (*identity).to_owned(),
        }),
        _ => Err(invalid(
            "family.lock must name exactly one release signing identity for every member",
        )),
    }
}

pub(super) fn git(tools: &Toolchain, repo: &Path, args: &[&str]) -> Result<String, CoordError> {
    let bytes = git_bytes(tools, repo, args)?;
    let text =
        String::from_utf8(bytes).map_err(|_| invalid("Git emitted non-UTF-8 identity output"))?;
    Ok(text.trim().to_owned())
}

pub(super) fn git_bytes(
    tools: &Toolchain,
    repo: &Path,
    args: &[&str],
) -> Result<Vec<u8>, CoordError> {
    let output = run_bounded(
        Command::new(&tools.git)
            .arg("-C")
            .arg(repo)
            .args([
                "-c",
                "core.fsmonitor=false",
                "-c",
                "core.untrackedCache=false",
                "-c",
                "core.attributesFile=/dev/null",
                "-c",
                "core.excludesFile=/dev/null",
            ])
            .args(args)
            .env_clear()
            .env("LC_ALL", "C")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_TERMINAL_PROMPT", "0"),
        "release build Git",
        TOOL_LIMITS,
    )?;
    if !output.status.success() {
        return Err(super::failed(format!(
            "git {} exited {:?}",
            args.join(" "),
            output.status.code()
        )));
    }
    Ok(output.stdout)
}

fn resolve(name: &str) -> Result<PathBuf, CoordError> {
    let path = std::env::var_os("PATH").ok_or_else(|| missing(name, "PATH is unavailable"))?;
    for directory in std::env::split_paths(&path) {
        if !directory.is_absolute() {
            continue;
        }
        match admitted_executable(&directory.join(name), name) {
            Ok(executable) => return Ok(executable),
            Err(_) => continue,
        }
    }
    Err(missing(name, "not found as a regular file on PATH"))
}

/// Admits the exact pathname that will be executed. The pathname is deliberately
/// not canonicalized: `cargo` and `rustc` here are rustup shims that dispatch on
/// their own argv[0], so resolving them to `rustup` would silently run a
/// different program and ignore `rust-toolchain.toml`.
fn admitted_executable(path: &Path, name: &str) -> Result<PathBuf, CoordError> {
    if !path.is_absolute() {
        return Err(missing(name, "executable path must be absolute"));
    }
    let metadata = fs::metadata(path).map_err(|error| missing(name, error.to_string()))?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err(missing(
            name,
            "resolved subject is not a nonempty regular file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(missing(name, "resolved subject is not executable"));
        }
    }
    Ok(path.to_path_buf())
}

fn version(executable: &Path, args: &[&str], name: &str) -> Result<String, CoordError> {
    let mut command = Command::new(executable);
    command
        .args(args)
        .env_clear()
        .env("LC_ALL", "C")
        .env("PATH", "/usr/bin:/bin");
    // rustup and nvm shims resolve their real toolchain relative to these.
    for inherited in ["HOME", "CARGO_HOME", "RUSTUP_HOME", "RUSTUP_TOOLCHAIN"] {
        if let Some(value) = std::env::var_os(inherited) {
            command.env(inherited, value);
        }
    }
    let output = run_bounded(&mut command, name, TOOL_LIMITS)?;
    if !output.status.success() {
        return Err(missing(name, "version probe exited nonzero"));
    }
    let text = String::from_utf8(output.stdout)
        .map_err(|_| missing(name, "version output is not UTF-8"))?;
    let line = text.lines().next().unwrap_or_default().trim().to_owned();
    if line.is_empty() || line.len() > 256 || !line.is_ascii() {
        return Err(missing(
            name,
            "version output is not one bounded ASCII line",
        ));
    }
    Ok(line)
}

fn bounded_read(path: &Path, maximum: u64) -> Result<Vec<u8>, CoordError> {
    let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(invalid(format!(
            "{} must be a regular non-symlink file",
            path.display()
        )));
    }
    if metadata.len() == 0 || metadata.len() > maximum {
        return Err(invalid(format!(
            "{} is empty or exceeds its admission limit",
            path.display()
        )));
    }
    fs::read(path).map_err(CoordError::io)
}

fn missing(name: &str, reason: impl std::fmt::Display) -> CoordError {
    CoordError::new(
        "RELEASE_TOOLCHAIN_MISSING",
        format!("release build requires an exact {name}: {reason}"),
    )
}
