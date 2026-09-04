use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use rustix::fs::{AtFlags, Mode, OFlags, ResolveFlags, fchmod, open, openat, openat2, statat};
use serde::{Serialize, de::DeserializeOwned};

use super::{CoordError, anonymous_link};

#[path = "sealed/raw.rs"]
mod raw;
#[path = "sealed/runtime.rs"]
mod runtime;

pub(crate) use raw::write_raw;

const MAX_DOCUMENT_BYTES: u64 = bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES as u64 + 1;

pub(crate) fn read<T>(path: &Path) -> Result<T, CoordError>
where
    T: DeserializeOwned + Serialize,
{
    read_canonical(path, MAX_DOCUMENT_BYTES, ParentAdmission::Sealed)
}

pub(crate) fn read_root_runtime<T>(path: &Path, maximum: u64) -> Result<T, CoordError>
where
    T: DeserializeOwned + Serialize,
{
    if maximum == 0 {
        return Err(invalid("root runtime document bound must be positive"));
    }
    read_canonical(path, maximum, ParentAdmission::RootRuntime)
}

fn read_canonical<T>(path: &Path, maximum: u64, admission: ParentAdmission) -> Result<T, CoordError>
where
    T: DeserializeOwned + Serialize,
{
    let bytes = read_bytes(path, maximum, admission)?;
    if bytes.last() != Some(&b'\n') || bytes[..bytes.len() - 1].contains(&b'\n') {
        return Err(invalid(
            "sealed recovery document must end in exactly one LF",
        ));
    }
    let body = &bytes[..bytes.len() - 1];
    let value = bullet_wire::decode_canonical::<T>(body).map_err(|error| {
        invalid(format!(
            "sealed recovery document is not canonical: {error}"
        ))
    })?;
    if bullet_wire::canonical_json(&value).map_err(|error| {
        invalid(format!(
            "cannot re-encode sealed recovery document: {error}"
        ))
    })? != body
    {
        return Err(invalid(
            "sealed recovery document changed after strict decode",
        ));
    }
    Ok(value)
}

pub(crate) fn write(path: &Path, value: &impl Serialize) -> Result<(), CoordError> {
    write_canonical(path, value, ParentAdmission::Sealed, false)
}

pub(in crate::coord) fn write_root_runtime(
    path: &Path,
    value: &impl Serialize,
) -> Result<(), CoordError> {
    write_canonical(path, value, ParentAdmission::RootRuntime, true)
}

fn write_canonical(
    path: &Path,
    value: &impl Serialize,
    admission: ParentAdmission,
    adopt_exact: bool,
) -> Result<(), CoordError> {
    let mut bytes = bullet_wire::canonical_json(value)
        .map_err(|error| invalid(format!("cannot canonicalize recovery document: {error}")))?;
    if bytes.len() as u64 >= MAX_DOCUMENT_BYTES {
        return Err(invalid("recovery document exceeds its byte bound"));
    }
    bytes.push(b'\n');
    write_bytes_as(path, &bytes, MAX_DOCUMENT_BYTES, admission, adopt_exact)
}

pub(crate) fn read_raw(path: &Path, maximum: u64) -> Result<Vec<u8>, CoordError> {
    if maximum == 0 {
        return Err(invalid("sealed recovery input bound must be positive"));
    }
    read_bytes(path, maximum, ParentAdmission::Sealed)
}

pub(crate) fn read_raw_legacy_live(path: &Path, maximum: u64) -> Result<Vec<u8>, CoordError> {
    if maximum == 0 {
        return Err(invalid("sealed recovery input bound must be positive"));
    }
    read_bytes(path, maximum, ParentAdmission::FrozenLegacy)
}

fn read_bytes(
    path: &Path,
    maximum: u64,
    admission: ParentAdmission,
) -> Result<Vec<u8>, CoordError> {
    let parent = Parent::open(path, admission)?;
    let (first, first_identity) = parent.read_once(maximum)?;
    let (second, second_identity) = parent.read_once(maximum)?;
    if first_identity != second_identity || first != second {
        return Err(changed(
            "sealed recovery document changed across independent reads",
        ));
    }
    parent.revalidate_path(path)?;
    Ok(first)
}

fn write_bytes(path: &Path, bytes: &[u8], maximum: u64) -> Result<(), CoordError> {
    write_bytes_as(path, bytes, maximum, ParentAdmission::Sealed, false)
}

