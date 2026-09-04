use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    os::unix::fs::MetadataExt,
    path::{Component, Path},
};

use fs2::FileExt;
use rustix::fs::{Mode, OFlags, ResolveFlags, fchmod, openat2};
use serde::Serialize;

use super::platform_fs as io;
use crate::coord::{
    CoordError,
    generation::{
        manifest::{CurrentPointer, GenerationManifest},
        segment::{AppendRequest, validate_append_request},
    },
    model::{GENERATION_SCHEMA_VERSION, Record, RecoveryBaselineBody},
};

const LOCK: &str = "LOCK";
const REQUEST_DOMAIN: &str = "bullet.coord.recovery-baseline-request.v2";

#[path = "authority/metadata.rs"]
pub(super) mod metadata;

pub(super) use metadata::{
    retain_tombstone, tombstone_identity, write_or_verify_intent,
    write_or_verify_tombstone_observation,
};

pub(super) struct Authority {
    root: File,
    lock: File,
    owner: u32,
}

#[derive(Debug)]
pub(super) struct BaselineSubject {
    pub(super) genesis_digest: String,
    pub(super) request_id: String,
    pub(super) request_digest: String,
}

#[derive(Serialize)]
struct BaselineRequestSubject<'a> {
    kind: &'static str,
    generation_id: &'a str,
    record: &'a Record,
}

impl Authority {
    #[cfg(test)]
    pub(super) fn acquire(coord_dir: &Path) -> Result<Self, CoordError> {
        Self::acquire_authorized(coord_dir, || Ok(()))
    }

    pub(super) fn acquire_authorized(
        coord_dir: &Path,
        mut revalidate_authority: impl FnMut() -> Result<(), CoordError>,
    ) -> Result<Self, CoordError> {
        let root = open_root(coord_dir)?;
        let owner = rustix::process::geteuid().as_raw();
        validate_root(&root, owner, true)?;
        revalidate_authority()?;
        let lock = open_or_create_lock(&root, owner)?;
        validate_lock(&lock, owner)?;
        lock.try_lock_exclusive().map_err(|error| {
            CoordError::new(
                "COORD_RECOVERY_LOCKED",
                format!("stable LOCK is busy: {error}"),
            )
        })?;
        let authority = Self { root, lock, owner };
        authority.revalidate(coord_dir, true)?;
        fchmod(&authority.root, Mode::RWXU)
            .map_err(|error| invalid(format!("cannot tighten coordination root: {error}")))?;
        authority.root.sync_all().map_err(CoordError::io)?;
        authority.revalidate(coord_dir, false)?;
        Ok(authority)
    }

    pub(super) const fn owner(&self) -> u32 {
        self.owner
    }

    pub(super) const fn root(&self) -> &File {
        &self.root
    }

    pub(super) fn revalidate_final(&self, coord_dir: &Path) -> Result<(), CoordError> {
        self.revalidate(coord_dir, false)
    }

    pub(super) fn publish_current(
        &self,
        coord_dir: &Path,
        generation_id: &str,
        bytes: &[u8],
    ) -> Result<(), CoordError> {
        self.revalidate(coord_dir, false)?;
        let name = format!(".CURRENT.next-{generation_id}");
        require_only_current_stage(&self.root, &name)?;
        let (mut staged, created) = open_or_create_current_stage(&self.root, &name, self.owner)?;
        let metadata = staged.metadata().map_err(CoordError::io)?;
        if metadata.mode() & 0o7777 == 0o600 {
            let observed = read_prefix(&mut staged, bytes.len())?;
            if !bytes.starts_with(&observed) {
                return Err(current_unknown("CURRENT stage is not an exact prefix"));
            }
            staged.seek(SeekFrom::End(0)).map_err(CoordError::io)?;
            staged
                .write_all(&bytes[observed.len()..])
                .map_err(CoordError::io)?;
        } else if metadata.mode() & 0o7777 == 0o400 {
            if read_prefix(&mut staged, bytes.len())? != bytes {
                return Err(current_unknown("sealed CURRENT stage differs"));
            }
        } else {
            return Err(current_unknown("CURRENT stage mode is not 0600 or 0400"));
        }
        staged.sync_all().map_err(CoordError::io)?;
        if created || metadata.mode() & 0o7777 == 0o600 {
            fchmod(&staged, Mode::RUSR)
                .map_err(|error| current_unknown(format!("cannot seal CURRENT stage: {error}")))?;
        }
        staged.sync_all().map_err(CoordError::io)?;
        self.root.sync_all().map_err(CoordError::io)?;
        let reopened = open_current_stage(&self.root, &name, self.owner, 0o400)?;
        if identity(&staged)? != identity(&reopened)? {
            return Err(current_unknown("CURRENT stage identity changed"));
        }
        io::publish_no_replace_at(&self.root, &name, "CURRENT")?;
        self.root.sync_all().map_err(CoordError::io)?;
        self.revalidate(coord_dir, false)
    }

