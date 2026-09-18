//! Descriptor-retained corpus and seed custody for the replay binary.

use std::path::{Path, PathBuf};

/// Deterministic hostile-test seams around a same-descriptor read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReadStage {
    /// The no-follow seed metadata is bound, immediately before open-at.
    BeforeOpen,
    /// The seed descriptor is identity-bound, immediately before reading.
    AfterOpen,
    /// At least one bounded chunk has been read from the retained descriptor.
    AfterFirstRead,
}

#[cfg(unix)]
mod imp {
    use super::{Path, PathBuf, ReadStage};
    use rustix::fs::{AtFlags, CWD, Dir, FileType, Mode, OFlags, Stat, fstat, openat, statat};
    use std::fs::File;
    use std::io::Read;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct Identity {
        dev: u128,
        ino: u128,
        mode: u128,
        links: u128,
        uid: u128,
        gid: u128,
        size: i128,
        modified: i128,
        modified_ns: i128,
        changed: i128,
        changed_ns: i128,
    }

    impl Identity {
        fn from_stat(stat: &Stat) -> Self {
            Self {
                dev: u128::from(stat.st_dev),
                ino: u128::from(stat.st_ino),
                mode: u128::from(stat.st_mode),
                links: u128::from(stat.st_nlink),
                uid: u128::from(stat.st_uid),
                gid: u128::from(stat.st_gid),
                size: i128::from(stat.st_size),
                modified: i128::from(stat.st_mtime),
                modified_ns: i128::from(stat.st_mtime_nsec),
                changed: i128::from(stat.st_ctime),
                changed_ns: i128::from(stat.st_ctime_nsec),
            }
        }
    }

    /// One retained corpus directory and its exact opening identity.
    pub(crate) struct CorpusDir {
        path: PathBuf,
        descriptor: File,
        identity: Identity,
    }

    impl CorpusDir {
        pub(crate) fn open(path: &Path) -> Result<Self, String> {
            let inspected = statat(CWD, path, AtFlags::SYMLINK_NOFOLLOW)
                .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
            if FileType::from_raw_mode(inspected.st_mode) != FileType::Directory {
                return Err(format!(
                    "{} is not a non-symlink corpus directory",
                    path.display()
                ));
            }
            let descriptor = openat(
                CWD,
                path,
                OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map(File::from)
            .map_err(|error| format!("cannot open {}: {error}", path.display()))?;
            let opened = fstat(&descriptor)
                .map_err(|error| format!("cannot bind {}: {error}", path.display()))?;
            let identity = Identity::from_stat(&inspected);
            ensure_identity(path, identity, Identity::from_stat(&opened))?;
            Ok(Self {
                path: path.to_path_buf(),
                descriptor,
                identity,
            })
        }

        pub(crate) fn inventory(&self) -> Result<Vec<String>, String> {
            let mut names = Vec::new();
            let entries = Dir::read_from(&self.descriptor)
                .map_err(|error| format!("cannot read {}: {error}", self.path.display()))?;
            for entry in entries {
                let entry = entry.map_err(|error| {
                    format!("cannot enumerate {}: {error}", self.path.display())
                })?;
                let name = entry
                    .file_name()
                    .to_str()
                    .map_err(|_| format!("non-UTF-8 corpus name in {}", self.path.display()))?;
                if name != "." && name != ".." {
                    names.push(name.to_string());
                }
            }
            names.sort();
            self.revalidate()?;
            Ok(names)
        }

        pub(crate) fn read_seed(
            &self,
            name: &str,
            max_bytes: u64,
            mut hook: impl FnMut(&Path, ReadStage),
        ) -> Result<Vec<u8>, String> {
            if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\0']) {
                return Err(format!("invalid corpus filename {name:?}"));
            }
            let public_path = self.path.join(name);
            let inspected = statat(&self.descriptor, name, AtFlags::SYMLINK_NOFOLLOW)
                .map_err(|error| format!("cannot inspect {}: {error}", public_path.display()))?;
            let identity = Identity::from_stat(&inspected);
            if FileType::from_raw_mode(inspected.st_mode) != FileType::RegularFile
                || identity.links != 1
            {
                return Err(format!(
                    "{} is not a regular single-link corpus file",
                    public_path.display()
                ));
            }
            if identity.size < 0 || u64::try_from(identity.size).unwrap_or(u64::MAX) > max_bytes {
                return Err(format!(
                    "{} exceeds the {max_bytes}-byte replay bound",
                    public_path.display()
                ));
            }

            hook(&public_path, ReadStage::BeforeOpen);
            let descriptor = openat(
                &self.descriptor,
                name,
                OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
                Mode::empty(),
            )
            .map(File::from)
            .map_err(|error| format!("cannot open {}: {error}", public_path.display()))?;
            let opened = fstat(&descriptor)
                .map_err(|error| format!("cannot bind {}: {error}", public_path.display()))?;
            ensure_identity(&public_path, identity, Identity::from_stat(&opened))?;
            hook(&public_path, ReadStage::AfterOpen);

            let mut bytes = Vec::with_capacity(
                usize::try_from(identity.size)
                    .unwrap_or(0)
                    .min(usize::try_from(max_bytes).unwrap_or(usize::MAX)),
            );
            let mut reader = descriptor;
            let mut chunk = [0_u8; 8 * 1024];
            let mut first = true;
            loop {
                let remaining = usize::try_from(max_bytes)
                    .unwrap_or(usize::MAX)
                    .saturating_add(1)
                    .saturating_sub(bytes.len());
                if remaining == 0 {
                    return Err(format!(
                        "{} exceeded the {max_bytes}-byte replay bound while being read",
                        public_path.display()
                    ));
                }
                let chunk_limit = remaining.min(chunk.len());
                let read = reader
                    .read(&mut chunk[..chunk_limit])
                    .map_err(|error| format!("cannot read {}: {error}", public_path.display()))?;
                if read == 0 {
                    break;
                }
                bytes.extend_from_slice(&chunk[..read]);
                if first {
                    first = false;
                    hook(&public_path, ReadStage::AfterFirstRead);
                }
                if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
                    return Err(format!(
                        "{} exceeded the {max_bytes}-byte replay bound while being read",
                        public_path.display()
                    ));
                }
            }

