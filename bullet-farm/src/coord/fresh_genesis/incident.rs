use std::{
    collections::VecDeque,
    ffi::{OsStr, OsString},
    fs::{File, Metadata},
    io::Read,
    os::unix::{
        ffi::{OsStrExt, OsStringExt},
        fs::MetadataExt,
    },
    path::{Path, PathBuf},
};

use rustix::fs::{AtFlags, FileType, Mode, OFlags, ResolveFlags, openat2, statat};
use sha2::{Digest, Sha256};

use crate::coord::{
    CoordError,
    model::{
        IncidentDirectoryIdentityV1, IncidentInventoryNodeTypeV1, IncidentInventoryNodeV1,
        IncidentInventorySubjectV1, IncidentInventoryV1,
    },
    recovery_manifest::require_normalized_absolute,
};

use super::{changed, invalid};

mod bounds;
use bounds::{add_file_bytes, list_names, require_path_bound, stat_child};

pub(crate) fn observe_incident_inventory(
    source: &Path,
    destination_name: &OsStr,
) -> Result<IncidentInventoryV1, CoordError> {
    require_normalized_absolute(source, "fresh-Genesis incident source")?;
    let destination_name_hex = hex(destination_name.as_bytes());
    IncidentInventoryV1::validate_destination_name_hex(&destination_name_hex)?;
    if source.file_name() == Some(destination_name) {
        return Err(invalid(
            "incident destination must differ from its source name",
        ));
    }
    let source_hex = hex(source.as_os_str().as_bytes());
    require_path_bound(source.as_os_str().as_bytes(), "incident source path")?;
    let retained = RetainedDirectory::open(source)?;
    let snapshot = retained.stable_snapshot()?;
    retained.revalidate(source)?;
    model_from_snapshot(snapshot, source_hex, destination_name_hex)
}

pub(crate) fn verify_retired_incident_inventory(
    expected: &IncidentInventoryV1,
) -> Result<(), CoordError> {
    expected.validate()?;
    let source = PathBuf::from(OsString::from_vec(decode_hex(
        &expected.subject.source_directory.absolute_path_hex,
    )?));
    require_normalized_absolute(&source, "retired incident source")?;
    let parent_path = source
        .parent()
        .ok_or_else(|| invalid("retired incident source has no parent"))?;
    let source_name = source
        .file_name()
        .ok_or_else(|| invalid("retired incident source has no name"))?;
    let destination = OsString::from_vec(decode_hex(&expected.subject.destination_name_hex)?);
    let parent = open_directory(parent_path)?;
    let parent_identity = StableIdentity::directory(&parent)?;
    require_absent(&parent, source_name)?;
    let retained = RetainedDirectory::open_at(&parent, &destination)?;
    let snapshot = retained.stable_snapshot()?;
    retained.revalidate_at(&parent, &destination)?;
    require_absent(&parent, source_name)?;
    if StableIdentity::directory(&parent)? != parent_identity {
        return Err(changed(
            "incident parent changed during retirement verification",
        ));
    }
    let observed = model_from_snapshot(
        snapshot,
        expected.subject.source_directory.absolute_path_hex.clone(),
        expected.subject.destination_name_hex.clone(),
    )?;
    if &observed != expected {
        return Err(changed(
            "retired incident tree differs from its exact sealed inventory",
        ));
    }
    Ok(())
}

struct RetainedDirectory {
    root: File,
    identity: StableIdentity,
}

impl RetainedDirectory {
    fn open(path: &Path) -> Result<Self, CoordError> {
        let parent_path = path
            .parent()
            .ok_or_else(|| invalid("incident source has no parent"))?;
        let name = path
            .file_name()
            .ok_or_else(|| invalid("incident source has no name"))?;
        let parent = open_directory(parent_path)?;
        let parent_identity = StableIdentity::directory(&parent)?;
        let value = Self::open_at(&parent, name)?;
        value.revalidate_at(&parent, name)?;
        if StableIdentity::directory(&parent)? != parent_identity {
            return Err(changed("incident parent changed while opening source"));
        }
        Ok(value)
    }

    fn open_at(parent: &File, name: &OsStr) -> Result<Self, CoordError> {
        let root = File::from(
            openat2(
                parent,
                Path::new(name),
                directory_flags(),
                Mode::empty(),
                beneath(),
            )
            .map_err(|error| invalid(format!("cannot open incident directory: {error}")))?,
        );
        let identity = StableIdentity::directory(&root)?;
        Ok(Self { root, identity })
    }

    fn stable_snapshot(&self) -> Result<Snapshot, CoordError> {
        let first = snapshot(&self.root)?;
        let second = snapshot(&self.root)?;
        if first != second || first.root != self.identity {
            return Err(changed(
                "incident tree changed across complete independent observations",
            ));
        }
        Ok(first)
    }

    fn revalidate(&self, path: &Path) -> Result<(), CoordError> {
        let reopened = Self::open(path)?;
        if reopened.identity != self.identity {
            return Err(changed("incident source pathname identity changed"));
        }
        Ok(())
    }

