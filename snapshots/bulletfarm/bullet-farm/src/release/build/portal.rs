//! Portal build from the exact committed subject, never from a tracked dist.

use std::{
    ffi::OsString,
    fs::OpenOptions,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use serde::Deserialize;

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

use super::{BuildPlan, cargo::RecordedCommand, failed, invalid, subject::MemberSubject};
use crate::{
    coord::CoordError,
    process::{Limits, run_bounded},
};

#[path = "portal_validate.rs"]
mod portal_validate;
use portal_validate::{
    admit_bundle_path, manifest_root, npm_install_args, validate_file_inventory,
    validate_manifest_semantics, validate_raw_tool_shapes,
};

#[cfg(test)]
#[path = "portal_tests.rs"]
mod tests;

const MANIFEST_NAME: &str = ".bullet-portal-bundle-v1.json";
const MAX_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;
pub(super) const MAX_PACKAGE_LOCK_BYTES: u64 = 16 * 1024 * 1024;
pub(super) const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
pub(super) const MAX_TOTAL_BYTES: u64 = 64 * 1024 * 1024;
pub(super) const MAX_FILES: usize = 2_048;
pub(super) const MAX_TOOL_BYTES: u64 = 256 * 1024 * 1024;
pub(super) const MAX_TOOL_TREE_BYTES: u64 = 128 * 1024 * 1024;
pub(super) const MAX_TOOL_TREE_FILES: u64 = 4_096;
pub(super) const PORTAL_ROOT_DOMAIN: &[u8] = b"bullet.portal.bundle.root.v1\0";
const NPM_LIMITS: Limits = Limits {
    timeout: Duration::from_secs(3600),
    stdout_bytes: 32 * 1024 * 1024,
    stderr_bytes: 32 * 1024 * 1024,
};

/// The verified Portal bundle this build embedded.
pub(super) struct PortalOutput {
    pub(super) dist: PathBuf,
    pub(super) root: String,
    pub(super) manifest: PortalManifest,
    pub(super) package_lock: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PortalManifest {
    pub(super) schema_version: String,
    pub(super) source: PortalSource,
    pub(super) package_lock: PortalLock,
    pub(super) tools: Vec<PortalTool>,
    pub(super) files: Vec<PortalFile>,
    pub(super) total_size: u64,
    pub(super) root: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PortalSource {
    pub(super) repository: String,
    pub(super) commit_oid: String,
    pub(super) tree_oid: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PortalLock {
    pub(super) path: String,
    pub(super) size: u64,
    pub(super) blake3: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PortalTool {
    pub(super) name: String,
    pub(super) version: String,
    pub(super) size: u64,
    pub(super) blake3: String,
    pub(super) platform: Option<String>,
    pub(super) architecture: Option<String>,
    pub(super) file_count: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PortalFile {
    pub(super) path: String,
    pub(super) size: u64,
    pub(super) mime: String,
    pub(super) blake3: String,
}

/// Clones the committed Portal subject into the build scratch directory, builds
/// it there, and generates and re-checks its own bundle manifest. No tracked
/// checkout is written to and no committed `dist/` is ever created.
pub(super) fn build(
    plan: &BuildPlan,
    commands: &mut Vec<RecordedCommand>,
) -> Result<PortalOutput, CoordError> {
    let subject = plan.member("bullet-portal")?;
    let root = plan.scratch.join("bullet-portal");
    if std::fs::symlink_metadata(&root).is_ok() {
        return Err(CoordError::new(
            "RELEASE_OUTPUT_EXISTS",
            format!("{} already exists", root.display()),
        ));
    }
    clone(plan, subject, &root, commands)?;
    npm(plan, &root, &npm_install_args(plan), commands)?;
    npm(
        plan,
        &root,
        &["run".to_owned(), "build".to_owned()],
        commands,
    )?;
    npm(
        plan,
        &root,
        &["run".to_owned(), "bundle:generate".to_owned()],
        commands,
    )?;
    npm(
        plan,
        &root,
        &["run".to_owned(), "bundle:check".to_owned()],
        commands,
    )?;
    let dist = root.join("dist");
    let manifest = read_manifest(&dist)?;
    if manifest.source.repository != "bullet-portal"
        || manifest.source.commit_oid != subject.commit_oid
        || manifest.source.tree_oid != subject.tree_oid
    {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "the Portal bundle manifest binds a different Git subject than the admitted member",
        ));
    }
    verify_manifest_files(&dist, &manifest)?;
    let package_lock = read_regular_bounded(
        &root.join("package-lock.json"),
        MAX_PACKAGE_LOCK_BYTES,
        "package-lock.json",
    )?;
    if super::digest_bytes(&package_lock) != manifest.package_lock.blake3
        || package_lock.len() as u64 != manifest.package_lock.size
        || manifest.package_lock.path != "package-lock.json"
    {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "the Portal bundle manifest does not bind the exact package-lock.json bytes",
        ));
    }
    Ok(PortalOutput {
        dist,
        root: manifest.root.clone(),
        manifest,
        package_lock,
    })
}

/// Re-reads every emitted file and refuses any manifest that disagrees with the
/// bytes on disk, before those bytes reach the Rust embedding build script.
fn verify_manifest_files(dist: &Path, manifest: &PortalManifest) -> Result<(), CoordError> {
    validate_file_inventory(&manifest.files, manifest.total_size)?;
    let mut total = 0_u64;
    for file in &manifest.files {
        let path = admit_relative(dist, &file.path)?;
        let bytes = read_regular_bounded(&path, MAX_FILE_BYTES, &file.path)?;
        let size = bytes.len() as u64;
        if size != file.size || super::digest_bytes(&bytes) != file.blake3 {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!("{} differs from the Portal bundle manifest", file.path),
            ));
        }
        total = total
            .checked_add(size)
            .ok_or_else(|| invalid("Portal bundle byte total overflowed"))?;
        if total > MAX_TOTAL_BYTES {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                "the Portal bundle exceeds its producer byte bound",
            ));
        }
    }
    if total != manifest.total_size {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "the Portal bundle manifest total size differs from its own file records",
        ));
    }
    Ok(())
}