            let after = fstat(&reader)
                .map_err(|error| format!("cannot rebind {}: {error}", public_path.display()))?;
            ensure_identity(&public_path, identity, Identity::from_stat(&after))?;
            let rebound = statat(&self.descriptor, name, AtFlags::SYMLINK_NOFOLLOW)
                .map_err(|error| format!("cannot re-inspect {}: {error}", public_path.display()))?;
            ensure_identity(&public_path, identity, Identity::from_stat(&rebound))?;
            self.revalidate()?;
            Ok(bytes)
        }

        pub(crate) fn revalidate(&self) -> Result<(), String> {
            let descriptor = fstat(&self.descriptor)
                .map_err(|error| format!("cannot rebind {}: {error}", self.path.display()))?;
            ensure_identity(&self.path, self.identity, Identity::from_stat(&descriptor))?;
            let public = statat(CWD, &self.path, AtFlags::SYMLINK_NOFOLLOW)
                .map_err(|error| format!("cannot re-inspect {}: {error}", self.path.display()))?;
            ensure_identity(&self.path, self.identity, Identity::from_stat(&public))
        }
    }

    fn ensure_identity(path: &Path, expected: Identity, actual: Identity) -> Result<(), String> {
        if actual != expected {
            return Err(format!("{} identity changed during replay", path.display()));
        }
        Ok(())
    }
}

#[cfg(not(unix))]
mod imp {
    use super::{Path, ReadStage};

    pub(crate) struct CorpusDir;

    impl CorpusDir {
        pub(crate) fn open(path: &Path) -> Result<Self, String> {
            Err(format!(
                "{} corpus custody is unavailable on this platform",
                path.display()
            ))
        }

        pub(crate) fn inventory(&self) -> Result<Vec<String>, String> {
            Err("corpus custody is unavailable on this platform".to_string())
        }

        pub(crate) fn read_seed(
            &self,
            _name: &str,
            _max_bytes: u64,
            _hook: impl FnMut(&Path, ReadStage),
        ) -> Result<Vec<u8>, String> {
            Err("corpus custody is unavailable on this platform".to_string())
        }

        pub(crate) fn revalidate(&self) -> Result<(), String> {
            Err("corpus custody is unavailable on this platform".to_string())
        }
    }
}

pub(super) use imp::CorpusDir;
