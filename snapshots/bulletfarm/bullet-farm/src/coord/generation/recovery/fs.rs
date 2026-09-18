use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom},
    os::{fd::AsRawFd, unix::fs::MetadataExt},
    path::{Component, Path},
};

#[cfg(test)]
use std::{
    fs::OpenOptions,
    io::Write,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::PathBuf,
};

use nix::{
    errno::Errno,
    fcntl::{RenameFlags, renameat2},
};
use sha2::{Digest, Sha256};

use super::{ContentExpectation, SourceExpectation};
use crate::coord::CoordError;
#[cfg(test)]
use crate::coord::generation::manifest::{ArtifactBinding, verify_artifact};

pub(super) fn open_exact_file(
    path: &Path,
    owner: u32,
    mode: u32,
    write: bool,
) -> Result<File, CoordError> {
    let before = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if !path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::RootDir | Component::Normal(_)))
    {
        return Err(invalid(
            "recovery subject path is not absolute and normalized",
        ));
    }
    let access = if write {
        rustix::fs::OFlags::RDWR
    } else {
        rustix::fs::OFlags::RDONLY
    };
    let descriptor = rustix::fs::openat2(
        rustix::fs::CWD,
        path,
        access
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::NONBLOCK
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
        rustix::fs::ResolveFlags::NO_SYMLINKS | rustix::fs::ResolveFlags::NO_MAGICLINKS,
    )
    .map_err(|error| {
        invalid(format!(
            "cannot open recovery subject without symlinks: {error}"
        ))
    })?;
    let file = File::from(descriptor);
    let after = file.metadata().map_err(CoordError::io)?;
    if !before.file_type().is_file()
        || !after.is_file()
        || before.dev() != after.dev()
        || before.ino() != after.ino()
        || after.uid() != owner
        || after.nlink() != 1
        || after.mode() & 0o7777 != mode
    {
        return Err(invalid(
            "file identity, owner, type, link count, or mode changed",
        ));
    }
    Ok(file)
}

pub(super) fn revalidate_path(
    retained: &File,
    path: &Path,
    owner: u32,
    mode: u32,
    write: bool,
) -> Result<(), CoordError> {
    let reopened = open_exact_file(path, owner, mode, write)?;
    if super::verifier::identity(retained)? != super::verifier::identity(&reopened)? {
        return Err(invalid(
            "retained descriptor no longer matches its exact pathname",
        ));
    }
    Ok(())
}

pub(super) fn open_source(source: &SourceExpectation, owner: u32) -> Result<File, CoordError> {
    let mut file = open_exact_file(&source.path, owner, 0o400, false)?;
    verify_open_file(&mut file, &source.content)?;
    Ok(file)
}

pub(super) fn verify_open_file(
    file: &mut File,
    expected: &ContentExpectation,
) -> Result<(), CoordError> {
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let mut hasher = Sha256::new();
    let copied = std::io::copy(file, &mut hasher).map_err(CoordError::io)?;
    let metadata = file.metadata().map_err(CoordError::io)?;
    if copied != expected.byte_length
        || metadata.len() != expected.byte_length
        || format!("sha256:{:x}", hasher.finalize()) != expected.sha256.as_str()
    {
        return Err(invalid("source length or SHA-256 differs from exact input"));
    }
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    Ok(())
}

pub(super) fn verify_prefix(
    file: &mut File,
    expected: &ContentExpectation,
) -> Result<(), CoordError> {
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let mut limited = file.take(expected.byte_length);
    let mut hasher = Sha256::new();
    let copied = std::io::copy(&mut limited, &mut hasher).map_err(CoordError::io)?;
    file.seek(SeekFrom::Start(expected.byte_length.saturating_sub(1)))
        .map_err(CoordError::io)?;
    let mut boundary = [0_u8; 2];
    let read = file.read(&mut boundary).map_err(CoordError::io)?;
    if copied != expected.byte_length
        || format!("sha256:{:x}", hasher.finalize()) != expected.sha256.as_str()
        || read != 2
        || boundary[0] != b'\n'
        || boundary[1] == b'\n'
    {
        return Err(invalid(
            "trusted prefix digest or exact LF boundary is invalid",
        ));
    }
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    Ok(())
}

