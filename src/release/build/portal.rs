//! Portal build from the exact committed subject, never from a tracked dist.

use std::{
    cmp::Ordering,
    collections::BTreeSet,
    ffi::OsString,
    fs::OpenOptions,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use serde::Deserialize;
use serde_json::Value;

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

use super::{BuildPlan, cargo::RecordedCommand, failed, invalid, subject::MemberSubject};
use crate::{
    coord::CoordError,
    process::{Limits, run_bounded},
};

const MANIFEST_NAME: &str = ".bullet-portal-bundle-v1.json";
const MAX_MANIFEST_BYTES: u64 = 2 * 1024 * 1024;
const MAX_PACKAGE_LOCK_BYTES: u64 = 16 * 1024 * 1024;
const MAX_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 64 * 1024 * 1024;
const MAX_FILES: usize = 2_048;
const MAX_TOOL_BYTES: u64 = 256 * 1024 * 1024;
const MAX_TOOL_TREE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_TOOL_TREE_FILES: u64 = 4_096;
const PORTAL_ROOT_DOMAIN: &[u8] = b"bullet.portal.bundle.root.v1\0";
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

fn validate_raw_tool_shapes(value: &Value) -> Result<(), CoordError> {
    let tools = value
        .get("tools")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                "Portal tool subjects must be a JSON array",
            )
        })?;
    let expected: [(&str, &[&str]); 3] = [
        ("git", &["blake3", "name", "size", "version"]),
        (
            "node",
            &[
                "architecture",
                "blake3",
                "name",
                "platform",
                "size",
                "version",
            ],
        ),
        ("npm", &["blake3", "file_count", "name", "size", "version"]),
    ];
    if tools.len() != expected.len() {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal tool subjects must contain exactly git, node, and npm",
        ));
    }
    for (index, tool) in tools.iter().enumerate() {
        let object = tool.as_object().ok_or_else(|| {
            CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                "Portal tool subject must be a JSON object",
            )
        })?;
        let (name, keys) = expected[index];
        let expected_keys = keys.iter().copied().collect::<BTreeSet<_>>();
        let actual_keys = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
        if object.get("name").and_then(Value::as_str) != Some(name) || actual_keys != expected_keys
        {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!("Portal {name} tool subject has producer-impossible keys"),
            ));
        }
    }
    Ok(())
}

fn manifest_root(mut value: Value) -> Result<String, CoordError> {
    let object = value.as_object_mut().ok_or_else(|| {
        CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal bundle manifest must be a JSON object",
        )
    })?;
    if object.remove("root").is_none() {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal bundle manifest root is missing",
        ));
    }
    let body = bullet_wire::canonical_json(&value).map_err(|error| {
        CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            format!("Portal bundle manifest body cannot be canonicalized: {error}"),
        )
    })?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(PORTAL_ROOT_DOMAIN);
    hasher.update(&body);
    Ok(format!("blake3:{}", hasher.finalize().to_hex()))
}

fn validate_manifest_semantics(manifest: &PortalManifest) -> Result<(), CoordError> {
    if manifest.schema_version != "bullet.portal.bundle.v1" {
        return Err(CoordError::new(
            "UNSUPPORTED_SCHEMA",
            format!(
                "Portal bundle manifest schema {} is unsupported",
                manifest.schema_version
            ),
        ));
    }
    if manifest.source.repository != "bullet-portal" {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal bundle manifest names the wrong source repository",
        ));
    }
    crate::release::schema::validate_oid("Portal source commit", &manifest.source.commit_oid)?;
    crate::release::schema::validate_oid("Portal source tree", &manifest.source.tree_oid)?;
    if manifest.package_lock.path != "package-lock.json"
        || manifest.package_lock.size > MAX_PACKAGE_LOCK_BYTES
    {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal package-lock subject is outside its producer contract",
        ));
    }
    for digest in std::iter::once(&manifest.root)
        .chain(std::iter::once(&manifest.package_lock.blake3))
        .chain(manifest.tools.iter().map(|tool| &tool.blake3))
        .chain(manifest.files.iter().map(|file| &file.blake3))
    {
        crate::release::schema::validate_digest(digest)?;
    }
    validate_tools(&manifest.tools)?;
    validate_file_inventory(&manifest.files, manifest.total_size)
}