fn admit_relative(dist: &Path, relative: &str) -> Result<PathBuf, CoordError> {
    admit_bundle_path(relative)?;
    Ok(dist.join(relative))
}

fn read_manifest(dist: &Path) -> Result<PortalManifest, CoordError> {
    let path = dist.join(MANIFEST_NAME);
    let bytes = read_regular_bounded(&path, MAX_MANIFEST_BYTES, "Portal bundle manifest")?;
    decode_manifest(&bytes)
}

fn read_regular_bounded(path: &Path, max_bytes: u64, label: &str) -> Result<Vec<u8>, CoordError> {
    read_regular_bounded_with_hooks(path, max_bytes, label, || {}, || {})
}

#[cfg(unix)]
#[derive(Clone, Copy, Eq, PartialEq)]
struct FileIdentity {
    device: u64,
    inode: u64,
    length: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
}

#[cfg(unix)]
impl FileIdentity {
    fn from_metadata(metadata: &std::fs::Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            length: metadata.len(),
            modified_seconds: metadata.mtime(),
            modified_nanoseconds: metadata.mtime_nsec(),
            changed_seconds: metadata.ctime(),
            changed_nanoseconds: metadata.ctime_nsec(),
        }
    }
}

fn read_regular_bounded_with_hooks<BeforeOpen, AfterOpen>(
    path: &Path,
    max_bytes: u64,
    label: &str,
    before_open: BeforeOpen,
    after_open: AfterOpen,
) -> Result<Vec<u8>, CoordError>
where
    BeforeOpen: FnOnce(),
    AfterOpen: FnOnce(),
{
    #[cfg(not(unix))]
    {
        let _ = (path, max_bytes, before_open, after_open);
        return Err(CoordError::new(
            "RELEASE_PORTAL_PLATFORM_UNSUPPORTED",
            format!("{label} cannot be descriptor-pinned on this platform"),
        ));
    }
    #[cfg(unix)]
    {
        let path_metadata = std::fs::symlink_metadata(path).map_err(CoordError::io)?;
        if path_metadata.file_type().is_symlink()
            || !path_metadata.file_type().is_file()
            || path_metadata.len() > max_bytes
        {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!("{label} is missing or outside its {max_bytes}-byte bound"),
            ));
        }
        let path_identity = FileIdentity::from_metadata(&path_metadata);
        before_open();
        let mut file = OpenOptions::new()
            .read(true)
            .custom_flags(nix::libc::O_NOFOLLOW | nix::libc::O_NONBLOCK | nix::libc::O_CLOEXEC)
            .open(path)
            .map_err(CoordError::io)?;
        let opened = file.metadata().map_err(CoordError::io)?;
        let opened_identity = FileIdentity::from_metadata(&opened);
        if !opened.is_file() || opened_identity != path_identity {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!("{label} changed before its bounded read"),
            ));
        }
        after_open();
        let capacity = usize::try_from(opened.len().min(max_bytes))
            .map_err(|_| invalid(format!("{label} is too large for this platform")))?;
        let read_limit = max_bytes
            .checked_add(1)
            .ok_or_else(|| invalid(format!("{label} byte limit cannot be represented")))?;
        let mut bytes = Vec::with_capacity(capacity);
        file.by_ref()
            .take(read_limit)
            .read_to_end(&mut bytes)
            .map_err(CoordError::io)?;
        let after = file.metadata().map_err(CoordError::io)?;
        let path_after = std::fs::symlink_metadata(path).map_err(CoordError::io)?;
        if bytes.len() as u64 > max_bytes
            || bytes.len() as u64 != opened.len()
            || FileIdentity::from_metadata(&after) != opened_identity
            || path_after.file_type().is_symlink()
            || !path_after.file_type().is_file()
            || FileIdentity::from_metadata(&path_after) != opened_identity
        {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!("{label} changed during its bounded read"),
            ));
        }
        Ok(bytes)
    }
}

