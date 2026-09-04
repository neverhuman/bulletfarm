use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::Read,
    os::unix::{
        ffi::OsStrExt,
        fs::{FileExt, MetadataExt},
    },
    path::{Path, PathBuf},
};

use rustix::fs::{Mode, OFlags, ResolveFlags, openat2};
use serde::Deserialize;
use sha2::Digest;

use super::DescriptorIdentity;
use crate::coord::{CoordError, validate_repo_name};

const MAX_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_HEAD_BYTES: u64 = 4 * 1024;
const MAX_INDEX_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PACKED_REFS_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FamilyRepository {
    family: DescriptorIdentity,
    manifest: DescriptorIdentity,
    manifest_sha256: String,
    pub(super) checkout: PathBuf,
    repository: RepositoryIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RepositoryIdentity {
    root: DescriptorIdentity,
    git: DescriptorIdentity,
    object_store_blake3: String,
}

/// The outer manifest as `select` reads it. Only the name and path are used
/// to resolve a member checkout, so a manifest that predates the Wave-0
/// descriptor fields still resolves here. Wave-0 admission keeps the strict
/// `FamilyManifest` below and fails closed on anything it cannot verify.
#[derive(Deserialize)]
struct OuterManifest {
    #[serde(default)]
    repo: Vec<OuterRepo>,
}

#[derive(Deserialize)]
struct OuterRepo {
    name: String,
    path: PathBuf,
}

#[derive(Deserialize)]
struct FamilyManifest {
    schema_version: String,
    family: String,
    required_repos: Vec<String>,
    repo: Vec<FamilyRepo>,
}

#[derive(Deserialize)]
struct FamilyRepo {
    name: String,
    path: PathBuf,
    jeryu_slug: String,
}

fn validate_wave0_manifest(bytes: &[u8], family_root: &Path) -> Result<(), CoordError> {
    if bytes.last() != Some(&b'\n') {
        return Err(wave0_changed("family manifest lacks its terminal newline"));
    }
    let document =
        std::str::from_utf8(bytes).map_err(|_| wave0_changed("family manifest is not UTF-8"))?;
    let manifest: FamilyManifest = toml::from_str(document)
        .map_err(|error| wave0_changed(format!("invalid family manifest: {error}")))?;
    let exact_members =
        manifest.repo.len() == super::WAVE0_REPOSITORIES.len()
            && manifest.repo.iter().zip(super::WAVE0_REPOSITORIES).all(
                |(entry, (name, identity))| {
                    entry.name == name
                        && entry.jeryu_slug == identity
                        && entry.path == family_root.join(name)
                        && entry.path.is_absolute()
                },
            );
    if manifest.schema_version != "1.2.0"
        || manifest.family != "bullet-farm"
        || manifest.required_repos != super::WAVE0_REPOSITORIES.map(|(name, _)| name.to_owned())
        || !exact_members
    {
        return Err(wave0_changed("family manifest header or members differ"));
    }
    Ok(())
}

pub(super) struct Wave0FamilyGuard {
    root_path: PathBuf,
    pub(super) root: File,
    root_identity: DescriptorIdentity,
    manifest: File,
    manifest_identity: DescriptorIdentity,
    manifest_bytes: Vec<u8>,
    members: Vec<RetainedMember>,
}

struct RetainedMember {
    name: &'static str,
    checkout: File,
    checkout_identity: DescriptorIdentity,
    git: File,
    git_identity: DescriptorIdentity,
    authority_files: Vec<RetainedRegular>,
}

struct RetainedRegular {
    relative_path: PathBuf,
    label: &'static str,
    maximum: u64,
    file: File,
    identity: DescriptorIdentity,
    bytes: Vec<u8>,
}

pub(super) fn select(family_root: &Path, repo: &str) -> Result<FamilyRepository, CoordError> {
    validate_repo_name(repo)?;
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
    if !opened.is_file()
        || opened.dev() != path_meta.dev()
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
    let final_path_meta = fs::symlink_metadata(&manifest_path).map_err(CoordError::io)?;
    if final_path_meta.file_type().is_symlink()
        || !final_path_meta.is_file()
        || final_meta.dev() != opened.dev()
        || final_meta.ino() != opened.ino()
        || final_meta.len() != opened.len()
        || final_meta.ctime() != opened.ctime()
        || final_meta.ctime_nsec() != opened.ctime_nsec()
        || final_path_meta.dev() != opened.dev()
        || final_path_meta.ino() != opened.ino()
    {
        return Err(mismatch("outer repository manifest changed while reading"));
    }
    let document = std::str::from_utf8(&bytes)
        .map_err(|_| mismatch("outer repository manifest is not UTF-8"))?;
    let manifest: OuterManifest = toml::from_str(document)
        .map_err(|error| mismatch(format!("invalid outer repository manifest: {error}")))?;
    let mut names = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut selected = None;
    for entry in manifest.repo {
        validate_repo_name(&entry.name)
            .map_err(|error| mismatch(format!("invalid repository manifest name: {error}")))?;
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
    if !declared.is_absolute() || declared != expected {
        return Err(mismatch(
            "outer repository manifest name and checkout path do not match",
        ));
    }
    Ok(FamilyRepository {
        family: DescriptorIdentity::from_metadata(root_meta, true)?,
        manifest: DescriptorIdentity::from_metadata(opened, false)?,
        manifest_sha256: format!("sha256:{:x}", sha2::Sha256::digest(&bytes)),
        checkout: expected.clone(),
        repository: repository_identity(&expected)?,
    })
}

impl Wave0FamilyGuard {
    pub(super) fn open(family_root: &Path) -> Result<Self, CoordError> {
        crate::coord::recovery_manifest::require_normalized_absolute(
            family_root,
            "Wave-0 family root",
        )?;
        if family_root.as_os_str().as_bytes().len() > 4096 {
            return Err(mismatch("Wave-0 family root exceeds 4,096 bytes"));
        }
        let root = open_absolute_directory(family_root)?;
        let root_identity = DescriptorIdentity::directory(&root)?;
        let manifest = open_relative_regular(&root, Path::new("repos.manifest.toml"))?;
        let (manifest_identity, manifest_bytes) =
            stable_regular(&manifest, MAX_MANIFEST_BYTES, "family manifest")?;
        validate_wave0_manifest(&manifest_bytes, family_root)?;
        let members = super::WAVE0_REPOSITORIES
            .iter()
            .map(|(name, _)| retain_member(&root, name))
            .collect::<Result<Vec<_>, _>>()?;
        let value = Self {
            root_path: family_root.to_path_buf(),
            root,
            root_identity,
            manifest,
            manifest_identity,
            manifest_bytes,
            members,
        };
        value.revalidate()?;
        Ok(value)
    }

    pub(super) fn revalidate(&self) -> Result<(), CoordError> {
        if DescriptorIdentity::directory(&self.root)? != self.root_identity
            || DescriptorIdentity::directory(&open_absolute_directory(&self.root_path)?)?
                != self.root_identity
        {
            return Err(mismatch("Wave-0 family root identity changed"));
        }
        let (retained_identity, retained_bytes) =
            stable_regular(&self.manifest, MAX_MANIFEST_BYTES, "family manifest")?;
        let reopened = open_relative_regular(&self.root, Path::new("repos.manifest.toml"))?;
        let (reopened_identity, reopened_bytes) =
            stable_regular(&reopened, MAX_MANIFEST_BYTES, "family manifest")?;
        if retained_identity != self.manifest_identity
            || reopened_identity != self.manifest_identity
            || retained_bytes != self.manifest_bytes
            || reopened_bytes != self.manifest_bytes
        {
            return Err(mismatch("Wave-0 family manifest changed"));
        }
        for member in &self.members {
            let checkout = open_relative_directory(&self.root, Path::new(member.name))?;
            let git = open_relative_directory(&checkout, Path::new(".git"))?;
            if DescriptorIdentity::directory(&member.checkout)? != member.checkout_identity
                || DescriptorIdentity::directory(&member.git)? != member.git_identity
                || DescriptorIdentity::directory(&checkout)? != member.checkout_identity
                || DescriptorIdentity::directory(&git)? != member.git_identity
            {
                return Err(mismatch(format!(
                    "Wave-0 {} checkout identity changed",
                    member.name
                )));
            }
            for authority in &member.authority_files {
                authority.revalidate(&member.git, &git)?;
            }
        }
        Ok(())
    }
}

fn retain_member(root: &File, name: &'static str) -> Result<RetainedMember, CoordError> {
    let checkout = open_relative_directory(root, Path::new(name))?;
    let git = open_relative_directory(&checkout, Path::new(".git"))?;
    let head = RetainedRegular::open(&git, Path::new("HEAD"), MAX_HEAD_BYTES, "HEAD")?;
    let index = RetainedRegular::open(&git, Path::new("index"), MAX_INDEX_BYTES, "index")?;
    let resolved = match super::wave0::head_reference(&head.bytes)? {
        Some(path) => match RetainedRegular::try_open(
            &git,
            Path::new(path),
            MAX_HEAD_BYTES,
            "resolved HEAD ref",
        )? {
            Some(value) => Some(value),
            None => Some(RetainedRegular::open(
                &git,
                Path::new("packed-refs"),
                MAX_PACKED_REFS_BYTES,
                "packed HEAD refs",
            )?),
        },
        None => None,
    };
    let mut authority_files = vec![head, index];
    authority_files.extend(resolved);
    Ok(RetainedMember {
        name,
        checkout_identity: DescriptorIdentity::directory(&checkout)?,
        git_identity: DescriptorIdentity::directory(&git)?,
        checkout,
        git,
        authority_files,
    })
}

impl RetainedRegular {
    fn open(
        parent: &File,
        relative_path: &Path,
        maximum: u64,
        label: &'static str,
    ) -> Result<Self, CoordError> {
        let file = open_relative_regular(parent, relative_path)?;
        Self::from_file(file, relative_path, maximum, label)
    }

    fn try_open(
        parent: &File,
        relative_path: &Path,
        maximum: u64,
        label: &'static str,
    ) -> Result<Option<Self>, CoordError> {
        let Some(file) = try_open_relative_regular(parent, relative_path)? else {
            return Ok(None);
        };
        Self::from_file(file, relative_path, maximum, label).map(Some)
    }

    fn from_file(
        file: File,
        relative_path: &Path,
        maximum: u64,
        label: &'static str,
    ) -> Result<Self, CoordError> {
        let (identity, bytes) = stable_regular(&file, maximum, label)?;
        Ok(Self {
            relative_path: relative_path.to_path_buf(),
            label,
            maximum,
            file,
            identity,
            bytes,
        })
    }

    fn revalidate(&self, retained_parent: &File, reopened_parent: &File) -> Result<(), CoordError> {
        let (identity, bytes) = stable_regular(&self.file, self.maximum, self.label)?;
        if identity != self.identity || bytes != self.bytes {
            return Err(mismatch(format!("Wave-0 retained {} changed", self.label)));
        }
        for parent in [retained_parent, reopened_parent] {
            let reopened = open_relative_regular(parent, &self.relative_path)?;
            let (identity, bytes) = stable_regular(&reopened, self.maximum, self.label)?;
            if identity != self.identity || bytes != self.bytes {
                return Err(mismatch(format!(
                    "Wave-0 {} pathname identity changed",
                    self.label
                )));
            }
        }
        Ok(())
    }
}

fn stable_regular(
    file: &File,
    maximum: u64,
    label: &str,
) -> Result<(DescriptorIdentity, Vec<u8>), CoordError> {
    let first_identity = DescriptorIdentity::regular(file)?;
    if first_identity.links != 1
        || first_identity.byte_length == 0
        || first_identity.byte_length > maximum
    {
        return Err(mismatch(format!("Wave-0 {label} framing is not admitted")));
    }
    let first = read_at(file, first_identity.byte_length)?;
    let middle_identity = DescriptorIdentity::regular(file)?;
    let second = read_at(file, first_identity.byte_length)?;
    let final_identity = DescriptorIdentity::regular(file)?;
    if first_identity != middle_identity || middle_identity != final_identity || first != second {
        return Err(mismatch(format!(
            "Wave-0 {label} changed during stable read"
        )));
    }
    Ok((final_identity, second))
}

fn read_at(file: &File, length: u64) -> Result<Vec<u8>, CoordError> {
    let length = usize::try_from(length)
        .map_err(|_| mismatch("Wave-0 retained file does not fit this host"))?;
    let mut bytes = vec![0_u8; length];
    let mut offset = 0;
    while offset < bytes.len() {
        let read = file
            .read_at(&mut bytes[offset..], offset as u64)
            .map_err(CoordError::io)?;
        if read == 0 {
            return Err(mismatch("Wave-0 retained file was truncated during read"));
        }
        offset += read;
    }
    Ok(bytes)
}

fn open_absolute_directory(path: &Path) -> Result<File, CoordError> {
    openat2(
        rustix::fs::CWD,
        path,
        directory_flags(),
        Mode::empty(),
        ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map(File::from)
    .map_err(|error| mismatch(format!("cannot retain Wave-0 family root: {error}")))
}

fn open_relative_directory(parent: &File, path: &Path) -> Result<File, CoordError> {
    openat2(parent, path, directory_flags(), Mode::empty(), beneath())
        .map(File::from)
        .map_err(|error| mismatch(format!("cannot retain Wave-0 directory: {error}")))
}

fn open_relative_regular(parent: &File, path: &Path) -> Result<File, CoordError> {
    openat2(
        parent,
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
        Mode::empty(),
        beneath(),
    )
    .map(File::from)
    .map_err(|error| mismatch(format!("cannot retain Wave-0 regular file: {error}")))
}

fn try_open_relative_regular(parent: &File, path: &Path) -> Result<Option<File>, CoordError> {
    match openat2(
        parent,
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
        Mode::empty(),
        beneath(),
    ) {
        Ok(file) => Ok(Some(File::from(file))),
        Err(rustix::io::Errno::NOENT) => Ok(None),
        Err(error) => Err(mismatch(format!(
            "cannot retain optional Wave-0 regular file: {error}"
        ))),
    }
}

fn directory_flags() -> OFlags {
    OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC
}

fn beneath() -> ResolveFlags {
    ResolveFlags::BENEATH
        | ResolveFlags::NO_SYMLINKS
        | ResolveFlags::NO_MAGICLINKS
        | ResolveFlags::NO_XDEV
}

fn repository_identity(root: &Path) -> Result<RepositoryIdentity, CoordError> {
    let root_metadata = fs::symlink_metadata(root).map_err(CoordError::io)?;
    let git_path = root.join(".git");
    let git_metadata = fs::symlink_metadata(&git_path).map_err(CoordError::io)?;
    if !root_metadata.is_dir()
        || root_metadata.file_type().is_symlink()
        || !git_metadata.is_dir()
        || git_metadata.file_type().is_symlink()
    {
        return Err(mismatch(
            "ordinary Git subject must be a primary repository directory",
        ));
    }
    let root_identity = DescriptorIdentity::from_metadata(root_metadata, true)?;
    let git_identity = DescriptorIdentity::from_metadata(git_metadata, true)?;
    let object_store_blake3 = super::descriptor_object_inventory(root)?;
    let final_root = fs::symlink_metadata(root).map_err(CoordError::io)?;
    let final_git = fs::symlink_metadata(&git_path).map_err(CoordError::io)?;
    if DescriptorIdentity::from_metadata(final_root, true)? != root_identity
        || DescriptorIdentity::from_metadata(final_git, true)? != git_identity
    {
        return Err(mismatch(
            "repository identity changed while inventorying Git",
        ));
    }
    Ok(RepositoryIdentity {
        root: root_identity,
        git: git_identity,
        object_store_blake3,
    })
}

pub(super) fn mismatch(reason: impl Into<String>) -> CoordError {
    CoordError::new("REPOSITORY_IDENTITY_MISMATCH", reason)
}

fn wave0_changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("WAVE0_OBSERVATION_CHANGED", reason)
}
