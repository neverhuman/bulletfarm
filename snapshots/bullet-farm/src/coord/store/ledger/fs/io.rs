use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    path::{Component, Path},
};

use crate::coord::CoordError;

use super::{Identity, changed, invalid, missing, os_error, platform};

pub(super) fn open_dir_path(path: &Path, mode: u32) -> Result<File, CoordError> {
    open_dir_path_optional(path, mode)?
        .ok_or_else(|| missing(format!("missing {}", path.display())))
}

pub(super) fn open_dir_path_optional(path: &Path, mode: u32) -> Result<Option<File>, CoordError> {
    if !normalized(path) {
        return Err(invalid("directory path is not absolute and normalized"));
    }
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{Mode, OFlags, ResolveFlags, openat2};
        let descriptor = match openat2(
            rustix::fs::CWD,
            path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
        ) {
            Ok(value) => value,
            Err(rustix::io::Errno::NOENT) => return Ok(None),
            Err(error) => return Err(os_error("cannot admit directory", error)),
        };
        let file = File::from(descriptor);
        validate_dir(&file, mode)?;
        Ok(Some(file))
    }
    #[cfg(not(target_os = "linux"))]
    Err(platform())
}

pub(super) fn open_dir_at(parent: &File, name: &str, mode: u32) -> Result<File, CoordError> {
    validate_name(name)?;
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{Mode, OFlags, openat};
        let descriptor = openat(
            parent,
            name,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| os_error("cannot admit child directory", error))?;
        let file = File::from(descriptor);
        validate_dir(&file, mode)?;
        Ok(file)
    }
    #[cfg(not(target_os = "linux"))]
    Err(platform())
}

pub(super) fn open_file_at(
    parent: &File,
    name: &str,
    writable: bool,
    mode: u32,
    length: Option<u64>,
) -> Result<File, CoordError> {
    validate_relative(name)?;
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{Mode, OFlags, ResolveFlags, openat2};
        let access = if writable {
            OFlags::RDWR
        } else {
            OFlags::RDONLY
        };
        let descriptor = openat2(
            parent,
            name,
            access | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
        )
        .map_err(|error| {
            if error == rustix::io::Errno::NOENT {
                missing(format!("missing {name}"))
            } else {
                os_error("cannot admit coordination file", error)
            }
        })?;
        let file = File::from(descriptor);
        validate_file(&file, mode, length)?;
        Ok(file)
    }
    #[cfg(not(target_os = "linux"))]
    Err(platform())
}

pub(super) fn open_optional_file(
    parent: &File,
    name: &str,
    mode: u32,
) -> Result<Option<File>, CoordError> {
    match open_file_at(parent, name, false, mode, None) {
        Ok(file) => Ok(Some(file)),
        Err(error) if error.code() == "COORD_SUBJECT_MISSING" => Ok(None),
        Err(error) => Err(error),
    }
}

pub(super) fn child_exists(directory: &File, name: &str) -> Result<bool, CoordError> {
    #[cfg(target_os = "linux")]
    {
        match rustix::fs::statat(directory, name, rustix::fs::AtFlags::SYMLINK_NOFOLLOW) {
            Ok(_) => Ok(true),
            Err(rustix::io::Errno::NOENT) => Ok(false),
            Err(error) => Err(os_error("cannot inspect coordination child", error)),
        }
    }
    #[cfg(not(target_os = "linux"))]
    Err(platform())
}

pub(super) fn open_sealed_dir_at(parent: &File, name: &str) -> Result<File, CoordError> {
    validate_name(name)?;
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{FileType, Mode, OFlags, fstat, openat};
        let descriptor = openat(
            parent,
            name,
            OFlags::PATH | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(|error| os_error("cannot retain sealed directory", error))?;
        let file = File::from(descriptor);
        let stat =
            fstat(&file).map_err(|error| os_error("cannot inspect sealed directory", error))?;
        if !FileType::from_raw_mode(stat.st_mode).is_dir()
            || stat.st_uid != owner()
            || stat.st_nlink != 2
            || stat.st_mode & 0o7777 != 0
        {
            return Err(invalid("sealed directory metadata is invalid"));
        }
        Ok(file)
    }
    #[cfg(not(target_os = "linux"))]
    Err(platform())
}

pub(super) fn double_empty_dir_at(
    parent: &File,
    name: &str,
    expected: Identity,
    mode: u32,
) -> Result<(), CoordError> {
    for _ in 0..2 {
        let directory = open_dir_at(parent, name, mode)?;
        if identity(&directory)? != expected {
            return Err(changed("directory inventory descriptor differs"));
        }
        inventory_empty_dir(&directory)?;
    }
    Ok(())
}

pub(super) fn inventory_empty_dir(directory: &File) -> Result<(), CoordError> {
    // `Dir::read_from` reopens `.` and therefore fails after the directory is
    // mode 000. A duplicate of the already-admitted readable descriptor keeps
    // the inventory bound to that inode without crossing the pathname again.
    let duplicate = rustix::io::dup(directory)
        .map_err(|error| os_error("cannot duplicate retained directory", error))?;
    rustix::fs::seek(&duplicate, rustix::fs::SeekFrom::Start(0))
        .map_err(|error| os_error("cannot rewind retained directory", error))?;
    let mut entries = rustix::fs::Dir::new(duplicate)
        .map_err(|error| os_error("cannot inventory retained directory", error))?;
    while let Some(entry) = entries.read() {
        let entry =
            entry.map_err(|error| os_error("cannot inventory retained directory", error))?;
        if !matches!(entry.file_name().to_bytes(), b"." | b"..") {
            return Err(invalid("retained directory is not empty"));
        }
    }
    Ok(())
}