fn decode_manifest(bytes: &[u8]) -> Result<PortalManifest, CoordError> {
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal bundle manifest exceeds its producer byte bound",
        ));
    }
    let body = bytes.strip_suffix(b"\n").ok_or_else(|| {
        CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal bundle manifest must end in exactly one LF",
        )
    })?;
    let value = bullet_wire::decode_unique_value_bounded(body, MAX_MANIFEST_BYTES as usize)
        .map_err(|error| {
            CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!("Portal bundle manifest is not strict JSON: {error}"),
            )
        })?;
    let canonical = bullet_wire::canonical_json(&value).map_err(|error| {
        CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            format!("Portal bundle manifest cannot be canonicalized: {error}"),
        )
    })?;
    if body != canonical {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal bundle manifest is not canonical JSON plus one LF",
        ));
    }
    validate_raw_tool_shapes(&value)?;
    let manifest: PortalManifest = serde_json::from_value(value.clone()).map_err(|error| {
        CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            format!("Portal bundle manifest does not match its typed schema: {error}"),
        )
    })?;
    validate_manifest_semantics(&manifest)?;
    let expected_root = manifest_root(value)?;
    if manifest.root != expected_root {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal bundle manifest root does not bind its canonical body",
        ));
    }
    Ok(manifest)
}

fn clone(
    plan: &BuildPlan,
    subject: &MemberSubject,
    root: &Path,
    commands: &mut Vec<RecordedCommand>,
) -> Result<(), CoordError> {
    let source = subject
        .path
        .to_str()
        .ok_or_else(|| failed("the Portal checkout path is not UTF-8"))?;
    let destination = root
        .to_str()
        .ok_or_else(|| failed("the Portal scratch path is not UTF-8"))?;
    let commit = subject
        .commit_oid
        .split_once(':')
        .ok_or_else(|| invalid("the Portal commit OID is not algorithm-tagged"))?
        .1;
    super::subject::git_bytes(
        &plan.tools,
        &plan.family_root,
        &["clone", "--no-hardlinks", "--quiet", source, destination],
    )?;
    super::subject::git_bytes(
        &plan.tools,
        root,
        &["checkout", "--quiet", "--detach", commit],
    )?;
    for args in [
        vec![
            "clone".to_owned(),
            "--no-hardlinks".to_owned(),
            "--quiet".to_owned(),
            source.to_owned(),
            destination.to_owned(),
        ],
        vec![
            "checkout".to_owned(),
            "--quiet".to_owned(),
            "--detach".to_owned(),
            commit.to_owned(),
        ],
    ] {
        commands.push(RecordedCommand {
            program: plan.tools.git.display().to_string(),
            args,
            cwd: plan.family_root.display().to_string(),
            env: Vec::new(),
        });
    }
    Ok(())
}

fn npm(
    plan: &BuildPlan,
    root: &Path,
    args: &[String],
    commands: &mut Vec<RecordedCommand>,
) -> Result<(), CoordError> {
    let tools = &plan.tools;
    let node_bin = tools
        .node
        .parent()
        .ok_or_else(|| failed("the admitted node has no parent directory"))?;
    let mut path = OsString::from(node_bin);
    path.push(":/usr/bin:/bin");
    let mut env = vec![
        (
            "PATH".to_owned(),
            path.to_str()
                .ok_or_else(|| failed("the Portal build PATH is not UTF-8"))?
                .to_owned(),
        ),
        ("LC_ALL".to_owned(), "C".to_owned()),
        ("CI".to_owned(), "1".to_owned()),
        (
            "npm_config_cache".to_owned(),
            plan.cache
                .join("npm")
                .to_str()
                .ok_or_else(|| failed("the npm cache path is not UTF-8"))?
                .to_owned(),
        ),
    ];
    if let Some(home) = std::env::var_os("HOME").and_then(|value| value.into_string().ok()) {
        env.push(("HOME".to_owned(), home));
    }
    env.sort();
    let mut command = Command::new(&tools.npm);
    command.args(args).current_dir(root).env_clear();
    for (name, value) in &env {
        command.env(name, value);
    }
    commands.push(RecordedCommand {
        program: tools.npm.display().to_string(),
        args: args.to_vec(),
        cwd: root.display().to_string(),
        env: env.clone(),
    });
    let output = run_bounded(&mut command, "release build npm", NPM_LIMITS)?;
    if output.status.success() {
        return Ok(());
    }
    Err(CoordError::new(
        "RELEASE_PORTAL_BUNDLE_INVALID",
        format!(
            "npm {} exited {:?}: {}",
            args.join(" "),
            output.status.code(),
            super::cargo::tail(&[output.stderr.as_slice(), output.stdout.as_slice()].concat())
        ),
    ))
}
