use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::Read,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use super::{RepositoryIdentity, mismatch, repository_identity, sha256};
use crate::coord::CoordError;

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FamilySelection {
    family_device: u64,
    family_inode: u64,
    manifest_device: u64,
    manifest_inode: u64,
    manifest_ctime_seconds: i64,
    manifest_ctime_nanoseconds: i64,
    manifest_sha256: String,
    pub(super) checkout: PathBuf,
    repository: RepositoryIdentity,
}

#[derive(Deserialize)]
struct FamilyManifest {
    #[serde(default)]
    repo: Vec<FamilyRepo>,
}

#[derive(Deserialize)]
struct FamilyRepo {
    name: String,
    path: PathBuf,
}

pub(super) fn family_selection(
    family_root: &Path,
    repo: &str,
) -> Result<FamilySelection, CoordError> {
    let root_meta = fs::symlink_metadata(family_root).map_err(CoordError::io)?;
    if !root_meta.is_dir() || root_meta.file_type().is_symlink() {
        return Err(mismatch("family root is not a direct directory"));
    }
    let manifest_path = family_root.join("repos.manifest.toml");
    let path_meta = fs::symlink_metadata(&manifest_path).map_err(CoordError::io)?;
    if !path_meta.is_file()
        || path_meta.file_type().is_symlink()
        || path_meta.len() > MAX_MANIFEST_BYTES
    {
        return Err(mismatch(
            "outer repository manifest is not a bounded regular file",
        ));
    }
    let mut file = File::open(&manifest_path).map_err(CoordError::io)?;
    let opened = file.metadata().map_err(CoordError::io)?;
    if opened.dev() != path_meta.dev()
        || opened.ino() != path_meta.ino()
        || opened.len() != path_meta.len()
    {
        return Err(mismatch("outer repository manifest changed while opening"));
    }
    let capacity = usize::try_from(opened.len())
        .map_err(|_| mismatch("outer repository manifest does not fit this host"))?;
    let mut bytes = Vec::with_capacity(capacity);
    file.by_ref()
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    if bytes.is_empty() || bytes.len() > MAX_MANIFEST_BYTES as usize || bytes.len() != capacity {
        return Err(mismatch(
            "outer repository manifest length changed or exceeds its bound",
        ));
    }
    let final_meta = file.metadata().map_err(CoordError::io)?;
    if final_meta.dev() != opened.dev()
        || final_meta.ino() != opened.ino()
        || final_meta.len() != opened.len()
        || final_meta.ctime() != opened.ctime()
        || final_meta.ctime_nsec() != opened.ctime_nsec()
    {
        return Err(mismatch("outer repository manifest changed while reading"));
    }
    let document = std::str::from_utf8(&bytes)
        .map_err(|_| mismatch("outer repository manifest is not UTF-8"))?;
    let manifest: FamilyManifest = toml::from_str(document)
        .map_err(|error| mismatch(format!("invalid outer repository manifest: {error}")))?;
    let mut names = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut selected = None;
    for entry in manifest.repo {
        if !names.insert(entry.name.clone()) || !paths.insert(entry.path.clone()) {
            return Err(mismatch(
                "outer repository manifest contains duplicate names or paths",
            ));
        }
        if entry.name == repo {
            selected = Some(entry.path);
        }
    }
    let declared = selected
        .ok_or_else(|| mismatch("repository is not listed in the outer family manifest"))?;
    let expected = family_root.join(repo);
    if !declared.is_absolute()
        || declared != expected
        || fs::canonicalize(&declared).map_err(CoordError::io)?
            != fs::canonicalize(&expected).map_err(CoordError::io)?
    {
        return Err(mismatch(
            "outer repository manifest name and checkout path do not match",
        ));
    }
    Ok(FamilySelection {
        family_device: root_meta.dev(),
        family_inode: root_meta.ino(),
        manifest_device: opened.dev(),
        manifest_inode: opened.ino(),
        manifest_ctime_seconds: opened.ctime(),
        manifest_ctime_nanoseconds: opened.ctime_nsec(),
        manifest_sha256: sha256(&bytes),
        checkout: expected.clone(),
        repository: repository_identity(&expected)?,
    })
}