    fn revalidate_at(&self, parent: &File, name: &OsStr) -> Result<(), CoordError> {
        let stat = stat_child(parent, name.as_bytes())?;
        if !self.identity.matches_stat(&stat) {
            return Err(changed("incident directory pathname identity changed"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Snapshot {
    root: StableIdentity,
    nodes: Vec<StableNode>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StableNode {
    model: IncidentInventoryNodeV1,
    identity: StableIdentity,
}

fn snapshot(root: &File) -> Result<Snapshot, CoordError> {
    let root_identity = StableIdentity::directory(root)?;
    let mut queue = VecDeque::from([(Vec::new(), root_identity.clone())]);
    let mut nodes = Vec::new();
    let mut total_bytes = 0_u64;
    while let Some((directory_path, expected_directory)) = queue.pop_front() {
        let directory = if directory_path.is_empty() {
            root.try_clone().map_err(CoordError::io)?
        } else {
            open_relative(root, &directory_path, true)?
        };
        if StableIdentity::directory(&directory)? != expected_directory {
            return Err(changed("incident directory changed before enumeration"));
        }
        let remaining = IncidentInventoryV1::maximum_nodes()
            .checked_sub(nodes.len())
            .ok_or_else(|| invalid("incident inventory exceeded its node bound"))?;
        for name in list_names(&directory, remaining)? {
            let relative = join(&directory_path, &name);
            require_path_bound(&relative, "incident relative path")?;
            let raw_stat = stat_child(&directory, &name)?;
            let file_type = FileType::from_raw_mode(raw_stat.st_mode);
            let (node_type, identity, digest) = if file_type.is_dir() {
                let child = open_relative(root, &relative, true)?;
                let identity = StableIdentity::directory(&child)?;
                queue.push_back((relative.clone(), identity.clone()));
                (IncidentInventoryNodeTypeV1::Directory, identity, None)
            } else if file_type.is_file() {
                let mut file = open_relative(root, &relative, false)?;
                let before = StableIdentity::regular(&file)?;
                let next_total = add_file_bytes(total_bytes, before.byte_length)?;
                let mut bytes = Vec::new();
                Read::by_ref(&mut file)
                    .take(before.byte_length + 1)
                    .read_to_end(&mut bytes)
                    .map_err(CoordError::io)?;
                let after = StableIdentity::regular(&file)?;
                if before != after || bytes.len() as u64 != before.byte_length {
                    return Err(changed(
                        "incident regular file changed during bounded observation",
                    ));
                }
                total_bytes = next_total;
                (
                    IncidentInventoryNodeTypeV1::RegularFile,
                    before,
                    Some(format!("sha256:{:x}", Sha256::digest(&bytes))),
                )
            } else {
                return Err(invalid(
                    "incident inventory admits only directories and regular files",
                ));
            };
            if !identity.matches_stat(&raw_stat)
                || !identity.matches_stat(&stat_child(&directory, &name)?)
            {
                return Err(changed("incident node pathname changed during observation"));
            }
            nodes.push(StableNode {
                model: IncidentInventoryNodeV1 {
                    relative_path_hex: hex(&relative),
                    node_type,
                    owner_uid: identity.owner_uid,
                    owner_gid: identity.owner_gid,
                    mode: identity.mode,
                    link_count: identity.link_count,
                    byte_length: identity.byte_length,
                    content_sha256: digest,
                },
                identity,
            });
        }
        if StableIdentity::directory(&directory)? != expected_directory
            || (!directory_path.is_empty()
                && StableIdentity::directory(&open_relative(root, &directory_path, true)?)?
                    != expected_directory)
        {
            return Err(changed("incident directory changed during enumeration"));
        }
    }
    if StableIdentity::directory(root)? != root_identity {
        return Err(changed("incident root changed during complete observation"));
    }
    nodes.sort_by(|left, right| {
        left.model
            .relative_path_hex
            .cmp(&right.model.relative_path_hex)
    });
    Ok(Snapshot {
        root: root_identity,
        nodes,
    })
}

fn model_from_snapshot(
    snapshot: Snapshot,
    source_path_hex: String,
    destination_name_hex: String,
) -> Result<IncidentInventoryV1, CoordError> {
    let directory_count = snapshot
        .nodes
        .iter()
        .filter(|node| node.model.node_type == IncidentInventoryNodeTypeV1::Directory)
        .count() as u64;
    let regular_file_count = snapshot.nodes.len() as u64 - directory_count;
    let regular_file_byte_length = snapshot
        .nodes
        .iter()
        .filter(|node| node.model.node_type == IncidentInventoryNodeTypeV1::RegularFile)
        .try_fold(0_u64, |total, node| {
            total.checked_add(node.model.byte_length)
        })
        .ok_or_else(|| invalid("incident byte count overflowed"))?;
    IncidentInventoryV1::from_subject(IncidentInventorySubjectV1 {
        source_directory: IncidentDirectoryIdentityV1 {
            absolute_path_hex: source_path_hex,
            device: snapshot.root.device,
            inode: snapshot.root.inode,
            owner_uid: snapshot.root.owner_uid,
            owner_gid: snapshot.root.owner_gid,
            mode: snapshot.root.mode,
            link_count: snapshot.root.link_count,
            byte_length: snapshot.root.byte_length,
        },
        destination_name_hex,
        node_count: snapshot.nodes.len() as u64,
        directory_count,
        regular_file_count,
        regular_file_byte_length,
        nodes: snapshot.nodes.into_iter().map(|node| node.model).collect(),
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StableIdentity {
    kind: IncidentInventoryNodeTypeV1,
    device: u64,
    inode: u64,
    owner_uid: u32,
    owner_gid: u32,
    mode: u32,
    link_count: u64,
    byte_length: u64,
    mtime_seconds: i64,
    mtime_nanoseconds: i64,
    ctime_seconds: i64,
    ctime_nanoseconds: i64,
}

impl StableIdentity {
    fn directory(file: &File) -> Result<Self, CoordError> {
        Self::from_metadata(file.metadata().map_err(CoordError::io)?, true)
    }

    fn regular(file: &File) -> Result<Self, CoordError> {
        Self::from_metadata(file.metadata().map_err(CoordError::io)?, false)
    }

    fn from_metadata(value: Metadata, directory: bool) -> Result<Self, CoordError> {
        if value.is_dir() != directory || value.is_file() == directory {
            return Err(invalid("incident node has an unexpected filesystem type"));
        }
        Ok(Self {
            kind: if directory {
                IncidentInventoryNodeTypeV1::Directory
            } else {
                IncidentInventoryNodeTypeV1::RegularFile
            },
            device: value.dev(),
            inode: value.ino(),
            owner_uid: value.uid(),
            owner_gid: value.gid(),
            mode: value.mode() & 0o7777,
            link_count: value.nlink(),
            byte_length: value.len(),
            mtime_seconds: value.mtime(),
            mtime_nanoseconds: value.mtime_nsec(),
            ctime_seconds: value.ctime(),
            ctime_nanoseconds: value.ctime_nsec(),
        })
    }

    fn matches_stat(&self, value: &rustix::fs::Stat) -> bool {
        FileType::from_raw_mode(value.st_mode).is_dir()
            == (self.kind == IncidentInventoryNodeTypeV1::Directory)
            && value.st_dev == self.device
            && value.st_ino == self.inode
            && value.st_uid == self.owner_uid
            && value.st_gid == self.owner_gid
            && value.st_mode & 0o7777 == self.mode
            && value.st_nlink == self.link_count
            && u64::try_from(value.st_size).ok() == Some(self.byte_length)
            && value.st_mtime == self.mtime_seconds
            && i64::try_from(value.st_mtime_nsec).ok() == Some(self.mtime_nanoseconds)
            && value.st_ctime == self.ctime_seconds
            && i64::try_from(value.st_ctime_nsec).ok() == Some(self.ctime_nanoseconds)
    }
}

fn open_directory(path: &Path) -> Result<File, CoordError> {
    require_normalized_absolute(path, "incident parent")?;
    openat2(
        rustix::fs::CWD,
        path,
        directory_flags(),
        Mode::empty(),
        ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map(File::from)
    .map_err(|error| invalid(format!("cannot open incident parent: {error}")))
}

fn open_relative(root: &File, relative: &[u8], directory: bool) -> Result<File, CoordError> {
    let mut flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    if directory {
        flags |= OFlags::DIRECTORY;
    } else {
        flags |= OFlags::NONBLOCK;
    }
    openat2(
        root,
        PathBuf::from(OsString::from_vec(relative.to_vec())),
        flags,
        Mode::empty(),
        beneath(),
    )
    .map(File::from)
    .map_err(|error| changed(format!("cannot retain incident node: {error}")))
}

fn require_absent(parent: &File, name: &OsStr) -> Result<(), CoordError> {
    match statat(parent, Path::new(name), AtFlags::SYMLINK_NOFOLLOW) {
        Err(rustix::io::Errno::NOENT) => Ok(()),
        Err(error) => Err(changed(format!(
            "cannot inspect retired source name: {error}"
        ))),
        Ok(_) => Err(changed(
            "incident source name still exists after retirement",
        )),
    }
}

fn join(parent: &[u8], name: &[u8]) -> Vec<u8> {
    if parent.is_empty() {
        name.to_vec()
    } else {
        let mut value = Vec::with_capacity(parent.len() + name.len() + 1);
        value.extend_from_slice(parent);
        value.push(b'/');
        value.extend_from_slice(name);
        value
    }
}

fn hex(value: &[u8]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex(value: &str) -> Result<Vec<u8>, CoordError> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            std::str::from_utf8(pair)
                .ok()
                .and_then(|text| u8::from_str_radix(text, 16).ok())
                .ok_or_else(|| invalid("sealed incident path contains invalid hexadecimal"))
        })
        .collect()
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