    pub(super) fn tombstone_identity(&self) -> Result<(u64, u64), CoordError> {
        metadata::tombstone_identity(&self.root, self.owner)
    }

    pub(super) fn retain_tombstone(&self, expected: (u64, u64)) -> Result<File, CoordError> {
        metadata::retain_tombstone(&self.root, self.owner, expected)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn write_or_verify_tombstone_observation(
        &self,
        recovery_dir: &File,
        manifest: &GenerationManifest,
        baseline: &BaselineSubject,
        intent_sha256: &str,
        prepared_observation_sha256: &str,
        retained: &File,
        allow_create: bool,
    ) -> Result<String, CoordError> {
        metadata::write_or_verify_tombstone_observation(
            &self.root,
            self.owner,
            recovery_dir,
            manifest,
            baseline,
            intent_sha256,
            prepared_observation_sha256,
            retained,
            allow_create,
        )
    }

    fn revalidate(&self, coord_dir: &Path, allow_legacy_mode: bool) -> Result<(), CoordError> {
        validate_root(&self.root, self.owner, allow_legacy_mode)?;
        validate_lock(&self.lock, self.owner)?;
        let reopened_root = open_root(coord_dir)?;
        validate_root(&reopened_root, self.owner, allow_legacy_mode)?;
        if identity(&self.root)? != identity(&reopened_root)? {
            return Err(invalid("coordination root pathname identity changed"));
        }
        let reopened_lock = open_lock(&self.root)?;
        validate_lock(&reopened_lock, self.owner)?;
        if identity(&self.lock)? != identity(&reopened_lock)? {
            return Err(invalid("stable LOCK pathname identity changed"));
        }
        Ok(())
    }
}

pub(super) fn baseline_subject(
    manifest: &GenerationManifest,
    record: &Record,
) -> Result<BaselineSubject, CoordError> {
    let pointer = CurrentPointer::for_manifest(manifest)?;
    let genesis_digest = pointer
        .manifest_blake3()
        .strip_prefix("blake3:")
        .ok_or_else(|| invalid("manifest BLAKE3 is not algorithm tagged"))?
        .to_owned();
    let bytes = bullet_wire::canonical_json(&BaselineRequestSubject {
        kind: "coord_recovery_baseline_request_v2",
        generation_id: manifest.generation_id().as_str(),
        record,
    })
    .map_err(|error| invalid(format!("cannot encode baseline request subject: {error}")))?;
    let request_id = format!(
        "recovery_{}",
        bullet_wire::hash_framed_bytes(REQUEST_DOMAIN, &bytes)
            .map_err(|error| invalid(format!("cannot hash baseline request subject: {error}")))?
            .to_hex()
    );
    let request_digest = validate_append_request(
        &AppendRequest {
            generation_id: manifest.generation_id().as_str(),
            sequence: 1,
            previous_digest: &genesis_digest,
            request_id: &request_id,
            record,
        },
        &genesis_digest,
    )?;
    Ok(BaselineSubject {
        genesis_digest,
        request_id,
        request_digest,
    })
}

pub(super) fn baseline_record(manifest: &GenerationManifest) -> Result<Record, CoordError> {
    let recovery = manifest.body.recovery()?;
    let pointer = CurrentPointer::for_manifest(manifest)?;
    Ok(Record::RecoveryBaselineV2 {
        schema_version: GENERATION_SCHEMA_VERSION,
        generation_id: manifest.generation_id().as_str().to_owned(),
        body: RecoveryBaselineBody {
            manifest_blake3: pointer.manifest_blake3().to_owned(),
            incident_at_unix_ms: recovery.incident_at_unix_ms,
            recovered_at_unix_ms: recovery.recovered_at_unix_ms,
            trusted_state_blake3: recovery.trusted_state_blake3.clone(),
            frozen_claims: recovery.frozen_claims.clone(),
        },
    })
}

fn open_or_create_current_stage(
    root: &File,
    name: &str,
    owner: u32,
) -> Result<(File, bool), CoordError> {
    let flags = OFlags::RDWR | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    match openat2(root, name, flags, Mode::RUSR | Mode::WUSR, resolve()) {
        Ok(descriptor) => Ok((File::from(descriptor), true)),
        Err(rustix::io::Errno::EXIST) => {
            let mode = open_current_stage_mode(root, name, owner)?;
            Ok((open_current_stage(root, name, owner, mode)?, false))
        }
        Err(error) => Err(current_unknown(format!(
            "cannot create CURRENT stage: {error}"
        ))),
    }
}

fn open_current_stage_mode(root: &File, name: &str, owner: u32) -> Result<u32, CoordError> {
    let file = openat2(
        root,
        name,
        OFlags::PATH | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        resolve(),
    )
    .map(File::from)
    .map_err(|error| current_unknown(format!("cannot inspect CURRENT stage: {error}")))?;
    let metadata = file.metadata().map_err(CoordError::io)?;
    if !metadata.is_file() || metadata.uid() != owner || metadata.nlink() != 1 {
        return Err(current_unknown(
            "CURRENT stage is not an exact regular file",
        ));
    }
    Ok(metadata.mode() & 0o7777)
}

fn open_current_stage(root: &File, name: &str, owner: u32, mode: u32) -> Result<File, CoordError> {
    let access = if mode == 0o600 {
        OFlags::RDWR
    } else {
        OFlags::RDONLY
    };
    let file = openat2(
        root,
        name,
        access | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        resolve(),
    )
    .map(File::from)
    .map_err(|error| current_unknown(format!("cannot retain CURRENT stage: {error}")))?;
    let metadata = file.metadata().map_err(CoordError::io)?;
    if !metadata.is_file()
        || metadata.uid() != owner
        || metadata.nlink() != 1
        || metadata.mode() & 0o7777 != mode
    {
        return Err(current_unknown("CURRENT stage admission differs"));
    }
    Ok(file)
}

fn read_prefix(file: &mut File, maximum: usize) -> Result<Vec<u8>, CoordError> {
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let mut bytes = Vec::new();
    Read::by_ref(file)
        .take(maximum as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    if bytes.len() > maximum || file.metadata().map_err(CoordError::io)?.len() != bytes.len() as u64
    {
        return Err(current_unknown("CURRENT stage exceeds the exact pointer"));
    }
    Ok(bytes)
}

fn require_only_current_stage(root: &File, expected: &str) -> Result<(), CoordError> {
    let mut entries = rustix::fs::Dir::read_from(root)
        .map_err(|error| current_unknown(format!("cannot inventory CURRENT stages: {error}")))?;
    while let Some(entry) = entries.read() {
        let entry = entry.map_err(|error| current_unknown(format!("cannot read root: {error}")))?;
        let name = entry.file_name().to_string_lossy();
        if name.contains("CURRENT.next") && name != expected {
            return Err(current_unknown("an unbound CURRENT stage exists"));
        }
    }
    Ok(())
}

fn resolve() -> ResolveFlags {
    ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS
}

fn current_unknown(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_CURRENT_OUTCOME_UNKNOWN", reason)
}

fn open_root(path: &Path) -> Result<File, CoordError> {
    if !path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::RootDir | Component::Normal(_)))
    {
        return Err(invalid(
            "coordination root path is not absolute and normalized",
        ));
    }
    openat2(
        rustix::fs::CWD,
        path,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map(File::from)
    .map_err(|error| invalid(format!("cannot open coordination root safely: {error}")))
}

fn open_lock(root: &File) -> Result<File, CoordError> {
    openat2(
        root,
        LOCK,
        OFlags::RDWR | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map(File::from)
    .map_err(|error| invalid(format!("cannot open stable LOCK safely: {error}")))
}

fn open_or_create_lock(root: &File, owner: u32) -> Result<File, CoordError> {
    let flags = OFlags::RDWR
        | OFlags::CREATE
        | OFlags::EXCL
        | OFlags::NOFOLLOW
        | OFlags::NONBLOCK
        | OFlags::CLOEXEC;
    let resolve = ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS;
    let lock_mode = Mode::RUSR | Mode::WUSR;
    let candidate = match openat2(root, LOCK, flags, lock_mode, resolve) {
        Ok(descriptor) => {
            let created = File::from(descriptor);
            fchmod(&created, lock_mode)
                .map_err(|error| invalid(format!("cannot seal stable LOCK mode: {error}")))?;
            created
        }
        Err(rustix::io::Errno::EXIST) => open_lock(root)?,
        Err(error) => {
            return Err(invalid(format!(
                "cannot create stable LOCK exclusively: {error}"
            )));
        }
    };
    validate_lock(&candidate, owner)?;
    candidate.sync_all().map_err(CoordError::io)?;
    root.sync_all().map_err(CoordError::io)?;
    let reopened = open_lock(root)?;
    validate_lock(&reopened, owner)?;
    if identity(&candidate)? != identity(&reopened)? {
        return Err(invalid("stable LOCK pathname identity changed"));
    }
    Ok(candidate)
}

fn validate_root(file: &File, owner: u32, allow_legacy_mode: bool) -> Result<(), CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    let mode = metadata.mode() & 0o7777;
    if !metadata.is_dir()
        || metadata.uid() != owner
        || if allow_legacy_mode {
            mode != 0o700 && mode != 0o775
        } else {
            mode != 0o700
        }
    {
        return Err(invalid(
            "coordination root must be current-owner mode 0700 or admitted legacy 0775",
        ));
    }
    Ok(())
}

fn validate_lock(file: &File, owner: u32) -> Result<(), CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    if !metadata.is_file()
        || metadata.uid() != owner
        || metadata.nlink() != 1
        || metadata.len() != 0
        || metadata.mode() & 0o7777 != 0o600
    {
        return Err(invalid(
            "stable LOCK must be current-owner mode-0600 regular and single-link",
        ));
    }
    Ok(())
}

fn identity(file: &File) -> Result<(u64, u64), CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    Ok((metadata.dev(), metadata.ino()))
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_RECOVERY", reason)
}
