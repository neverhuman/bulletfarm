//! Portable outer manifests resolve only against a caller-admitted absolute root.

#[cfg(all(test, target_os = "linux"))]
mod tests;

use std::{
    fs,
    path::{Path, PathBuf},
};

#[cfg(target_os = "linux")]
use std::{
    fs::{File, Metadata},
    io::{Read, Seek},
    os::unix::fs::MetadataExt,
};

#[cfg(target_os = "linux")]
use rustix::fs::{Mode, OFlags, ResolveFlags, openat2};
use serde::Deserialize;

use crate::coord::CoordError;

const MEMBERS: [&str; 4] = [
    "bullet-farm",
    "bullet-kernel",
    "bullet-git",
    "bullet-portal",
];
const MAX_BYTES: u64 = 1024 * 1024;

#[derive(Deserialize)]
pub(crate) struct PathMode {
    schema_version: Option<String>,
    split_root: Option<PathBuf>,
}

impl PathMode {
    pub(crate) fn validate_if_portable(&self, bytes: &[u8], root: &Path) -> Result<(), CoordError> {
        match self.schema_version.as_deref() {
            Some("1.3.0") => validate_exact_manifest(bytes, root),
            Some("1.2.0") | None => Ok(()),
            Some(_) => Err(invalid("outer manifest schema version is unsupported")),
        }
    }

    pub(crate) fn resolve(
        &self,
        root: &Path,
        name: &str,
        declared: &Path,
    ) -> Result<PathBuf, CoordError> {
        crate::coord::validate_repo_name(name)?;
        let expected = root.join(name);
        let valid = match self.schema_version.as_deref() {
            Some("1.3.0") => {
                require_canonical_root(root)?;
                self.split_root.as_deref().map(Path::as_os_str) == Some(Path::new(".").as_os_str())
                    && declared.as_os_str() == Path::new(name).as_os_str()
            }
            // Older outer manifests omitted their version. They never admit relative paths.
            Some("1.2.0") | None => declared.is_absolute() && declared == expected,
            Some(_) => return Err(invalid("outer manifest schema version is unsupported")),
        };
        if !valid {
            return Err(CoordError::new(
                "INVALID_MEMBER_PATH",
                format!("manifest path for {name} differs from the admitted family root/name"),
            ));
        }
        Ok(expected)
    }
}

#[derive(Deserialize)]
struct ExactManifest {
    #[serde(flatten)]
    path_mode: PathMode,
    family: String,
    required_repos: Vec<String>,
    repo: Vec<Member>,
}

#[derive(Deserialize)]
struct Member {
    name: String,
    path: PathBuf,
    jeryu_slug: String,
}

pub(crate) fn validate_exact_manifest(bytes: &[u8], root: &Path) -> Result<(), CoordError> {
    if bytes.is_empty() || bytes.len() > MAX_BYTES as usize || bytes.last() != Some(&b'\n') {
        return Err(invalid(
            "outer manifest must be bounded and end in a newline",
        ));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| invalid("outer manifest is not UTF-8"))?;
    let manifest: ExactManifest = toml::from_str(text)
        .map_err(|error| invalid(format!("invalid outer manifest: {error}")))?;
    if !matches!(
        manifest.path_mode.schema_version.as_deref(),
        Some("1.2.0" | "1.3.0")
    ) || manifest.family != "bullet-farm"
        || manifest.required_repos != MEMBERS
        || manifest.repo.len() != MEMBERS.len()
    {
        return Err(invalid(
            "outer manifest header or exact ordered member set differs",
        ));
    }
    for (entry, name) in manifest.repo.iter().zip(MEMBERS) {
        if entry.name != name || entry.jeryu_slug != format!("root/{name}") {
            return Err(invalid(
                "outer manifest member order or source identity differs",
            ));
        }
        manifest.path_mode.resolve(root, name, &entry.path)?;
    }
    Ok(())
}

