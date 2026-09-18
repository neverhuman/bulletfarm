use std::{fs, os::unix::fs::MetadataExt, path::Path};

use serde::Serialize;

use super::mismatch;
use crate::coord::CoordError;

const MAX_OBJECT_NODES: usize = 1_000_000;

#[derive(Serialize)]
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

pub(super) fn inventory(repo_root: &Path) -> Result<String, CoordError> {
    let root = repo_root.join(".git/objects");
    let root_meta = fs::symlink_metadata(&root).map_err(CoordError::io)?;
    if !root_meta.is_dir() || root_meta.file_type().is_symlink() {
        return Err(mismatch("Git object store is not a direct directory"));
    }
    let mut pending = vec![root.clone()];
    let mut nodes = Vec::new();
    while let Some(directory) = pending.pop() {
        let mut children = fs::read_dir(&directory)
            .map_err(CoordError::io)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(CoordError::io)?;
        children.sort_by_key(fs::DirEntry::file_name);
        for child in children {
            let path = child.path();
            let metadata = fs::symlink_metadata(&path).map_err(CoordError::io)?;
            let kind = if metadata.is_dir() {
                1
            } else if metadata.is_file() {
                2
            } else {
                return Err(mismatch(
                    "Git object store contains a special or symbolic node",
                ));
            };
            let relative = path
                .strip_prefix(&root)
                .map_err(|_| mismatch("Git object path escaped its store"))?;
            let relative_path = relative
                .to_str()
                .ok_or_else(|| mismatch("Git object path is not UTF-8"))?
                .replace(std::path::MAIN_SEPARATOR, "/");
            if relative_path.is_empty() || relative_path.len() > 512 {
                return Err(mismatch("Git object path exceeds its closed bound"));
            }
            if relative_path == "info/alternates" {
                return Err(mismatch(
                    "Git alternate object stores are not admitted for recovery evidence",
                ));
            }
            nodes.push(ObjectNode {
                relative_path,
                kind,
                device: metadata.dev(),
                inode: metadata.ino(),
                links: metadata.nlink(),
                byte_length: metadata.len(),
                mtime_seconds: metadata.mtime(),
                mtime_nanoseconds: metadata.mtime_nsec(),
                ctime_seconds: metadata.ctime(),
                ctime_nanoseconds: metadata.ctime_nsec(),
            });
            if nodes.len() > MAX_OBJECT_NODES {
                return Err(mismatch(
                    "Git object store exceeds its closed inventory bound",
                ));
            }
            if metadata.is_dir() {
                pending.push(path);
            }
        }
    }
    nodes.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(format!(
        "blake3:{}",
        bullet_wire::hash_canonical("bullet-family.coord.git-object-store.v1", &nodes)
            .map_err(|error| mismatch(format!("cannot inventory Git object store: {error}")))?
            .to_hex()
    ))
}
