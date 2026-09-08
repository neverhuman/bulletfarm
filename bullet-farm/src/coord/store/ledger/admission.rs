//! Refuse incident-derived initialization through the pristine bootstrap API.
//!
//! Absence is a local observation, not proof that history never existed. A
//! complete incident admission must eventually bind both locations into Genesis;
//! this guard grants no retirement, recovery, or fresh-generation authority.

#[cfg(target_os = "linux")]
pub(super) use linux::{require_matching_journal, require_pristine_context};

#[cfg(not(target_os = "linux"))]
pub(super) fn require_pristine_context(
    _: &std::path::Path,
) -> Result<(), crate::coord::CoordError> {
    Err(crate::coord::CoordError::new(
        "COORD_PLATFORM_UNSUPPORTED",
        "generation ledger requires Linux",
    ))
}

#[cfg(not(target_os = "linux"))]
pub(super) fn require_matching_journal(
    root: &std::path::Path,
    _: &super::genesis::PreparedGenesis,
) -> Result<(), crate::coord::CoordError> {
    require_pristine_context(root)
}

#[cfg(target_os = "linux")]
mod linux {
    use std::{collections::BTreeSet, fs::File, os::unix::fs::MetadataExt, path::Path};

    use rustix::fs::{Dir, Mode, OFlags, ResolveFlags, openat2};

    use crate::coord::{CoordError, recovery_manifest::require_normalized_absolute};

    const MAX_ENTRIES: usize = 64;
    const JOURNAL_FILES: &[&[u8]] = &[
        b"LOCK",
        b"CURRENT",
        b"events.jsonl",
        b"generations",
        b"genesis-init-intent.json",
        b"genesis-fence-create-plan.json",
        b"genesis-fence-create-observation.json",
        b"genesis-fence-publish-intent.json",
        b"genesis-fence-seal-observation.json",
        b"genesis-fence-publication-plan.json",
        b"genesis-fence-observation.json",
    ];

    pub(in crate::coord::store::ledger) fn require_pristine_context(
        root: &Path,
    ) -> Result<(), CoordError> {
        require_normalized_absolute(root, "pristine Genesis family root")?;
        let outer = root.join(".bullet-family");
        if inventory(&outer)?.is_some_and(|names| names.iter().any(|name| name != b"coord")) {
            return Err(required(
                "outer family metadata retains non-pristine material",
            ));
        }
        let hub = root.join("bullet-farm/.bullet-family");
        if inventory(&hub)?.is_some_and(|names| !names.is_empty()) {
            return Err(required(
                "Hub-local metadata retains incident or coordinator material",
            ));
        }
        let Some(names) = inventory(&outer.join("coord"))? else {
            return Ok(());
        };
        // An intact legacy source keeps the existing explicit recovery refusal.
        // An interrupted/new bootstrap without an intent may contain only LOCK.
        if names.contains(b"genesis-init-intent.json".as_slice())
            && names
                .iter()
                .any(|name| name.starts_with(b".genesis-init-intent.json.next-"))
        {
            return Err(required(
                "published initialization intent retains an unconsumed stage",
            ));
        }
        let current = names.contains(b"CURRENT".as_slice());
        let journal = current
            || names.contains(b"genesis-init-intent.json".as_slice())
            || names
                .iter()
                .any(|name| digest_suffix(name, b".genesis-init-intent.json.next-"));
        for name in &names {
            let admitted = if journal {
                JOURNAL_FILES.contains(&name.as_slice()) || (!current && journal_stage(name))
            } else {
                matches!(name.as_slice(), b"LOCK" | b"events.jsonl")
            };
            if !admitted {
                return Err(required(
                    "coordinator retains bytes outside the pristine initialization journal",
                ));
            }
        }
        Ok(())
    }