fn validate_tools(tools: &[PortalTool]) -> Result<(), CoordError> {
    if tools.len() != 3
        || tools
            .iter()
            .map(|tool| tool.name.as_str())
            .ne(["git", "node", "npm"])
    {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal tool subjects must be ordered exactly as git, node, npm",
        ));
    }
    let git = &tools[0];
    let node = &tools[1];
    let npm = &tools[2];
    let git_valid = exact_numeric_version(&git.version, "git version ", 3, 4)
        && (1..=MAX_TOOL_BYTES).contains(&git.size)
        && git.platform.is_none()
        && git.architecture.is_none()
        && git.file_count.is_none();
    let node_valid = exact_numeric_version(&node.version, "v", 3, 3)
        && (1..=MAX_TOOL_BYTES).contains(&node.size)
        && node.platform.as_deref() == Some("linux")
        && node.architecture.as_deref() == Some("x64")
        && node.file_count.is_none();
    let npm_valid = exact_numeric_version(&npm.version, "", 3, 3)
        && (1..=MAX_TOOL_TREE_BYTES).contains(&npm.size)
        && npm.platform.is_none()
        && npm.architecture.is_none()
        && npm
            .file_count
            .is_some_and(|count| (1..=MAX_TOOL_TREE_FILES).contains(&count));
    if !git_valid || !node_valid || !npm_valid {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal tool subjects do not match the exact git/node/npm producer contracts",
        ));
    }
    Ok(())
}

fn exact_numeric_version(
    version: &str,
    prefix: &str,
    minimum_parts: usize,
    maximum_parts: usize,
) -> bool {
    if !(1..=160).contains(&version.len()) {
        return false;
    }
    let Some(number) = version.strip_prefix(prefix) else {
        return false;
    };
    let parts = number.split('.').collect::<Vec<_>>();
    (minimum_parts..=maximum_parts).contains(&parts.len())
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn validate_file_inventory(files: &[PortalFile], declared_total: u64) -> Result<(), CoordError> {
    if files.is_empty() || files.len() > MAX_FILES {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            format!("Portal bundle manifest must name 1..={MAX_FILES} files"),
        ));
    }
    let mut exact = BTreeSet::new();
    let mut portable = BTreeSet::new();
    let mut previous: Option<&str> = None;
    let mut index_count = 0_usize;
    let mut total = 0_u64;
    for file in files {
        let expected = admit_bundle_path(&file.path)?;
        if file.mime != expected {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!("{} has MIME {}, expected {expected}", file.path, file.mime),
            ));
        }
        if previous.is_some_and(|path| js_string_cmp(path, &file.path) != Ordering::Less) {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                "Portal bundle file records are not in strict producer path order",
            ));
        }
        previous = Some(&file.path);
        if !exact.insert(file.path.clone()) {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!("Portal bundle repeats exact path {}", file.path),
            ));
        }
        if !portable.insert(file.path.to_ascii_lowercase()) {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!(
                    "Portal bundle has a portable path collision at {}",
                    file.path
                ),
            ));
        }
        if file.path == "index.html" {
            index_count += 1;
        }
        if file.size > MAX_FILE_BYTES {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                format!("{} exceeds its producer file-size bound", file.path),
            ));
        }
        total = total
            .checked_add(file.size)
            .ok_or_else(|| invalid("Portal bundle byte total overflowed"))?;
        if total > MAX_TOTAL_BYTES {
            return Err(CoordError::new(
                "RELEASE_PORTAL_BUNDLE_INVALID",
                "Portal bundle exceeds its producer byte bound",
            ));
        }
    }
    if index_count != 1 {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal bundle must contain exactly one index.html",
        ));
    }
    if total != declared_total {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            "Portal bundle manifest total size differs from its file records",
        ));
    }
    Ok(())
}

