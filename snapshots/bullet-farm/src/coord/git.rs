use std::{
    ffi::OsStr,
    fs::{File, Metadata},
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::Path,
    process::Command,
    time::Duration,
};

use super::{CoordError, validate_commit_oid, validate_path, validate_repo_name};
use crate::process::{Limits, run_bounded};
use rustix::{
    fd::AsFd,
    fs::{AtFlags, FileType, Mode, OFlags, ResolveFlags, openat2, statat},
};
use serde::Serialize;

#[path = "recovery_adoption_verify/git.rs"]
mod recovery;
pub(super) use recovery::{derive_recovery_commit, verify_recovery_commit};
mod repository;
pub(in crate::coord) mod wave0;

const GIT_BIN: &str = "/usr/bin/git";
const GIT_LIMITS: Limits = Limits {
    timeout: Duration::from_secs(120),
    stdout_bytes: 16 * 1024 * 1024,
    stderr_bytes: 16 * 1024 * 1024,
};
const MAX_OBJECT_NODES: usize = 1_000_000;
const MAX_OBJECT_PATH_BYTES: usize = 512;
pub(super) const WAVE0_REPOSITORIES: [(&str, &str); 4] = [
    ("bullet-farm", "root/bullet-farm"),
    ("bullet-kernel", "root/bullet-kernel"),
    ("bullet-git", "root/bullet-git"),
    ("bullet-portal", "root/bullet-portal"),
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct ObjectNode {
    relative_path: String,
    kind: u8,
    device: u64,
    inode: u64,
    links: u64,
    byte_length: u64,
    mtime_seconds: i64,
    mtime_nanoseconds: i64,
    ctime_seconds: i64,
    ctime_nanoseconds: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DescriptorIdentity {
    directory: bool,
    device: u64,
    inode: u64,
    links: u64,
    byte_length: u64,
    mtime_seconds: i64,
    mtime_nanoseconds: i64,
    ctime_seconds: i64,
    ctime_nanoseconds: i64,
}

pub(super) fn descriptor_object_inventory(repo_root: &Path) -> Result<String, CoordError> {
    descriptor_object_inventory_with(repo_root, || {})
}

#[cfg(test)]
pub(super) fn descriptor_object_inventory_after_open(
    repo_root: &Path,
    after_open: impl FnOnce(),
) -> Result<String, CoordError> {
    descriptor_object_inventory_with(repo_root, after_open)
}

fn descriptor_object_inventory_with(
    repo_root: &Path,
    after_open: impl FnOnce(),
) -> Result<String, CoordError> {
    let root = open_object_root(repo_root)?;
    let root_identity = DescriptorIdentity::directory(&root)?;
    let git = open_object_directory(&root, Path::new(".git"))?;
    let git_identity = DescriptorIdentity::directory(&git)?;
    let objects = open_object_directory(&git, Path::new("objects"))?;
    let objects_identity = DescriptorIdentity::directory(&objects)?;
    after_open();
    let first = object_snapshot(&objects, MAX_OBJECT_NODES)?;
    let second = object_snapshot(&objects, MAX_OBJECT_NODES)?;
    if first != second
        || DescriptorIdentity::directory(&root)? != root_identity
        || DescriptorIdentity::directory(&git)? != git_identity
        || DescriptorIdentity::directory(&objects)? != objects_identity
    {
        return Err(object_mismatch(
            "Git object store changed across complete descriptor observations",
        ));
    }
    let reopened_root = open_object_root(repo_root)?;
    let reopened_git = open_object_directory(&reopened_root, Path::new(".git"))?;
    let reopened_objects = open_object_directory(&reopened_git, Path::new("objects"))?;
    if DescriptorIdentity::directory(&reopened_root)? != root_identity
        || DescriptorIdentity::directory(&reopened_git)? != git_identity
        || DescriptorIdentity::directory(&reopened_objects)? != objects_identity
    {
        return Err(object_mismatch(
            "Git object-store pathname identity changed",
        ));
    }
    Ok(format!(
        "blake3:{}",
        bullet_wire::hash_canonical("bullet-family.coord.git-object-store.v1", &second)
            .map_err(|error| object_mismatch(format!("cannot hash Git object store: {error}")))?
            .to_hex()
    ))
}

#[cfg(test)]
pub(super) fn object_snapshot_for_test(root: &File, maximum: usize) -> Result<usize, CoordError> {
    object_snapshot(root, maximum).map(|nodes| nodes.len())
}

fn object_snapshot(root: &File, maximum: usize) -> Result<Vec<ObjectNode>, CoordError> {
    let expected = DescriptorIdentity::directory(root)?;
    let mut nodes = Vec::new();
    walk_object_directory(root, &[], &mut nodes, maximum)?;
    if DescriptorIdentity::directory(root)? != expected {
        return Err(object_mismatch(
            "Git object-store root changed during inventory",
        ));
    }
    nodes.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    if nodes
        .windows(2)
        .any(|pair| pair[0].relative_path == pair[1].relative_path)
    {
        return Err(object_mismatch(
            "Git object inventory contains a duplicate path",
        ));
    }
    Ok(nodes)
}

fn walk_object_directory(
    directory: &File,
    prefix: &[u8],
    nodes: &mut Vec<ObjectNode>,
    maximum: usize,
) -> Result<(), CoordError> {
    let directory_identity = DescriptorIdentity::directory(directory)?;
    let mut entries = rustix::fs::Dir::read_from(directory)
        .map_err(|error| object_mismatch(format!("cannot enumerate Git objects: {error}")))?;
    while let Some(entry) = entries.read() {
        let entry = entry
            .map_err(|error| object_mismatch(format!("cannot read Git object entry: {error}")))?;
        let name = entry.file_name().to_bytes();
        if matches!(name, b"." | b"..") {
            continue;
        }
        if nodes.len() == maximum {
            return Err(object_mismatch(
                "Git object store exceeds its closed inventory bound",
            ));
        }
        if name.is_empty() || name.contains(&b'/') {
            return Err(object_mismatch("Git object store contains an unsafe name"));
        }
        let mut relative =
            Vec::with_capacity(prefix.len() + usize::from(!prefix.is_empty()) + name.len());
        relative.extend_from_slice(prefix);
        if !prefix.is_empty() {
            relative.push(b'/');
        }
        relative.extend_from_slice(name);
        if relative.len() > MAX_OBJECT_PATH_BYTES {
            return Err(object_mismatch("Git object path exceeds its closed bound"));
        }
        let relative_path = std::str::from_utf8(&relative)
            .map_err(|_| object_mismatch("Git object path is not UTF-8"))?
            .to_owned();
        if matches!(
            relative_path.as_str(),
            "info/alternates" | "info/http-alternates"
        ) || relative_path.ends_with(".promisor")
        {
            return Err(object_mismatch(
                "Git alternate or promisor object carriers are not admitted",
            ));
        }
        let raw = statat(
            directory,
            Path::new(OsStr::from_bytes(name)),
            AtFlags::SYMLINK_NOFOLLOW,
        )
        .map_err(|error| object_mismatch(format!("cannot stat Git object entry: {error}")))?;
        let file_type = FileType::from_raw_mode(raw.st_mode);
        if !file_type.is_dir() && !file_type.is_file() {
            return Err(object_mismatch(
                "Git object store contains a special or symbolic node",
            ));
        }
        let child = open_object_child(directory, name, file_type.is_dir())?;
        let identity = DescriptorIdentity::from_metadata(
            child.metadata().map_err(CoordError::io)?,
            file_type.is_dir(),
        )?;
        if !identity.matches_stat(&raw) {
            return Err(object_mismatch("Git object entry changed while opening"));
        }
        nodes.push(identity.node(relative_path));
        if file_type.is_dir() {
            walk_object_directory(&child, &relative, nodes, maximum)?;
        }
        let rebound = statat(
            directory,
            Path::new(OsStr::from_bytes(name)),
            AtFlags::SYMLINK_NOFOLLOW,
        )
        .map_err(|error| object_mismatch(format!("cannot re-stat Git object entry: {error}")))?;
        if DescriptorIdentity::from_metadata(
            child.metadata().map_err(CoordError::io)?,
            file_type.is_dir(),
        )? != identity
            || !identity.matches_stat(&rebound)
        {
            return Err(object_mismatch("Git object entry changed during inventory"));
        }
    }
    if DescriptorIdentity::directory(directory)? != directory_identity {
        return Err(object_mismatch(
            "Git object directory changed during enumeration",
        ));
    }
    Ok(())
}

impl DescriptorIdentity {
    pub(super) fn directory(file: &File) -> Result<Self, CoordError> {
        Self::from_metadata(file.metadata().map_err(CoordError::io)?, true)
    }

    pub(super) fn regular(file: &File) -> Result<Self, CoordError> {
        Self::from_metadata(file.metadata().map_err(CoordError::io)?, false)
    }

    pub(super) fn from_metadata(metadata: Metadata, directory: bool) -> Result<Self, CoordError> {
        if metadata.is_dir() != directory || metadata.is_file() == directory {
            return Err(object_mismatch("retained Git path has an unexpected type"));
        }
        Ok(Self {
            directory,
            device: metadata.dev(),
            inode: metadata.ino(),
            links: metadata.nlink(),
            byte_length: metadata.len(),
            mtime_seconds: metadata.mtime(),
            mtime_nanoseconds: metadata.mtime_nsec(),
            ctime_seconds: metadata.ctime(),
            ctime_nanoseconds: metadata.ctime_nsec(),
        })
    }

    fn matches_stat(&self, stat: &rustix::fs::Stat) -> bool {
        FileType::from_raw_mode(stat.st_mode).is_dir() == self.directory
            && stat.st_dev == self.device
            && stat.st_ino == self.inode
            && stat.st_nlink == self.links
            && u64::try_from(stat.st_size).ok() == Some(self.byte_length)
            && stat.st_mtime == self.mtime_seconds
            && i64::try_from(stat.st_mtime_nsec).ok() == Some(self.mtime_nanoseconds)
            && stat.st_ctime == self.ctime_seconds
            && i64::try_from(stat.st_ctime_nsec).ok() == Some(self.ctime_nanoseconds)
    }

    fn node(&self, relative_path: String) -> ObjectNode {
        ObjectNode {
            relative_path,
            kind: if self.directory { 1 } else { 2 },
            device: self.device,
            inode: self.inode,
            links: self.links,
            byte_length: self.byte_length,
            mtime_seconds: self.mtime_seconds,
            mtime_nanoseconds: self.mtime_nanoseconds,
            ctime_seconds: self.ctime_seconds,
            ctime_nanoseconds: self.ctime_nanoseconds,
        }
    }
}

fn open_object_root(path: &Path) -> Result<File, CoordError> {
    open_object(
        rustix::fs::CWD,
        path,
        true,
        ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
}

fn open_object_directory(parent: &File, path: &Path) -> Result<File, CoordError> {
    open_object(parent, path, true, object_beneath())
}

fn open_object_child(parent: &File, name: &[u8], directory: bool) -> Result<File, CoordError> {
    open_object(
        parent,
        Path::new(OsStr::from_bytes(name)),
        directory,
        object_beneath(),
    )
}

fn open_object(
    parent: impl AsFd,
    path: &Path,
    directory: bool,
    resolve: ResolveFlags,
) -> Result<File, CoordError> {
    let mut flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK;
    if directory {
        flags |= OFlags::DIRECTORY;
    }
    openat2(parent, path, flags, Mode::empty(), resolve)
        .map(File::from)
        .map_err(|error| object_mismatch(format!("cannot retain Git object path: {error}")))
}

fn object_beneath() -> ResolveFlags {
    ResolveFlags::BENEATH
        | ResolveFlags::NO_SYMLINKS
        | ResolveFlags::NO_MAGICLINKS
        | ResolveFlags::NO_XDEV
}

fn object_mismatch(reason: impl Into<String>) -> CoordError {
    CoordError::new("REPOSITORY_IDENTITY_MISMATCH", reason)
}

pub(super) fn commit_paths(
    family_root: &Path,
    repo: &str,
    commit_oid: &str,
) -> Result<ObservedCommitPaths, CoordError> {
    validate_repo_name(repo)?;
    validate_commit_oid(commit_oid)?;
    let before = repository::select(family_root, repo)?;
    let first = read_commit_paths(&before.checkout, commit_oid)?;
    let middle = repository::select(family_root, repo)?;
    let second = read_commit_paths(&middle.checkout, commit_oid)?;
    let after = repository::select(family_root, repo)?;
    if before != middle || middle != after || first != second {
        return Err(CoordError::new(
            "REPOSITORY_IDENTITY_MISMATCH",
            "repository identity or Git objects changed across exact read-back",
        ));
    }
    Ok(ObservedCommitPaths {
        paths: parse_paths(&first)?,
        repository: RepositoryGuard {
            repo: repo.to_owned(),
            selected: after,
        },
    })
}

pub(super) fn verify_repository(
    family_root: &Path,
    repo: &str,
) -> Result<RepositoryGuard, CoordError> {
    let before = repository::select(family_root, repo)?;
    let after = repository::select(family_root, repo)?;
    if before == after {
        Ok(RepositoryGuard {
            repo: repo.to_owned(),
            selected: after,
        })
    } else {
        Err(CoordError::new(
            "REPOSITORY_IDENTITY_MISMATCH",
            "repository identity changed across admission read-back",
        ))
    }
}

pub(super) struct ObservedCommitPaths {
    pub(super) paths: Vec<String>,
    pub(super) repository: RepositoryGuard,
}

pub(super) struct RepositoryGuard {
    repo: String,
    selected: repository::FamilyRepository,
}

impl RepositoryGuard {
    pub(super) fn revalidate(&self, family_root: &Path) -> Result<(), CoordError> {
        let first = repository::select(family_root, &self.repo)?;
        let second = repository::select(family_root, &self.repo)?;
        if self.selected == first && first == second {
            Ok(())
        } else {
            Err(CoordError::new(
                "REPOSITORY_IDENTITY_MISMATCH",
                "repository identity changed before coordination append",
            ))
        }
    }
}

fn read_commit_paths(repo_root: &Path, commit_oid: &str) -> Result<Vec<u8>, CoordError> {
    git(
        repo_root,
        &["cat-file", "-e", &format!("{commit_oid}^{{commit}}")],
    )?;
    git(
        repo_root,
        &[
            "diff-tree",
            "--root",
            "--no-commit-id",
            "--name-only",
            "-z",
            "-r",
            commit_oid,
        ],
    )
}

fn parse_paths(output: &[u8]) -> Result<Vec<String>, CoordError> {
    if !output.is_empty() && output.last() != Some(&0) {
        return Err(CoordError::new(
            "INVALID_GIT_OUTPUT",
            "Git leaf-path output lacks its terminal NUL",
        ));
    }
    let mut actual = output
        .strip_suffix(&[0])
        .unwrap_or_default()
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| {
            let path = std::str::from_utf8(path).map_err(|_| {
                CoordError::new("INVALID_GIT_OUTPUT", "Git emitted a non-UTF-8 leaf path")
            })?;
            validate_path(path).map_err(|error| {
                CoordError::new(
                    "INVALID_GIT_OUTPUT",
                    format!("Git emitted an invalid leaf path: {error}"),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    actual.sort();
    actual.dedup();
    Ok(actual)
}

fn git(repo_root: &Path, args: &[&str]) -> Result<Vec<u8>, CoordError> {
    let output = run_bounded(
        Command::new(GIT_BIN)
            .arg("-C")
            .arg(repo_root)
            .arg("--no-replace-objects")
            .args(args)
            .env_clear()
            .env("LC_ALL", "C")
            .env("LANG", "C")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_NO_LAZY_FETCH", "1")
            .env("GIT_NO_REPLACE_OBJECTS", "1")
            .env("GIT_OPTIONAL_LOCKS", "0"),
        "Git coordination receipt check",
        GIT_LIMITS,
    )?;
    if !output.status.success() {
        return Err(CoordError::new(
            "COMMIT_NOT_FOUND",
            format!(
                "Git could not resolve the receipted commit in {}",
                repo_root.display()
            ),
        ));
    }
    Ok(output.stdout)
}