#[cfg(test)]
pub(super) fn copy_prefix(
    source: &mut File,
    root: &Path,
    binding: &ArtifactBinding,
) -> Result<(), CoordError> {
    source.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    copy_reader(&mut source.take(binding.byte_length), root, binding)
}

#[cfg(test)]
pub(super) fn copy_artifact(
    source: &mut File,
    root: &Path,
    binding: &ArtifactBinding,
) -> Result<(), CoordError> {
    source.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    copy_reader(source, root, binding)
}

#[cfg(test)]
fn copy_reader(
    source: &mut impl Read,
    root: &Path,
    binding: &ArtifactBinding,
) -> Result<(), CoordError> {
    let relative = Path::new(binding.relative_path.as_str());
    if relative.parent() != Some(Path::new("archive")) {
        return Err(invalid(
            "recovery artifacts must be direct archive children",
        ));
    }
    let archive = admitted_directory(root, "archive", rustix::process::geteuid().as_raw())?;
    let destination = root.join(relative);
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o600);
    let mut output = options.open(&destination).map_err(collision)?;
    let mut buffer = [0_u8; 64 * 1024];
    let mut line_count = 0_u64;
    let mut byte_count = 0_u64;
    let mut last = None;
    loop {
        let read = source.read(&mut buffer).map_err(CoordError::io)?;
        if read == 0 {
            break;
        }
        output.write_all(&buffer[..read]).map_err(CoordError::io)?;
        byte_count = byte_count
            .checked_add(read as u64)
            .ok_or_else(|| invalid("artifact byte count overflowed"))?;
        line_count += buffer[..read].iter().filter(|byte| **byte == b'\n').count() as u64;
        last = Some(buffer[read - 1]);
    }
    if byte_count != binding.byte_length
        || binding.record_count != Some(line_count)
        || binding.ends_with_lf != (last == Some(b'\n'))
    {
        return Err(invalid(
            "artifact byte/record/LF shape differs from manifest",
        ));
    }
    output.sync_all().map_err(CoordError::io)?;
    output
        .set_permissions(fs::Permissions::from_mode(0o400))
        .map_err(CoordError::io)?;
    output.sync_all().map_err(CoordError::io)?;
    sync_dir(&archive)?;
    verify_artifact(root, binding, &binding.relative_path)
}

pub(super) fn open_directory(path: &Path, owner: u32, mode: u32) -> Result<File, CoordError> {
    let descriptor = rustix::fs::openat2(
        rustix::fs::CWD,
        path,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::DIRECTORY
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
        rustix::fs::ResolveFlags::NO_SYMLINKS | rustix::fs::ResolveFlags::NO_MAGICLINKS,
    )
    .map(File::from)
    .map_err(|error| invalid(format!("cannot retain recovery directory: {error}")))?;
    validate_directory(&descriptor, owner, mode)?;
    Ok(descriptor)
}

pub(super) fn open_child_directory(
    parent: &File,
    name: &str,
    owner: u32,
    mode: u32,
) -> Result<File, CoordError> {
    let directory = open_child_directory_any_mode(parent, name, owner)?;
    validate_directory(&directory, owner, mode)?;
    Ok(directory)
}

#[cfg(test)]
pub(super) fn admitted_directory(
    parent: &Path,
    name: &str,
    owner: u32,
) -> Result<PathBuf, CoordError> {
    let path = parent.join(name);
    match fs::DirBuilder::new().mode(0o700).create(&path) {
        Ok(()) => {
            sync_dir(parent)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(CoordError::io(error)),
    }
    require_directory_owner(&path, owner, 0o700)?;
    Ok(path)
}

#[cfg(test)]
pub(super) fn write_new_file(path: &Path, bytes: &[u8], mode: u32) -> Result<(), CoordError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(0o600);
    let mut file = options.open(path).map_err(collision)?;
    file.write_all(bytes).map_err(CoordError::io)?;
    file.sync_all().map_err(CoordError::io)?;
    file.set_permissions(fs::Permissions::from_mode(mode))
        .map_err(CoordError::io)?;
    file.sync_all().map_err(CoordError::io)?;
    revalidate_path(
        &file,
        path,
        rustix::process::geteuid().as_raw(),
        mode,
        false,
    )
}