    pub(in crate::coord::store::ledger) fn require_matching_journal(
        root: &Path,
        prepared: &crate::coord::store::ledger::genesis::PreparedGenesis,
    ) -> Result<(), CoordError> {
        require_pristine_context(root)?;
        let coord = root.join(".bullet-family/coord");
        let names = inventory(&coord)?.unwrap_or_default();
        let current = prepared.current.canonical_bytes()?;
        let expected = [
            stage_name("genesis-init-intent.json", &prepared.intent_bytes)?,
            stage_name("CURRENT", &current)?,
            format!(
                ".events.jsonl.genesis-next-{}",
                prepared.manifest.generation_id().as_str()
            )
            .into_bytes(),
        ];
        if names
            .iter()
            .any(|name| name.starts_with(b".") && !expected.contains(name))
        {
            return Err(required(
                "initialization stage is not bound to the exact durable intent",
            ));
        }
        let generations = coord.join("generations");
        let generation = prepared.manifest.generation_id().as_str();
        let staged = format!(".next-{generation}");
        let children = inventory(&generations)?.unwrap_or_default();
        if children.len() > 1
            || children
                .iter()
                .any(|name| name != generation.as_bytes() && name != staged.as_bytes())
        {
            return Err(required(
                "generation inventory differs from the exact initialization subject",
            ));
        }
        for name in children {
            let child = if name == generation.as_bytes() {
                generation
            } else {
                &staged
            };
            let files = inventory(&generations.join(child))?.unwrap_or_default();
            let manifest_stage =
                stage_name("manifest.json", &prepared.manifest.canonical_bytes()?)?;
            if files.contains(b"manifest.json".as_slice())
                && files
                    .iter()
                    .any(|name| name.starts_with(b".manifest.json.next-"))
            {
                return Err(required("published manifest retains an unconsumed stage"));
            }

            if files.iter().any(|name| {
                !matches!(
                    name.as_slice(),
                    b"manifest.json" | b"events.jsonl" | b"pending"
                ) && (names.contains(b"CURRENT".as_slice()) || *name != manifest_stage)
            }) {
                return Err(required(
                    "generation contains unreferenced initialization material",
                ));
            }
        }
        Ok(())
    }

    fn stage_name(name: &str, bytes: &[u8]) -> Result<Vec<u8>, CoordError> {
        let digest = bullet_wire::hash_framed_bytes("bullet.coord.staged-file.v2", bytes)
            .map_err(storage)?;
        Ok(format!(".{name}.next-{}", digest.to_hex()).into_bytes())
    }

    fn journal_stage(name: &[u8]) -> bool {
        digest_suffix(name, b".events.jsonl.genesis-next-gen_")
            || JOURNAL_FILES
                .iter()
                .skip(1)
                .filter(|base| !matches!(**base, b"events.jsonl" | b"generations"))
                .any(|base| {
                    let mut prefix = Vec::with_capacity(base.len() + 7);
                    prefix.push(b'.');
                    prefix.extend_from_slice(base);
                    prefix.extend_from_slice(b".next-");
                    digest_suffix(name, &prefix)
                })
    }

    fn digest_suffix(name: &[u8], prefix: &[u8]) -> bool {
        name.strip_prefix(prefix).is_some_and(|suffix| {
            suffix.len() == 64
                && suffix
                    .iter()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
        })
    }

    fn inventory(path: &Path) -> Result<Option<BTreeSet<Vec<u8>>>, CoordError> {
        let Some(directory) = open_directory(path)? else {
            return Ok(None);
        };
        let before = identity(&directory)?;
        let mut entries = Dir::read_from(&directory).map_err(storage)?;
        let mut names = BTreeSet::new();
        while let Some(entry) = entries.read() {
            let entry = entry.map_err(storage)?;
            let name = entry.file_name().to_bytes();
            if matches!(name, b"." | b"..") {
                continue;
            }
            if names.len() == MAX_ENTRIES || !names.insert(name.to_vec()) {
                return Err(required(
                    "metadata inventory exceeds its bound or repeats a name",
                ));
            }
        }
        let reopened = open_directory(path)?
            .ok_or_else(|| required("metadata disappeared during pristine inspection"))?;
        if before != identity(&directory)? || before != identity(&reopened)? {
            return Err(required("metadata changed during pristine inspection"));
        }
        Ok(Some(names))
    }

    fn open_directory(path: &Path) -> Result<Option<File>, CoordError> {
        match openat2(
            rustix::fs::CWD,
            path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
        ) {
            Ok(fd) => Ok(Some(File::from(fd))),
            Err(rustix::io::Errno::NOENT) => Ok(None),
            Err(error) => Err(storage(error)),
        }
    }

    fn identity(file: &File) -> Result<(u64, u64, i64, i64), CoordError> {
        let metadata = file.metadata().map_err(storage)?;
        Ok((
            metadata.dev(),
            metadata.ino(),
            metadata.ctime(),
            metadata.ctime_nsec(),
        ))
    }

    fn storage(error: impl std::fmt::Display) -> CoordError {
        required(format!(
            "cannot inspect pristine initialization context: {error}"
        ))
    }

    fn required(reason: impl Into<String>) -> CoordError {
        CoordError::new("FRESH_GENESIS_ADMISSION_REQUIRED", reason)
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests;