/// Admit manifest bytes and four ordinary primary-checkout directories at an explicit root.
/// Commit, tree, object format and clean-state verification belong to the publication caller.
/// No release lock, coordinator generation, or source authority is created by this read.
pub fn validate_publication_family_root(root: &Path) -> Result<(), CoordError> {
    #[cfg(target_os = "linux")]
    {
        validate_root_with(root, || {})
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = root;
        Err(CoordError::new(
            "UNSUPPORTED_PLATFORM_CONTAINMENT",
            "publication root admission requires the Linux descriptor boundary",
        ))
    }
}

#[cfg(target_os = "linux")]
fn validate_root_with(root: &Path, after_read: impl FnOnce()) -> Result<(), CoordError> {
    require_canonical_root(root)?;
    let mut retained = vec![retain(root, true)?];
    let manifest_path = root.join("repos.manifest.toml");
    let mut manifest = retain(&manifest_path, false)?;
    if manifest.identity.len == 0
        || manifest.identity.len > MAX_BYTES
        || manifest.identity.links != 1
    {
        return Err(invalid(
            "outer manifest must be a bounded, singly-linked regular file",
        ));
    }
    let bytes = read_manifest(&mut manifest.file)?;
    validate_exact_manifest(&bytes, root)?;
    after_read();
    for name in MEMBERS {
        retained.push(retain(&root.join(name), true)?);
        retained.push(retain(&root.join(name).join(".git"), true)?);
    }
    if read_manifest(&mut manifest.file)? != bytes {
        return Err(invalid("outer manifest bytes changed during admission"));
    }
    retained.push(manifest);
    for entry in retained {
        let current = retain(&entry.path, entry.directory)?;
        if identity(&entry.file.metadata().map_err(CoordError::io)?) != entry.identity
            || current.identity != entry.identity
        {
            return Err(invalid(
                "family root, manifest or primary checkout changed during admission",
            ));
        }
    }
    Ok(())
}

fn require_canonical_root(root: &Path) -> Result<(), CoordError> {
    if !root.is_absolute()
        || root.as_os_str() != fs::canonicalize(root).map_err(CoordError::io)?.as_os_str()
    {
        return Err(invalid(
            "family root must be an explicit canonical absolute directory",
        ));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
struct Retained {
    path: PathBuf,
    directory: bool,
    file: File,
    identity: Identity,
}

#[cfg(target_os = "linux")]
#[derive(PartialEq)]
struct Identity {
    dev: u64,
    ino: u64,
    mode: u32,
    links: u64,
    len: u64,
    mtime: (i64, i64),
    ctime: (i64, i64),
}

#[cfg(target_os = "linux")]
fn identity(metadata: &Metadata) -> Identity {
    Identity {
        dev: metadata.dev(),
        ino: metadata.ino(),
        mode: metadata.mode(),
        links: metadata.nlink(),
        len: metadata.len(),
        mtime: (metadata.mtime(), metadata.mtime_nsec()),
        ctime: (metadata.ctime(), metadata.ctime_nsec()),
    }
}

#[cfg(target_os = "linux")]
fn retain(path: &Path, directory: bool) -> Result<Retained, CoordError> {
    let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK;
    let file = openat2(
        rustix::fs::CWD,
        path,
        if directory {
            flags | OFlags::DIRECTORY
        } else {
            flags
        },
        Mode::empty(),
        ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map(File::from)
    .map_err(|error| invalid(format!("cannot retain {}: {error}", path.display())))?;
    let metadata = file.metadata().map_err(CoordError::io)?;
    if metadata.is_dir() != directory || metadata.is_file() == directory {
        return Err(invalid("family path has an unexpected filesystem type"));
    }
    Ok(Retained {
        path: path.to_path_buf(),
        directory,
        identity: identity(&metadata),
        file,
    })
}

#[cfg(target_os = "linux")]
fn read_manifest(file: &mut File) -> Result<Vec<u8>, CoordError> {
    file.rewind().map_err(CoordError::io)?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    if bytes.len() > MAX_BYTES as usize {
        return Err(invalid("outer manifest exceeds 1 MiB"));
    }
    Ok(bytes)
}

fn invalid(message: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_FAMILY_MANIFEST", message)
}
