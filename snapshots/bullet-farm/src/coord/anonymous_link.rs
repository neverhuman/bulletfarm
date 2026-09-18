#[cfg(test)]
use std::cell::RefCell;
use std::{fs::File, os::fd::AsRawFd, path::PathBuf};

use rustix::fs::{AtFlags, CWD, Mode, OFlags, PROC_SUPER_MAGIC, fstatfs, linkat, openat, statat};

#[derive(Debug, Eq, PartialEq)]
pub(super) enum LinkOutcome {
    Linked,
    Exists,
}

#[cfg(test)]
thread_local! {
    static PROC_FD_DIRECTORY_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static PROC_FD_ENTRY_OVERRIDE: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub(super) fn link(
    anonymous: &File,
    parent: &File,
    name: &str,
    expected_identity: (u64, u64),
) -> Result<LinkOutcome, String> {
    let proc_fd_directory = proc_fd_directory();
    let proc_fds = openat(
        CWD,
        &proc_fd_directory,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| {
        format!(
            "cannot open admitted proc descriptor directory {}: {error}",
            proc_fd_directory.display()
        )
    })?;
    let filesystem = fstatfs(&proc_fds)
        .map_err(|error| format!("cannot identify proc descriptor filesystem: {error}"))?;
    if filesystem.f_type != PROC_SUPER_MAGIC {
        return Err("proc descriptor directory is not procfs".to_owned());
    }

    let entry = proc_fd_entry(anonymous);
    let followed = statat(&proc_fds, &entry, AtFlags::empty())
        .map_err(|error| format!("cannot bind retained descriptor through procfs: {error}"))?;
    if (followed.st_dev, followed.st_ino) != expected_identity {
        return Err("proc descriptor entry differs from retained anonymous inode".to_owned());
    }

    match linkat(&proc_fds, &entry, parent, name, AtFlags::SYMLINK_FOLLOW) {
        Ok(()) => Ok(LinkOutcome::Linked),
        Err(rustix::io::Errno::EXIST) => Ok(LinkOutcome::Exists),
        Err(error) => Err(format!(
            "cannot publish retained anonymous inode through procfs: {error}"
        )),
    }
}

fn proc_fd_directory() -> PathBuf {
    #[cfg(test)]
    if let Some(path) = PROC_FD_DIRECTORY_OVERRIDE.with(|value| value.borrow().clone()) {
        return path;
    }
    PathBuf::from("/proc/self/fd")
}

fn proc_fd_entry(anonymous: &File) -> String {
    #[cfg(test)]
    if let Some(entry) = PROC_FD_ENTRY_OVERRIDE.with(|value| value.borrow().clone()) {
        return entry;
    }
    anonymous.as_raw_fd().to_string()
}

#[cfg(test)]
struct ProcFdOverride {
    directory: Option<PathBuf>,
    entry: Option<String>,
}

#[cfg(test)]
impl ProcFdOverride {
    fn install(directory: Option<PathBuf>, entry: Option<String>) -> Self {
        let directory = PROC_FD_DIRECTORY_OVERRIDE.with(|value| value.replace(directory));
        let entry = PROC_FD_ENTRY_OVERRIDE.with(|value| value.replace(entry));
        Self { directory, entry }
    }
}

#[cfg(test)]
impl Drop for ProcFdOverride {
    fn drop(&mut self) {
        PROC_FD_DIRECTORY_OVERRIDE.with(|value| {
            value.replace(self.directory.take());
        });
        PROC_FD_ENTRY_OVERRIDE.with(|value| {
            value.replace(self.entry.take());
        });
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{Read, Write},
        os::unix::fs::{MetadataExt, PermissionsExt, symlink},
    };

    use super::*;

    fn parent(root: &tempfile::TempDir) -> File {
        File::open(root.path()).unwrap()
    }

    fn anonymous_file(parent: &File, bytes: &[u8]) -> File {
        let descriptor = openat(
            parent,
            ".",
            OFlags::TMPFILE | OFlags::RDWR | OFlags::CLOEXEC,
            Mode::RUSR | Mode::WUSR,
        )
        .unwrap();
        let mut file = File::from(descriptor);
        file.write_all(bytes).unwrap();
        file.sync_all().unwrap();
        file
    }

    fn identity(file: &File) -> (u64, u64) {
        let metadata = file.metadata().unwrap();
        (metadata.dev(), metadata.ino())
    }

    #[test]
    fn procfs_link_is_exact_and_never_replaces() {
        let root = tempfile::tempdir().unwrap();
        let parent = parent(&root);
        let first = anonymous_file(&parent, b"first");
        assert_eq!(
            link(&first, &parent, "published", identity(&first)).unwrap(),
            LinkOutcome::Linked
        );
        assert_eq!(
            identity(&File::open(root.path().join("published")).unwrap()),
            identity(&first)
        );

        let second = anonymous_file(&parent, b"second");
        assert_eq!(
            link(&second, &parent, "published", identity(&second)).unwrap(),
            LinkOutcome::Exists
        );
        let mut bytes = Vec::new();
        File::open(root.path().join("published"))
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        assert_eq!(bytes, b"first");
    }

    #[test]
    fn absent_procfs_refuses_before_publication() {
        let root = tempfile::tempdir().unwrap();
        let parent = parent(&root);
        let anonymous = anonymous_file(&parent, b"bytes");
        let _override = ProcFdOverride::install(Some(root.path().join("missing")), None);
        assert!(link(&anonymous, &parent, "published", identity(&anonymous)).is_err());
        assert!(!root.path().join("published").exists());
    }

    #[test]
    fn inaccessible_procfs_refuses_before_publication() {
        let root = tempfile::tempdir().unwrap();
        let hidden = root.path().join("hidden");
        fs::create_dir(&hidden).unwrap();
        fs::set_permissions(&hidden, fs::Permissions::from_mode(0o0)).unwrap();
        let parent = parent(&root);
        let anonymous = anonymous_file(&parent, b"bytes");
        let _override = ProcFdOverride::install(Some(hidden.clone()), None);
        assert!(link(&anonymous, &parent, "published", identity(&anonymous)).is_err());
        assert!(!root.path().join("published").exists());
        fs::set_permissions(hidden, fs::Permissions::from_mode(0o700)).unwrap();
    }

    #[test]
    fn non_procfs_and_symlink_substitution_refuse_before_publication() {
        for symlinked in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let substitute = root.path().join("substitute");
            if symlinked {
                symlink("/proc/self/fd", &substitute).unwrap();
            } else {
                fs::create_dir(&substitute).unwrap();
            }
            let parent = parent(&root);
            let anonymous = anonymous_file(&parent, b"bytes");
            let _override = ProcFdOverride::install(Some(substitute), None);
            assert!(link(&anonymous, &parent, "published", identity(&anonymous)).is_err());
            assert!(!root.path().join("published").exists());
        }
    }

    #[test]
    fn wrong_proc_descriptor_refuses_before_publication() {
        let root = tempfile::tempdir().unwrap();
        let parent = parent(&root);
        let anonymous = anonymous_file(&parent, b"bytes");
        let wrong = anonymous_file(&parent, b"wrong");
        let _override = ProcFdOverride::install(None, Some(wrong.as_raw_fd().to_string()));
        assert!(link(&anonymous, &parent, "published", identity(&anonymous)).is_err());
        assert!(!root.path().join("published").exists());
    }
}