pub(super) fn publish_no_replace_at(
    parent: &File,
    source: &str,
    destination: &str,
) -> Result<(), CoordError> {
    renameat2(
        Some(parent.as_raw_fd()),
        source,
        Some(parent.as_raw_fd()),
        destination,
        RenameFlags::RENAME_NOREPLACE,
    )
    .map_err(|error| match error {
        Errno::EEXIST => CoordError::new(
            "COORD_GENERATION_CONFLICT",
            "generation destination appeared during publication",
        ),
        _ => CoordError::new("COORD_GENERATION_PUBLISH_FAILED", error.to_string()),
    })
}

#[cfg(test)]
pub(super) fn sync_dir(path: &Path) -> Result<(), CoordError> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .custom_flags(nix::libc::O_DIRECTORY | nix::libc::O_NOFOLLOW | nix::libc::O_CLOEXEC);
    options
        .open(path)
        .and_then(|file| file.sync_all())
        .map_err(CoordError::io)
}

#[cfg(test)]
fn require_directory_owner(path: &Path, owner: u32, mode: u32) -> Result<(), CoordError> {
    let metadata = fs::symlink_metadata(path).map_err(CoordError::io)?;
    if !metadata.file_type().is_dir() || metadata.uid() != owner || metadata.mode() & 0o7777 != mode
    {
        return Err(invalid("directory type, owner, or mode is not admitted"));
    }
    Ok(())
}

fn open_child_directory_any_mode(
    parent: &File,
    name: &str,
    owner: u32,
) -> Result<File, CoordError> {
    if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." {
        return Err(invalid("recovery directory child name is invalid"));
    }
    let directory = rustix::fs::openat2(
        parent,
        name,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::DIRECTORY
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
        rustix::fs::ResolveFlags::BENEATH
            | rustix::fs::ResolveFlags::NO_SYMLINKS
            | rustix::fs::ResolveFlags::NO_MAGICLINKS,
    )
    .map(File::from)
    .map_err(|error| invalid(format!("cannot retain recovery child directory: {error}")))?;
    if directory.metadata().map_err(CoordError::io)?.uid() != owner {
        return Err(invalid("recovery child directory owner is not admitted"));
    }
    Ok(directory)
}

fn validate_directory(file: &File, owner: u32, mode: u32) -> Result<(), CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    if !metadata.is_dir() || metadata.uid() != owner || metadata.mode() & 0o7777 != mode {
        return Err(invalid(
            "retained directory type, owner, or mode is not admitted",
        ));
    }
    Ok(())
}

pub(super) fn require_empty_descriptor(directory: &File) -> Result<(), CoordError> {
    let duplicate = rustix::io::dup(directory)
        .map_err(|error| invalid(format!("cannot duplicate retained directory: {error}")))?;
    let mut entries = rustix::fs::Dir::new(duplicate)
        .map_err(|error| invalid(format!("cannot inventory retained directory: {error}")))?;
    while let Some(entry) = entries.read() {
        let entry =
            entry.map_err(|error| invalid(format!("cannot read recovery directory: {error}")))?;
        let name = entry.file_name().to_bytes();
        if name != b"." && name != b".." {
            return Err(invalid("recovery exchange directory is not empty"));
        }
    }
    Ok(())
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_RECOVERY", reason)
}

#[cfg(test)]
fn collision(error: std::io::Error) -> CoordError {
    if error.kind() == std::io::ErrorKind::AlreadyExists {
        CoordError::new(
            "COORD_RECOVERY_COLLISION",
            "a recovery destination already exists and was preserved",
        )
    } else {
        CoordError::io(error)
    }
}