fn write_bytes_as(
    path: &Path,
    bytes: &[u8],
    maximum: u64,
    admission: ParentAdmission,
    adopt_exact: bool,
) -> Result<(), CoordError> {
    let length = u64::try_from(bytes.len())
        .map_err(|_| invalid("recovery output length cannot be represented"))?;
    if maximum == 0 || length == 0 || length > maximum {
        return Err(invalid(
            "recovery output must be nonempty and within its explicit byte bound",
        ));
    }
    let parent = Parent::open(path, admission)?;
    if adopt_exact && runtime::adopt_exact(&parent, path, bytes, maximum)? {
        return Ok(());
    }
    parent.require_absent()?;
    let descriptor = openat(
        &parent.file,
        ".",
        OFlags::TMPFILE | OFlags::RDWR | OFlags::CLOEXEC,
        admission.file_mode(),
    )
    .map_err(|error| invalid(format!("cannot create anonymous recovery output: {error}")))?;
    let mut anonymous = File::from(descriptor);
    fchmod(&anonymous, admission.file_mode())
        .map_err(|error| invalid(format!("cannot seal recovery output mode: {error}")))?;
    anonymous.write_all(bytes).map_err(CoordError::io)?;
    exact_readback(&mut anonymous, bytes, maximum)?;
    anonymous.sync_all().map_err(CoordError::io)?;
    exact_readback(&mut anonymous, bytes, maximum)?;
    let before_link = Identity::for_file(&anonymous)?;
    before_link.validate_file(0, length, admission)?;
    match anonymous_link::link(
        &anonymous,
        &parent.file,
        &parent.name,
        (before_link.device, before_link.inode),
    ) {
        Ok(anonymous_link::LinkOutcome::Linked) => {}
        Ok(anonymous_link::LinkOutcome::Exists) => {
            return Err(invalid("recovery output already exists"));
        }
        Err(error) => {
            return Err(invalid(format!(
                "cannot publish anonymous recovery output: {error}"
            )));
        }
    }
    let after_link = Identity::for_file(&anonymous)?;
    after_link.validate_file(1, length, admission)?;
    if !before_link.same_inode_and_content(&after_link) {
        return Err(changed(
            "anonymous recovery output changed while being published",
        ));
    }
    parent.file.sync_all().map_err(CoordError::io)?;
    parent.verify_published(&mut anonymous, bytes, maximum, after_link)?;
    parent.revalidate_path(path)
}

fn exact_readback(file: &mut File, expected: &[u8], maximum: u64) -> Result<(), CoordError> {
    let read_bound = maximum
        .checked_add(1)
        .ok_or_else(|| invalid("recovery output byte bound is too large"))?;
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let mut observed = Vec::new();
    file.take(read_bound)
        .read_to_end(&mut observed)
        .map_err(CoordError::io)?;
    if observed != expected {
        return Err(changed(
            "sealed recovery document differs during descriptor read-back",
        ));
    }
    Ok(())
}

struct Parent {
    file: File,
    name: String,
    identity: ParentIdentity,
    admission: ParentAdmission,
}

