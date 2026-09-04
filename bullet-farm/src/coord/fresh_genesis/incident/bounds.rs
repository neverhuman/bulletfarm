use std::{collections::BTreeSet, ffi::OsStr, fs::File, os::unix::ffi::OsStrExt, path::Path};

use rustix::fs::{AtFlags, Stat, statat};

use crate::coord::{CoordError, model::IncidentInventoryV1};

use super::{changed, invalid};

const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 256 * 1024 * 1024;

pub(super) fn list_names(
    directory: &File,
    remaining_nodes: usize,
) -> Result<Vec<Vec<u8>>, CoordError> {
    let mut reader = rustix::fs::Dir::read_from(directory)
        .map_err(|error| changed(format!("cannot inventory incident directory: {error}")))?;
    let mut names = BTreeSet::new();
    while let Some(entry) = reader.read() {
        let entry =
            entry.map_err(|error| changed(format!("cannot read incident directory: {error}")))?;
        let name = entry.file_name().to_bytes();
        if name == b"." || name == b".." {
            continue;
        }
        if name.is_empty() || name.contains(&b'/') {
            return Err(changed(
                "incident directory contains an unsafe or duplicate name",
            ));
        }
        if names.len() == remaining_nodes {
            return Err(invalid(
                "incident directory exceeds the remaining global node budget",
            ));
        }
        if !names.insert(name.to_vec()) {
            return Err(changed("incident directory contains a duplicate name"));
        }
    }
    Ok(names.into_iter().collect())
}

pub(super) fn stat_child(parent: &File, name: &[u8]) -> Result<Stat, CoordError> {
    if name.is_empty() || name.contains(&b'/') || matches!(name, b"." | b"..") {
        return Err(invalid(
            "incident child stat requires exactly one normalized component",
        ));
    }
    statat(
        parent,
        Path::new(OsStr::from_bytes(name)),
        AtFlags::SYMLINK_NOFOLLOW,
    )
    .map_err(|error| changed(format!("cannot stat retained incident child: {error}")))
}

pub(super) fn require_path_bound(bytes: &[u8], label: &str) -> Result<(), CoordError> {
    if bytes.len() > IncidentInventoryV1::maximum_path_bytes() {
        return Err(invalid(format!("{label} exceeds its closed byte bound")));
    }
    Ok(())
}

pub(super) fn add_file_bytes(total: u64, file_length: u64) -> Result<u64, CoordError> {
    if file_length > MAX_FILE_BYTES {
        return Err(invalid("incident regular file exceeds its byte bound"));
    }
    total
        .checked_add(file_length)
        .filter(|value| *value <= MAX_TOTAL_BYTES)
        .ok_or_else(|| invalid("incident regular-file aggregate exceeds its byte bound"))
}

#[cfg(test)]
mod tests {
    use std::{fs, fs::File, os::unix::fs::MetadataExt};

    use super::*;

    #[test]
    fn enumeration_refuses_before_retaining_entry_2049() {
        let root = tempfile::tempdir().unwrap();
        for index in 0..IncidentInventoryV1::maximum_nodes() {
            fs::write(root.path().join(format!("node-{index:04}")), b"x").unwrap();
        }
        let directory = File::open(root.path()).unwrap();
        assert_eq!(
            list_names(&directory, IncidentInventoryV1::maximum_nodes())
                .unwrap()
                .len(),
            IncidentInventoryV1::maximum_nodes()
        );
        fs::write(root.path().join("node-extra"), b"x").unwrap();
        let directory = File::open(root.path()).unwrap();
        assert!(list_names(&directory, IncidentInventoryV1::maximum_nodes()).is_err());
    }

    #[test]
    fn component_path_file_and_aggregate_boundaries_are_exact() {
        let root = tempfile::tempdir().unwrap();
        let directory = File::open(root.path()).unwrap();
        assert!(stat_child(&directory, b"intermediate/child").is_err());

        let maximum_path = IncidentInventoryV1::maximum_path_bytes();
        assert!(require_path_bound(&vec![b'x'; maximum_path], "path").is_ok());
        assert!(require_path_bound(&vec![b'x'; maximum_path + 1], "path").is_err());

        assert_eq!(add_file_bytes(0, MAX_FILE_BYTES).unwrap(), MAX_FILE_BYTES);
        assert!(add_file_bytes(0, MAX_FILE_BYTES + 1).is_err());
        assert_eq!(
            add_file_bytes(MAX_TOTAL_BYTES - MAX_FILE_BYTES, MAX_FILE_BYTES).unwrap(),
            MAX_TOTAL_BYTES
        );
        assert!(add_file_bytes(MAX_TOTAL_BYTES, 1).is_err());
    }

    #[test]
    fn retained_parent_stat_does_not_follow_substituted_intermediate_path() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("source");
        let outside = root.path().join("outside");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&outside).unwrap();
        fs::write(source.join("child"), b"inside").unwrap();
        fs::write(outside.join("child"), b"outside").unwrap();
        let retained = File::open(&source).unwrap();
        let moved = root.path().join("moved");
        fs::rename(&source, &moved).unwrap();
        std::os::unix::fs::symlink(&outside, &source).unwrap();

        let observed = stat_child(&retained, b"child").unwrap();
        assert_eq!(
            observed.st_ino,
            fs::metadata(moved.join("child")).unwrap().ino()
        );
        assert_ne!(
            observed.st_ino,
            fs::metadata(outside.join("child")).unwrap().ino()
        );
    }
}
