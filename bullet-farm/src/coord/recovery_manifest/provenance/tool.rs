use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    os::{fd::AsRawFd, unix::fs::MetadataExt},
    path::{Path, PathBuf},
    process::{Command, Output},
};

use rustix::fs::{AtFlags, Mode, OFlags, ResolveFlags, open, openat, openat2, statat};
use sha2::{Digest, Sha256};

use crate::{
    coord::CoordError,
    process::{Limits, run_bounded},
};

const MAX_EXECUTABLE_BYTES: u64 = 512 * 1024 * 1024;
const VERSION_LIMITS: Limits = Limits {
    timeout: std::time::Duration::from_secs(10),
    stdout_bytes: 16 * 1024,
    stderr_bytes: 16 * 1024,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileIdentity {
    device: u64,
    inode: u64,
    owner_uid: u32,
    owner_gid: u32,
    mode: u32,
    links: u64,
    length: u64,
    mtime_seconds: i64,
    mtime_nanoseconds: i64,
    ctime_seconds: i64,
    ctime_nanoseconds: i64,
}

impl FileIdentity {
    fn read(file: &File) -> Result<Self, CoordError> {
        let value = file.metadata().map_err(CoordError::io)?;
        if !value.is_file() {
            return Err(invalid("executable subject is not a regular file"));
        }
        Ok(Self {
            device: value.dev(),
            inode: value.ino(),
            owner_uid: value.uid(),
            owner_gid: value.gid(),
            mode: value.mode(),
            links: value.nlink(),
            length: value.len(),
            mtime_seconds: value.mtime(),
            mtime_nanoseconds: value.mtime_nsec(),
            ctime_seconds: value.ctime(),
            ctime_nanoseconds: value.ctime_nsec(),
        })
    }

    fn require_bounded_elf(self, bytes: &[u8]) -> Result<(), CoordError> {
        if self.length == 0
            || self.length > MAX_EXECUTABLE_BYTES
            || self.mode & 0o111 == 0
            || !bytes.starts_with(b"\x7fELF")
        {
            return Err(invalid(
                "executable subject must be an executable ELF regular file within the 512 MiB bound",
            ));
        }
        Ok(())
    }

    fn require_tool_custody(self, label: &str) -> Result<(), CoordError> {
        let admitted_owner =
            matches!(self.owner_uid, 0) || self.owner_uid == rustix::process::geteuid().as_raw();
        if self.links != 1 || self.mode & 0o7022 != 0 || !admitted_owner {
            return Err(invalid(format!(
                "{label} must be one-link, root-or-exact-euid owned, non-privileged, and non-group/world-writable"
            )));
        }
        Ok(())
    }
}

pub(super) struct RetainedTool {
    label: &'static str,
    path: PathBuf,
    file: File,
    identity: FileIdentity,
    sha256: [u8; 32],
}

impl RetainedTool {
    pub(super) fn admit(path: &Path, label: &'static str) -> Result<Self, CoordError> {
        require_direct_absolute(path, label)?;
        let mut file = open_direct_file(path, label)?;
        let identity = FileIdentity::read(&file)?;
        let bytes = read_exact_bounded(&mut file, identity, MAX_EXECUTABLE_BYTES)?;
        identity.require_bounded_elf(&bytes)?;
        identity.require_tool_custody(label)?;
        Ok(Self {
            label,
            path: path.to_path_buf(),
            file,
            identity,
            sha256: Sha256::digest(&bytes).into(),
        })
    }

    pub(super) fn proc_path(&self) -> PathBuf {
        PathBuf::from(format!("/proc/self/fd/{}", self.file.as_raw_fd()))
    }

    pub(super) fn revalidate(&mut self) -> Result<(), CoordError> {
        let retained_identity = FileIdentity::read(&self.file)?;
        let retained = read_exact_bounded(&mut self.file, retained_identity, MAX_EXECUTABLE_BYTES)?;
        let mut reopened = open_direct_file(&self.path, self.label)?;
        let reopened_identity = FileIdentity::read(&reopened)?;
        let reopened_bytes =
            read_exact_bounded(&mut reopened, reopened_identity, MAX_EXECUTABLE_BYTES)?;
        if retained_identity != self.identity
            || reopened_identity != self.identity
            || Sha256::digest(&retained).as_slice() != self.sha256
            || Sha256::digest(&reopened_bytes).as_slice() != self.sha256
        {
            return Err(changed(format!(
                "{} executable changed across retained-descriptor read-back",
                self.label
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct VersionObservation {
    pub(super) rendered: String,
    pub(super) release: String,
    pub(super) host: String,
}

pub(super) fn probe_version(
    tool: &mut RetainedTool,
    expected_name: &'static str,
) -> Result<VersionObservation, CoordError> {
    let first = probe_once(tool, expected_name)?;
    let second = probe_once(tool, expected_name)?;
    if first != second {
        return Err(changed(format!(
            "{expected_name} -Vv changed across independent probes"
        )));
    }
    parse_version(&first, expected_name)
}

fn probe_once(tool: &mut RetainedTool, name: &str) -> Result<Vec<u8>, CoordError> {
    tool.revalidate()?;
    let output = run_bounded(
        Command::new(tool.proc_path())
            .arg("-Vv")
            .env_clear()
            .env("LC_ALL", "C")
            .env("LANG", "C"),
        &format!("recovery provenance {name} version probe"),
        VERSION_LIMITS,
    )?;
    tool.revalidate()?;
    require_success(output, name)
}

fn require_success(output: Output, name: &str) -> Result<Vec<u8>, CoordError> {
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(invalid(format!(
            "{name} -Vv must succeed with empty stderr"
        )));
    }
    Ok(output.stdout)
}

fn parse_version(bytes: &[u8], name: &str) -> Result<VersionObservation, CoordError> {
    if bytes.len() < 2
        || bytes.last() != Some(&b'\n')
        || bytes[bytes.len() - 2] == b'\n'
        || bytes[..bytes.len() - 1]
            .iter()
            .any(|byte| !byte.is_ascii() || (*byte != b'\n' && byte.is_ascii_control()))
    {
        return Err(invalid(format!(
            "{name} -Vv must be bounded ASCII with exactly one terminal LF"
        )));
    }
    let text = std::str::from_utf8(&bytes[..bytes.len() - 1])
        .map_err(|_| invalid(format!("{name} -Vv is not UTF-8")))?;
    let lines = text.split('\n').collect::<Vec<_>>();
    if lines.iter().any(|line| line.is_empty()) || !lines[0].starts_with(&format!("{name} ")) {
        return Err(invalid(format!("{name} -Vv has an invalid line shape")));
    }
    let release = unique_field(&lines, "release:", name)?;
    let host = unique_field(&lines, "host:", name)?;
    let rendered = lines.join(" | ");
    if rendered.len() > 1_024 {
        return Err(invalid(format!("{name} -Vv normalization is too large")));
    }
    Ok(VersionObservation {
        rendered,
        release,
        host,
    })
}

fn unique_field(lines: &[&str], prefix: &str, name: &str) -> Result<String, CoordError> {
    let values = lines
        .iter()
        .filter_map(|line| line.strip_prefix(prefix).map(str::trim))
        .collect::<Vec<_>>();
    match values.as_slice() {
        [value] if !value.is_empty() => Ok((*value).to_owned()),
        _ => Err(invalid(format!(
            "{name} -Vv must contain exactly one nonempty {prefix} field"
        ))),
    }
}

pub(super) struct SelfExecutable {
    identity: FileIdentity,
    length: u64,
    sha256: String,
}

impl SelfExecutable {
    pub(super) fn observe() -> Result<Self, CoordError> {
        let (bytes, identity) = read_self_executable()?;
        Ok(Self {
            identity,
            length: bytes.len() as u64,
            sha256: format!("{:x}", Sha256::digest(&bytes)),
        })
    }

    pub(super) fn revalidate(&self) -> Result<(), CoordError> {
        let (bytes, identity) = read_self_executable()?;
        if identity != self.identity
            || bytes.len() as u64 != self.length
            || format!("{:x}", Sha256::digest(&bytes)) != self.sha256
        {
            return Err(changed(
                "/proc/self/exe changed across provenance production",
            ));
        }
        Ok(())
    }

    pub(super) fn facts(&self) -> (u64, String) {
        (self.length, format!("sha256:{}", self.sha256))
    }
}

fn read_self_executable() -> Result<(Vec<u8>, FileIdentity), CoordError> {
    let mut file = File::open("/proc/self/exe").map_err(CoordError::io)?;
    let identity = FileIdentity::read(&file)?;
    let bytes = read_exact_bounded(&mut file, identity, MAX_EXECUTABLE_BYTES)?;
    identity.require_bounded_elf(&bytes)?;
    Ok((bytes, identity))
}

pub(super) struct OutputGuard {
    path: PathBuf,
    parent_path: PathBuf,
    parent: File,
    parent_identity: DirectoryIdentity,
    name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DirectoryIdentity {
    device: u64,
    inode: u64,
    owner_uid: u32,
    mode: u32,
}

impl OutputGuard {
    pub(super) fn admit(path: &Path, checkout: &Path) -> Result<Self, CoordError> {
        require_path_text(path, "provenance output")?;
        if path.starts_with(checkout) {
            return Err(invalid(
                "recovery provenance output must remain outside the family checkout",
            ));
        }
        let parent_path = path
            .parent()
            .ok_or_else(|| invalid("recovery provenance output has no parent"))?
            .to_path_buf();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| invalid("recovery provenance output name is not UTF-8"))?
            .to_owned();
        let parent = open_direct_directory(&parent_path)?;
        let parent_identity = directory_identity(&parent)?;
        require_private_parent(parent_identity)?;
        require_absent(&parent, &name)?;
        Ok(Self {
            path: path.to_path_buf(),
            parent_path,
            parent,
            parent_identity,
            name,
        })
    }

    pub(super) fn revalidate_absent(&self) -> Result<(), CoordError> {
        self.revalidate_parent()?;
        require_absent(&self.parent, &self.name)
    }

    pub(super) fn verify_published(&self) -> Result<(), CoordError> {
        self.revalidate_parent()?;
        let descriptor = openat(
            &self.parent,
            &self.name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| changed(format!("cannot reopen provenance output: {error}")))?;
        let file = File::from(descriptor);
        let metadata = file.metadata().map_err(CoordError::io)?;
        if !metadata.is_file()
            || metadata.uid() != rustix::process::geteuid().as_raw()
            || metadata.mode() & 0o7777 != 0o400
            || metadata.nlink() != 1
            || metadata.len() == 0
        {
            return Err(changed(
                "published provenance output lacks exact mode-0400 single-link custody",
            ));
        }
        Ok(())
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn parent_path(&self) -> &Path {
        &self.parent_path
    }

    fn revalidate_parent(&self) -> Result<(), CoordError> {
        let reopened = open_direct_directory(&self.parent_path)?;
        let identity = directory_identity(&reopened)?;
        require_private_parent(identity)?;
        if identity != self.parent_identity {
            return Err(changed("provenance output parent identity changed"));
        }
        Ok(())
    }
}

fn require_absent(parent: &File, name: &str) -> Result<(), CoordError> {
    match statat(parent, name, AtFlags::SYMLINK_NOFOLLOW) {
        Err(rustix::io::Errno::NOENT) => Ok(()),
        Ok(_) => Err(invalid("recovery provenance output already exists")),
        Err(error) => Err(invalid(format!(
            "cannot inspect recovery provenance output: {error}"
        ))),
    }
}

fn open_direct_directory(path: &Path) -> Result<File, CoordError> {
    let root = open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| invalid(format!("cannot open filesystem root: {error}")))?;
    let relative = path
        .strip_prefix("/")
        .map_err(|_| invalid("provenance output parent is outside filesystem root"))?;
    let relative = if relative.as_os_str().is_empty() {
        Path::new(".")
    } else {
        relative
    };
    let descriptor = openat2(
        &root,
        relative,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map_err(|error| invalid(format!("cannot open provenance output parent: {error}")))?;
    Ok(File::from(descriptor))
}

fn directory_identity(file: &File) -> Result<DirectoryIdentity, CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    if !metadata.is_dir() {
        return Err(invalid("provenance output parent is not a directory"));
    }
    Ok(DirectoryIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
        owner_uid: metadata.uid(),
        mode: metadata.mode() & 0o7777,
    })
}

fn require_private_parent(identity: DirectoryIdentity) -> Result<(), CoordError> {
    if identity.owner_uid != rustix::process::geteuid().as_raw() || identity.mode != 0o700 {
        return Err(invalid(
            "provenance output parent must have exact-euid mode-0700 custody",
        ));
    }
    Ok(())
}

fn require_direct_absolute(path: &Path, label: &str) -> Result<(), CoordError> {
    require_path_text(path, label)?;
    let canonical = fs::canonicalize(path)
        .map_err(|error| invalid(format!("cannot resolve {label} executable: {error}")))?;
    if canonical != path {
        return Err(invalid(format!(
            "{label} executable must be its canonical direct pathname, never a symlink or alias"
        )));
    }
    Ok(())
}

fn require_path_text(path: &Path, label: &str) -> Result<(), CoordError> {
    if !super::super::is_normalized_absolute(path) {
        return Err(invalid(format!(
            "{label} path must be normalized absolute lexical bytes"
        )));
    }
    let text = path
        .to_str()
        .ok_or_else(|| invalid(format!("{label} path is not UTF-8")))?;
    if text.len() > 4_096 || text.chars().any(char::is_control) {
        return Err(invalid(format!("{label} path is outside its text bound")));
    }
    Ok(())
}

fn open_direct_file(path: &Path, label: &str) -> Result<File, CoordError> {
    let root = open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| invalid(format!("cannot open filesystem root: {error}")))?;
    let relative = path
        .strip_prefix("/")
        .map_err(|_| invalid(format!("{label} is outside filesystem root")))?;
    let descriptor = openat2(
        &root,
        relative,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map_err(|error| invalid(format!("cannot open {label} executable: {error}")))?;
    Ok(File::from(descriptor))
}

fn read_exact_bounded(
    file: &mut File,
    before: FileIdentity,
    maximum: u64,
) -> Result<Vec<u8>, CoordError> {
    if before.length == 0 || before.length > maximum {
        return Err(invalid("executable subject length is outside its bound"));
    }
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let mut bytes = Vec::new();
    file.take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    let after = FileIdentity::read(file)?;
    if before != after || bytes.len() as u64 != before.length {
        return Err(changed("executable subject changed while being read"));
    }
    Ok(bytes)
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_PRODUCTION", reason)
}

fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_SUBJECT_CHANGED", reason)
}