impl Parent {
    fn open(path: &Path, admission: ParentAdmission) -> Result<Self, CoordError> {
        if !super::recovery_manifest::is_normalized_absolute(path) {
            return Err(invalid(
                "sealed recovery path must be normalized absolute lexical bytes",
            ));
        }
        let parent_path = path
            .parent()
            .ok_or_else(|| invalid("sealed recovery path has no parent"))?;
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| invalid("sealed recovery filename is not UTF-8"))?
            .to_owned();
        let file = if admission == ParentAdmission::RootRuntime {
            runtime::open_parent(parent_path)?
        } else {
            let root = open(
                "/",
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|error| invalid(format!("cannot open filesystem root: {error}")))?;
            let relative = parent_path
                .strip_prefix("/")
                .map_err(|_| invalid("sealed recovery parent is outside filesystem root"))?;
            let relative = if relative.as_os_str().is_empty() {
                PathBuf::from(".")
            } else {
                relative.to_owned()
            };
            let descriptor = openat2(
                &root,
                &relative,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
                ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
            )
            .map_err(|error| invalid(format!("cannot open sealed recovery parent: {error}")))?;
            File::from(descriptor)
        };
        let identity = ParentIdentity::for_file(&file)?;
        identity.validate(admission)?;
        Ok(Self {
            file,
            name,
            identity,
            admission,
        })
    }

    fn require_absent(&self) -> Result<(), CoordError> {
        match statat(&self.file, &self.name, AtFlags::SYMLINK_NOFOLLOW) {
            Err(rustix::io::Errno::NOENT) => Ok(()),
            Ok(_) => Err(invalid("recovery output already exists")),
            Err(error) => Err(invalid(format!(
                "cannot inspect recovery output name: {error}"
            ))),
        }
    }

    fn read_once(&self, maximum: u64) -> Result<(Vec<u8>, Identity), CoordError> {
        let mut file = if self.admission == ParentAdmission::RootRuntime {
            runtime::open_document(&self.file, &self.name)?
        } else {
            let descriptor = openat(
                &self.file,
                &self.name,
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map_err(|error| invalid(format!("cannot open sealed recovery document: {error}")))?;
            File::from(descriptor)
        };
        let before = Identity::for_file(&file)?;
        before.validate_file(1, maximum, self.admission)?;
        let mut bytes = Vec::new();
        (&mut file)
            .take(maximum + 1)
            .read_to_end(&mut bytes)
            .map_err(CoordError::io)?;
        let after = Identity::for_file(&file)?;
        if before != after || bytes.is_empty() || bytes.len() as u64 != before.length {
            return Err(changed("sealed recovery document changed while being read"));
        }
        Ok((bytes, before))
    }

    fn verify_published(
        &self,
        anonymous: &mut File,
        bytes: &[u8],
        maximum: u64,
        expected: Identity,
    ) -> Result<(), CoordError> {
        let descriptor = openat(
            &self.file,
            &self.name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| changed(format!("cannot reopen recovery output: {error}")))?;
        let mut published = File::from(descriptor);
        let identity = Identity::for_file(&published)?;
        identity.validate_file(1, bytes.len() as u64, self.admission)?;
        if identity != expected {
            return Err(changed(
                "published recovery output differs from retained anonymous inode",
            ));
        }
        exact_readback(&mut published, bytes, maximum)?;
        exact_readback(anonymous, bytes, maximum)?;
        if Identity::for_file(&published)? != identity {
            return Err(changed(
                "published recovery output changed during read-back",
            ));
        }
        let public = statat(&self.file, &self.name, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|error| changed(format!("cannot restat recovery output: {error}")))?;
        if (public.st_dev, public.st_ino) != (identity.device, identity.inode)
            || public.st_uid != identity.owner_uid
            || public.st_gid != identity.owner_gid
            || public.st_nlink != identity.links
            || public.st_mode & 0o7777 != identity.mode
            || u64::try_from(public.st_size).ok() != Some(identity.length)
        {
            return Err(changed(
                "recovery output pathname differs from retained descriptor",
            ));
        }
        Ok(())
    }

    fn revalidate_path(&self, path: &Path) -> Result<(), CoordError> {
        let reopened = Self::open(path, self.admission)?;
        if reopened.identity != self.identity {
            return Err(changed("sealed recovery parent identity changed"));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ParentAdmission {
    Sealed,
    FrozenLegacy,
    RootRuntime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ParentIdentity {
    device: u64,
    inode: u64,
    owner_uid: u32,
    owner_gid: u32,
    mode: u32,
}

impl ParentIdentity {
    fn for_file(file: &File) -> Result<Self, CoordError> {
        let value = file.metadata().map_err(CoordError::io)?;
        if !value.is_dir() {
            return Err(invalid("sealed recovery parent is not a directory"));
        }
        Ok(Self {
            device: value.dev(),
            inode: value.ino(),
            owner_uid: value.uid(),
            owner_gid: value.gid(),
            mode: value.mode() & 0o7777,
        })
    }

    fn validate(self, admission: ParentAdmission) -> Result<(), CoordError> {
        let admitted_custody = match admission {
            ParentAdmission::Sealed => {
                self.owner_uid == rustix::process::geteuid().as_raw() && self.mode == 0o700
            }
            ParentAdmission::FrozenLegacy => {
                self.owner_uid == rustix::process::geteuid().as_raw()
                    && matches!(self.mode, 0o700 | 0o775)
            }
            ParentAdmission::RootRuntime => {
                self.owner_uid == 0 && self.owner_gid == 0 && self.mode == 0o755
            }
        };
        if !admitted_custody {
            return Err(invalid(
                "recovery parent owner or role-specific exact mode is not admitted",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Identity {
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

impl Identity {
    fn for_file(file: &File) -> Result<Self, CoordError> {
        let value = file.metadata().map_err(CoordError::io)?;
        if !value.is_file() {
            return Err(invalid("sealed recovery document is not a regular file"));
        }
        Ok(Self {
            device: value.dev(),
            inode: value.ino(),
            owner_uid: value.uid(),
            owner_gid: value.gid(),
            mode: value.mode() & 0o7777,
            links: value.nlink(),
            length: value.len(),
            mtime_seconds: value.mtime(),
            mtime_nanoseconds: value.mtime_nsec(),
            ctime_seconds: value.ctime(),
            ctime_nanoseconds: value.ctime_nsec(),
        })
    }

    fn validate_file(
        self,
        expected_links: u64,
        maximum: u64,
        admission: ParentAdmission,
    ) -> Result<(), CoordError> {
        let admitted_custody = match admission {
            ParentAdmission::Sealed | ParentAdmission::FrozenLegacy => {
                self.owner_uid == rustix::process::geteuid().as_raw() && self.mode == 0o400
            }
            ParentAdmission::RootRuntime => {
                self.owner_uid == 0 && self.owner_gid == 0 && self.mode == 0o444
            }
        };
        if !admitted_custody
            || self.links != expected_links
            || self.length == 0
            || self.length > maximum
        {
            return Err(invalid(
                "recovery document must have role-specific exact custody, one link, and bounded length",
            ));
        }
        Ok(())
    }

    fn same_inode_and_content(self, other: &Self) -> bool {
        self.device == other.device
            && self.inode == other.inode
            && self.owner_uid == other.owner_uid
            && self.owner_gid == other.owner_gid
            && self.mode == other.mode
            && self.length == other.length
            && self.mtime_seconds == other.mtime_seconds
            && self.mtime_nanoseconds == other.mtime_nanoseconds
    }
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_PRODUCTION", reason)
}

fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_SUBJECT_CHANGED", reason)
}

#[cfg(test)]
#[path = "sealed/tests.rs"]
mod tests;