fn admit_bundle_path(relative: &str) -> Result<&'static str, CoordError> {
    if relative.is_empty()
        || relative.len() > 240
        || relative.starts_with('/')
        || relative.contains(['\\', ':'])
        || relative
            .chars()
            .any(|character| character <= '\u{1f}' || character == '\u{7f}')
    {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            format!("unsafe Portal bundle path {relative:?}"),
        ));
    }
    let components = relative.split('/').collect::<Vec<_>>();
    if components.iter().any(|component| {
        component.is_empty()
            || matches!(*component, "." | "..")
            || component.starts_with('.')
            || component.ends_with(['.', ' '])
            || component.eq_ignore_ascii_case(".git")
    }) || (relative != "index.html" && !(components.len() == 2 && components[0] == "assets"))
    {
        return Err(CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            format!("unsafe or unexpected Portal bundle path {relative:?}"),
        ));
    }
    expected_mime(relative).ok_or_else(|| {
        CoordError::new(
            "RELEASE_PORTAL_BUNDLE_INVALID",
            format!("{relative} has an unsupported bundle media type"),
        )
    })
}

fn js_string_cmp(left: &str, right: &str) -> Ordering {
    left.encode_utf16().cmp(right.encode_utf16())
}

fn expected_mime(path: &str) -> Option<&'static str> {
    let extension = path.rsplit_once('.')?.1;
    match extension {
        "css" => Some("text/css; charset=utf-8"),
        "gif" => Some("image/gif"),
        "html" => Some("text/html; charset=utf-8"),
        "ico" => Some("image/x-icon"),
        "jpeg" | "jpg" => Some("image/jpeg"),
        "js" => Some("text/javascript; charset=utf-8"),
        "json" => Some("application/json"),
        "png" => Some("image/png"),
        "svg" => Some("image/svg+xml"),
        "txt" => Some("text/plain; charset=utf-8"),
        "webp" => Some("image/webp"),
        "woff" => Some("font/woff"),
        "woff2" => Some("font/woff2"),
        _ => None,
    }
}

