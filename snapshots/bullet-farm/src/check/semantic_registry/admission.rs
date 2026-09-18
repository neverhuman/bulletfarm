use std::{
    fs::{self, File, Metadata},
    io::Read,
    os::unix::fs::MetadataExt,
    path::{Component, Path, PathBuf},
};

use bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES;
use rustix::{
    fs::{Mode, OFlags, ResolveFlags, openat2},
    io::Errno,
};

use super::{Reject, reject};

pub(super) const MAX_REGISTRY_TOTAL_BYTES: u64 = 16 * 1024 * 1024;

const DIRECTORY_FLAGS: OFlags = OFlags::RDONLY
    .union(OFlags::DIRECTORY)
    .union(OFlags::NOFOLLOW)
    .union(OFlags::NONBLOCK)
    .union(OFlags::CLOEXEC);

pub(super) struct RegistryRoot {
    path: PathBuf,
    directory: File,
    identity: Identity,
}

impl RegistryRoot {
    pub(super) fn open(path: &Path) -> Result<Option<Self>, Reject> {
        validate_absolute(path)?;
        let descriptor = match openat2(
            rustix::fs::CWD,
            path,
            DIRECTORY_FLAGS,
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
        ) {
            Ok(descriptor) => descriptor,
            Err(Errno::NOENT) => return Ok(None),
            Err(error) => {
                return Err(reject(format!("registry root admission failed: {error}")));
            }
        };
        let directory = File::from(descriptor);
        let metadata = directory
            .metadata()
            .map_err(|error| reject(error.to_string()))?;
        if !metadata.is_dir() {
            return Err(reject("registry root is not a real non-symlink directory"));
        }
        let root = Self {
            path: path.to_path_buf(),
            identity: Identity::from(&metadata),
            directory,
        };
        root.ensure_identity()?;
        Ok(Some(root))
    }

    pub(super) fn ensure_identity(&self) -> Result<(), Reject> {
        let metadata = fs::symlink_metadata(&self.path)
            .map_err(|error| reject(format!("registry root was substituted: {error}")))?;
        if !metadata.is_dir() || Identity::from(&metadata).key() != self.identity.key() {
            return Err(reject("registry root was substituted during admission"));
        }
        Ok(())
    }

    pub(super) fn read_optional(
        &self,
        path: &str,
        remaining_bytes: &mut u64,
    ) -> Result<Option<AdmittedFile>, Reject> {
        match self.read(path, remaining_bytes) {
            Err(ReadError::Missing) => Ok(None),
            Err(ReadError::Rejected(error)) => Err(error),
            Ok(file) => Ok(Some(file)),
        }
    }

    pub(super) fn read_required(
        &self,
        path: &str,
        remaining_bytes: &mut u64,
    ) -> Result<AdmittedFile, Reject> {
        self.read(path, remaining_bytes)
            .map_err(|error| match error {
                ReadError::Missing => reject(format!("registry object {path} is absent")),
                ReadError::Rejected(error) => error,
            })
    }

    fn read(&self, path: &str, remaining_bytes: &mut u64) -> Result<AdmittedFile, ReadError> {
        self.ensure_identity().map_err(ReadError::Rejected)?;
        let mut file = self.open_relative(path)?;
        let before = file
            .metadata()
            .map_err(|error| ReadError::Rejected(reject(error.to_string())))?;
        let identity = Identity::from(&before);
        if !before.is_file()
            || before.len() == 0
            || before.len() > MAX_CANONICAL_DOCUMENT_BYTES as u64
            || before.nlink() != 1
        {
            return Err(ReadError::Rejected(reject(format!(
                "registry object {path} must be bounded, nonempty, regular, and single-link"
            ))));
        }
        reserve_bytes(remaining_bytes, before.len()).map_err(ReadError::Rejected)?;
        let mut bytes = Vec::with_capacity(before.len() as usize);
        Read::by_ref(&mut file)
            .take((MAX_CANONICAL_DOCUMENT_BYTES + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(|error| ReadError::Rejected(reject(error.to_string())))?;
        let after = file
            .metadata()
            .map_err(|error| ReadError::Rejected(reject(error.to_string())))?;
        if bytes.len() as u64 != before.len() || Identity::from(&after) != identity {
            return Err(ReadError::Rejected(reject(format!(
                "registry object {path} changed during descriptor read"
            ))));
        }
        let reopened = self.open_relative(path)?;
        let reopened = reopened
            .metadata()
            .map_err(|error| ReadError::Rejected(reject(error.to_string())))?;
        if Identity::from(&reopened) != identity {
            return Err(ReadError::Rejected(reject(format!(
                "registry object {path} was substituted during admission"
            ))));
        }
        self.ensure_identity().map_err(ReadError::Rejected)?;
        Ok(AdmittedFile { bytes, identity })
    }

    fn open_relative(&self, path: &str) -> Result<File, ReadError> {
        validate_relative(path).map_err(ReadError::Rejected)?;
        openat2(
            &self.directory,
            path,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
        )
        .map(File::from)
        .map_err(|error| classify_open(error, path))
    }
}

fn reserve_bytes(remaining: &mut u64, length: u64) -> Result<(), Reject> {
    *remaining = remaining
        .checked_sub(length)
        .ok_or_else(|| reject("registry aggregate byte budget is exhausted"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{MAX_REGISTRY_TOTAL_BYTES, reserve_bytes};

    #[test]
    fn aggregate_budget_accepts_the_exact_boundary_and_rejects_overflow() {
        let mut exact = MAX_REGISTRY_TOTAL_BYTES;
        reserve_bytes(&mut exact, MAX_REGISTRY_TOTAL_BYTES).unwrap();
        assert_eq!(exact, 0);
        assert!(reserve_bytes(&mut exact, 1).is_err());

        let mut overflow = MAX_REGISTRY_TOTAL_BYTES;
        assert!(reserve_bytes(&mut overflow, MAX_REGISTRY_TOTAL_BYTES + 1).is_err());
        assert_eq!(overflow, MAX_REGISTRY_TOTAL_BYTES);
    }
}

fn classify_open(error: Errno, path: &str) -> ReadError {
    if error == Errno::NOENT {
        ReadError::Missing
    } else {
        ReadError::Rejected(reject(format!(
            "registry object {path} failed no-follow admission: {error}"
        )))
    }
}

enum ReadError {
    Missing,
    Rejected(Reject),
}

pub(super) struct AdmittedFile {
    pub(super) bytes: Vec<u8>,
    identity: Identity,
}

impl AdmittedFile {
    pub(super) const fn identity_key(&self) -> (u64, u64) {
        self.identity.key()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Identity {
    device: u64,
    inode: u64,
    length: u64,
    links: u64,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
}

impl Identity {
    fn from(metadata: &Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            length: metadata.len(),
            links: metadata.nlink(),
            modified_seconds: metadata.mtime(),
            modified_nanoseconds: metadata.mtime_nsec(),
            changed_seconds: metadata.ctime(),
            changed_nanoseconds: metadata.ctime_nsec(),
        }
    }

    const fn key(self) -> (u64, u64) {
        (self.device, self.inode)
    }
}

fn validate_absolute(path: &Path) -> Result<(), Reject> {
    if !path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::RootDir | Component::Normal(_)))
    {
        return Err(reject("registry root must be an absolute normalized path"));
    }
    Ok(())
}

fn validate_relative(path: &str) -> Result<(), Reject> {
    if path.is_empty()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(reject("registry object path is unsafe"));
    }
    Ok(())
}