pub(super) fn child_directory_mode(parent: &File, name: &str) -> Result<Option<u32>, CoordError> {
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{AtFlags, FileType, statat};
        let stat = match statat(parent, name, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(value) => value,
            Err(rustix::io::Errno::NOENT) => return Ok(None),
            Err(error) => return Err(os_error("cannot inspect child directory", error)),
        };
        if !FileType::from_raw_mode(stat.st_mode).is_dir()
            || stat.st_uid != owner()
            || stat.st_nlink != 2
        {
            return Err(invalid("child directory type, owner, or links are invalid"));
        }
        Ok(Some(stat.st_mode & 0o7777))
    }
    #[cfg(not(target_os = "linux"))]
    Err(platform())
}

pub(super) fn revalidate_dir_at(
    parent: &File,
    name: &str,
    expected: Identity,
    mode: u32,
) -> Result<(), CoordError> {
    #[cfg(target_os = "linux")]
    {
        use rustix::fs::{AtFlags, FileType, statat};
        let stat = statat(parent, name, AtFlags::SYMLINK_NOFOLLOW)
            .map_err(|error| os_error("cannot revalidate child directory", error))?;
        if Identity(stat.st_dev, stat.st_ino) != expected
            || !FileType::from_raw_mode(stat.st_mode).is_dir()
            || stat.st_uid != owner()
            || stat.st_nlink != 2
            || stat.st_mode & 0o7777 != mode
        {
            return Err(changed("child directory identity or metadata changed"));
        }
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    Err(platform())
}

pub(super) fn exact(file: &mut File, expected: &[u8]) -> Result<(), CoordError> {
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let mut actual = Vec::new();
    Read::by_ref(file)
        .take(expected.len() as u64 + 1)
        .read_to_end(&mut actual)
        .map_err(CoordError::io)?;
    if actual != expected || file.metadata().map_err(CoordError::io)?.len() != expected.len() as u64
    {
        return Err(changed("durable file differs from intended exact bytes"));
    }
    Ok(())
}

pub(super) fn read_canonical(file: &mut File) -> Result<Vec<u8>, CoordError> {
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let expected = file.metadata().map_err(CoordError::io)?.len();
    let maximum = bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES as u64 + 1;
    if expected > maximum {
        return Err(invalid("canonical authority file is oversized"));
    }
    let mut bytes = Vec::new();
    Read::by_ref(file)
        .take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    if bytes.len() as u64 != expected {
        return Err(changed("canonical authority file changed while read"));
    }
    Ok(bytes)
}

pub(super) fn validate_file(file: &File, mode: u32, length: Option<u64>) -> Result<(), CoordError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let value = file.metadata().map_err(CoordError::io)?;
        if !value.is_file()
            || value.uid() != owner()
            || value.nlink() != 1
            || value.mode() & 0o7777 != mode
            || length.is_some_and(|n| value.len() != n)
        {
            return Err(invalid(
                "file owner, link, type, mode, or length is invalid",
            ));
        }
        Ok(())
    }
    #[cfg(not(unix))]
    Err(platform())
}

pub(super) fn identity(file: &File) -> Result<Identity, CoordError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let value = file.metadata().map_err(CoordError::io)?;
        Ok(Identity(value.dev(), value.ino()))
    }
    #[cfg(not(unix))]
    Err(platform())
}

pub(super) fn current_mode(path: &Path) -> Result<u32, CoordError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(fs::symlink_metadata(path).map_err(CoordError::io)?.mode() & 0o7777)
    }
    #[cfg(not(unix))]
    Err(platform())
}

pub(super) fn owner() -> u32 {
    rustix::process::geteuid().as_raw()
}
pub(super) fn normalized(path: &Path) -> bool {
    path.is_absolute()
        && path
            .components()
            .all(|p| matches!(p, Component::RootDir | Component::Normal(_)))
}
pub(super) fn validate_name(value: &str) -> Result<(), CoordError> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains(['/', '\\'])
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        Err(invalid("invalid child name"))
    } else {
        Ok(())
    }
}
pub(super) fn validate_relative(value: &str) -> Result<(), CoordError> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains('\\')
        || path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        Err(invalid("invalid relative path"))
    } else {
        Ok(())
    }
}
pub(super) fn linux_only() -> Result<(), CoordError> {
    if cfg!(target_os = "linux") {
        Ok(())
    } else {
        Err(platform())
    }
}

fn validate_dir(file: &File, mode: u32) -> Result<(), CoordError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let value = file.metadata().map_err(CoordError::io)?;
        if !value.is_dir() || value.uid() != owner() || value.mode() & 0o7777 != mode {
            return Err(invalid("directory owner, type, or mode is invalid"));
        }
        Ok(())
    }
    #[cfg(not(unix))]
    Err(platform())
}