fn npm_install_args(plan: &BuildPlan) -> Vec<String> {
    let mut args = vec![
        "ci".to_owned(),
        "--ignore-scripts".to_owned(),
        "--no-audit".to_owned(),
        "--no-fund".to_owned(),
    ];
    args.push(if plan.offline {
        "--offline".to_owned()
    } else {
        "--prefer-offline".to_owned()
    });
    args
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

#[cfg(test)]
mod tests {
    use super::{
        MANIFEST_NAME, MAX_FILE_BYTES, MAX_MANIFEST_BYTES, MAX_PACKAGE_LOCK_BYTES, MAX_TOOL_BYTES,
        MAX_TOOL_TREE_BYTES, decode_manifest, manifest_root, read_manifest, read_regular_bounded,
        read_regular_bounded_with_hooks,
    };

    fn manifest_value() -> serde_json::Value {
        serde_json::json!({
            "schema_version": "bullet.portal.bundle.v1",
            "source": {
                "repository": "bullet-portal",
                "commit_oid": "sha1:0000000000000000000000000000000000000000",
                "tree_oid": "sha1:1111111111111111111111111111111111111111"
            },
            "package_lock": {
                "path": "package-lock.json",
                "size": 1,
                "blake3": "blake3:0000000000000000000000000000000000000000000000000000000000000000"
            },
            "tools": [
                {"name":"git","version":"git version 2.44.0","size":1,"blake3":"blake3:2222222222222222222222222222222222222222222222222222222222222222"},
                {"name":"node","version":"v24.1.0","size":2,"blake3":"blake3:3333333333333333333333333333333333333333333333333333333333333333","platform":"linux","architecture":"x64"},
                {"name":"npm","version":"11.2.0","size":3,"blake3":"blake3:4444444444444444444444444444444444444444444444444444444444444444","file_count":1}
            ],
            "files": [{"path":"index.html","size":0,"mime":"text/html; charset=utf-8","blake3":"blake3:5555555555555555555555555555555555555555555555555555555555555555"}],
            "total_size": 0
        })
    }

    fn rooted_manifest(value: &serde_json::Value) -> serde_json::Value {
        let mut rooted = value.clone();
        rooted["root"] = serde_json::Value::String(format!("blake3:{}", "0".repeat(64)));
        let root = manifest_root(rooted.clone()).expect("manifest root");
        rooted["root"] = serde_json::Value::String(root);
        rooted
    }

    fn raw_manifest_bytes(value: &serde_json::Value) -> Vec<u8> {
        let mut bytes = bullet_wire::canonical_json(value).expect("canonical manifest");
        bytes.push(b'\n');
        bytes
    }

    fn manifest_bytes(value: &serde_json::Value) -> Vec<u8> {
        raw_manifest_bytes(&rooted_manifest(value))
    }

    fn file(path: &str, mime: &str) -> serde_json::Value {
        serde_json::json!({
            "path": path,
            "size": 0,
            "mime": mime,
            "blake3": "blake3:5555555555555555555555555555555555555555555555555555555555555555"
        })
    }

    #[test]
    fn portal_manifest_refuses_ambiguous_or_unsafe_projections() {
        let value = manifest_value();
        let valid = manifest_bytes(&value);
        let decoded = decode_manifest(&valid).expect("full canonical producer schema");
        assert_eq!(decoded.tools.len(), 3);
        assert_eq!(decoded.files[0].mime, "text/html; charset=utf-8");
        let mut unicode = value.clone();
        unicode["files"] = serde_json::json!([
            file("assets/\u{e9}.js", "text/javascript; charset=utf-8"),
            file("index.html", "text/html; charset=utf-8")
        ]);
        decode_manifest(&manifest_bytes(&unicode)).expect("NFC UTF-8 Portal path");
        let mut utf16_order = value.clone();
        utf16_order["files"] = serde_json::json!([
            file("assets/\u{1f600}.js", "text/javascript; charset=utf-8"),
            file("assets/\u{e000}.js", "text/javascript; charset=utf-8"),
            file("index.html", "text/html; charset=utf-8")
        ]);
        decode_manifest(&manifest_bytes(&utf16_order)).expect("producer UTF-16 path order");
        utf16_order["files"]
            .as_array_mut()
            .expect("file records")
            .swap(0, 1);
        assert!(
            decode_manifest(&manifest_bytes(&utf16_order)).is_err(),
            "Rust scalar ordering was admitted instead of producer UTF-16 ordering"
        );

        let mut wrong_root = rooted_manifest(&value);
        wrong_root["root"] = serde_json::Value::String(format!("blake3:{}", "f".repeat(64)));
        let error = decode_manifest(&raw_manifest_bytes(&wrong_root))
            .err()
            .expect("a root not binding the canonical body must fail closed");
        assert!(error.to_string().contains("root does not bind"));

        let canonical = String::from_utf8(valid.clone()).expect("UTF-8 manifest");
        let unsafe_size =
            canonical.replacen("\"total_size\":0", "\"total_size\":9007199254740992", 1);
        let error = decode_manifest(unsafe_size.as_bytes())
            .err()
            .expect("an unsafe Portal byte total must fail closed");
        assert!(error.to_string().contains("UNSAFE_JSON_INTEGER"));

        let duplicate =
            canonical.replacen("\"total_size\":0", "\"total_size\":0,\"total_size\":0", 1);
        let error = decode_manifest(duplicate.as_bytes())
            .err()
            .expect("a duplicate Portal manifest member must fail closed");
        assert!(error.to_string().contains("DUPLICATE_JSON_KEY"));

        let mut unknown = value.clone();
        unknown["unexpected"] = serde_json::Value::Bool(true);
        let error = decode_manifest(&manifest_bytes(&unknown))
            .err()
            .expect("unknown Portal manifest members must fail closed");
        assert!(error.to_string().contains("unknown field"));

        let mut pretty =
            serde_json::to_vec_pretty(&rooted_manifest(&value)).expect("pretty manifest");
        pretty.push(b'\n');
        assert!(
            decode_manifest(&pretty).is_err(),
            "noncanonical JSON was admitted"
        );
        assert!(
            decode_manifest(&valid[..valid.len() - 1]).is_err(),
            "a manifest without its one LF was admitted"
        );

        let mut hostile_subjects = Vec::new();
        let mut hostile = value.clone();
        hostile["tools"][0]["version"] = serde_json::Value::String("git 2.44".to_owned());
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][0]["version"] =
            serde_json::Value::String(format!("git version {}.1.1", "1".repeat(150)));
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][0]["size"] = serde_json::Value::from(MAX_TOOL_BYTES + 1);
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][0]["platform"] = serde_json::Value::String("linux".to_owned());
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][1]
            .as_object_mut()
            .expect("node subject")
            .remove("platform");
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][1]["architecture"] = serde_json::Value::String("arm64".to_owned());
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][1]["file_count"] = serde_json::Value::from(1);
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][2]["size"] = serde_json::Value::from(MAX_TOOL_TREE_BYTES + 1);
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][2]["file_count"] = serde_json::Value::from(4_097);
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][2]
            .as_object_mut()
            .expect("npm subject")
            .remove("file_count");
        hostile_subjects.push(hostile);
        let mut hostile = value.clone();
        hostile["tools"][2]["architecture"] = serde_json::Value::String("x64".to_owned());
        hostile_subjects.push(hostile);
        for (index, key) in [
            (0, "platform"),
            (0, "architecture"),
            (0, "file_count"),
            (1, "file_count"),
            (2, "platform"),
            (2, "architecture"),
            (1, "platform"),
            (1, "architecture"),
            (2, "file_count"),
        ] {
            let mut hostile = value.clone();
            hostile["tools"][index][key] = serde_json::Value::Null;
            hostile_subjects.push(hostile);
        }
        for hostile in hostile_subjects {
            assert!(
                decode_manifest(&manifest_bytes(&hostile)).is_err(),
                "an invalid tool subject was admitted"
            );
        }

        let mut hostile_inventories = Vec::new();
        let mut hostile = value.clone();
        hostile["files"] = serde_json::json!([
            file("index.html", "text/html; charset=utf-8"),
            file("index.html", "text/html; charset=utf-8")
        ]);
        hostile_inventories.push(hostile);
        let mut hostile = value.clone();
        hostile["files"] = serde_json::json!([
            file("assets/App.js", "text/javascript; charset=utf-8"),
            file("assets/app.js", "text/javascript; charset=utf-8"),
            file("index.html", "text/html; charset=utf-8")
        ]);
        hostile_inventories.push(hostile);
        let mut hostile = value.clone();
        hostile["files"] =
            serde_json::json!([file("assets/app.js", "text/javascript; charset=utf-8")]);
        hostile_inventories.push(hostile);
        let mut hostile = value.clone();
        hostile["files"] = serde_json::json!([
            file("assets/nested/app.js", "text/javascript; charset=utf-8"),
            file("index.html", "text/html; charset=utf-8")
        ]);
        hostile_inventories.push(hostile);
        for path in ["assets/bad:name.js", "assets/bad.", "assets/.GIT"] {
            let mut hostile = value.clone();
            hostile["files"] = serde_json::json!([
                file(path, "text/javascript; charset=utf-8"),
                file("index.html", "text/html; charset=utf-8")
            ]);
            hostile_inventories.push(hostile);
        }
        let mut hostile = value.clone();
        hostile["files"] = serde_json::json!([
            file("index.html", "text/html; charset=utf-8"),
            file("assets/app.js", "text/javascript; charset=utf-8")
        ]);
        hostile_inventories.push(hostile);
        let mut hostile = value.clone();
        hostile["files"] = serde_json::json!([
            file("assets/e\u{301}.js", "text/javascript; charset=utf-8"),
            file("index.html", "text/html; charset=utf-8")
        ]);
        hostile_inventories.push(hostile);
        let mut hostile = value.clone();
        hostile["files"][0]["mime"] = serde_json::Value::String("text/plain".to_owned());
        hostile_inventories.push(hostile);
        let mut hostile = value.clone();
        hostile["files"][0]["size"] = serde_json::Value::from(MAX_FILE_BYTES + 1);
        hostile["total_size"] = serde_json::Value::from(MAX_FILE_BYTES + 1);
        hostile_inventories.push(hostile);
        let mut hostile = value.clone();
        hostile["total_size"] = serde_json::Value::from(1);
        hostile_inventories.push(hostile);
        for hostile in hostile_inventories {
            assert!(
                decode_manifest(&manifest_bytes(&hostile)).is_err(),
                "an invalid Portal file inventory was admitted"
            );
        }

        for (pointer, hostile_value) in [
            ("/source/repository", serde_json::json!("other")),
            ("/source/commit_oid", serde_json::json!("deadbeef")),
            ("/package_lock/path", serde_json::json!("other-lock.json")),
            (
                "/package_lock/size",
                serde_json::json!(MAX_PACKAGE_LOCK_BYTES + 1),
            ),
        ] {
            let mut hostile = value.clone();
            *hostile.pointer_mut(pointer).expect("projection pointer") = hostile_value;
            assert!(
                decode_manifest(&manifest_bytes(&hostile)).is_err(),
                "an invalid release projection at {pointer} was admitted"
            );
        }

        let dist = tempfile::tempdir().expect("Portal dist");
        let oversized_manifest = dist.path().join(MANIFEST_NAME);
        std::fs::File::create(&oversized_manifest)
            .expect("oversized manifest")
            .set_len(MAX_MANIFEST_BYTES + 1)
            .expect("size oversized manifest");
        assert!(read_manifest(dist.path()).is_err());
        let oversized_lock = dist.path().join("package-lock.json");
        std::fs::File::create(&oversized_lock)
            .expect("oversized lock")
            .set_len(MAX_PACKAGE_LOCK_BYTES + 1)
            .expect("size oversized lock");
        assert!(
            read_regular_bounded(&oversized_lock, MAX_PACKAGE_LOCK_BYTES, "package-lock.json")
                .is_err()
        );

        #[cfg(unix)]
        {
            use std::{io::Write, os::unix::fs::symlink};

            let race = tempfile::tempdir().expect("race directory");
            let target = race.path().join("target");
            std::fs::write(&target, b"target").expect("race target");
            let swapped = race.path().join("swapped");
            std::fs::write(&swapped, b"original").expect("swap source");
            assert!(
                read_regular_bounded_with_hooks(
                    &swapped,
                    64,
                    "swapped input",
                    || {
                        std::fs::remove_file(&swapped).expect("remove swap source");
                        symlink(&target, &swapped).expect("replace source with symlink");
                    },
                    || {},
                )
                .is_err(),
                "a symlink swapped between lstat and open was admitted"
            );

            let mutated = race.path().join("mutated");
            std::fs::write(&mutated, b"before").expect("mutation source");
            assert!(
                read_regular_bounded_with_hooks(
                    &mutated,
                    64,
                    "mutated input",
                    || {},
                    || {
                        std::fs::OpenOptions::new()
                            .append(true)
                            .open(&mutated)
                            .expect("open mutation source")
                            .write_all(b"after")
                            .expect("mutate opened source");
                    },
                )
                .is_err(),
                "a file mutated after descriptor admission was accepted"
            );

            let replaced = race.path().join("replaced");
            let displaced = race.path().join("displaced");
            std::fs::write(&replaced, b"subject").expect("replacement source");
            assert!(
                read_regular_bounded_with_hooks(
                    &replaced,
                    64,
                    "replaced input",
                    || {},
                    || {
                        std::fs::rename(&replaced, &displaced).expect("displace opened path");
                        std::fs::write(&replaced, b"subject").expect("replace opened path");
                    },
                )
                .is_err(),
                "a pathname replaced after descriptor admission was accepted"
            );
        }
    }
}
