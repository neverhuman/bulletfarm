#[cfg(test)]
use std::cell::Cell;
use std::{
    fs::File,
    io::{Read, Seek, Write},
    os::unix::fs::MetadataExt,
};

use rustix::fs::{Mode, OFlags, ResolveFlags, fchmod, openat, openat2};
use serde::Serialize;

use super::BaselineSubject;
use crate::coord::{
    CoordError,
    anonymous_link::{self, LinkOutcome},
    generation::{
        manifest::{CurrentPointer, GenerationManifest, Sha256Digest},
        recovery::ContentExpectation,
    },
};

const OBSERVATION: &str = "tombstone-seal-observation.json";
const MAX_EVIDENCE_BYTES: usize = 16 * 1024;

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::coord::generation::recovery) enum EvidenceCrash {
    WriteOffset(usize),
    AfterDataSync,
    AfterSeal,
    AfterLink,
}

#[cfg(test)]
thread_local! {
    static EVIDENCE_CRASH: Cell<Option<EvidenceCrash>> = const { Cell::new(None) };
}

#[cfg(test)]
pub(in crate::coord::generation::recovery) fn test_crash_at(point: EvidenceCrash) {
    EVIDENCE_CRASH.with(|selected| selected.set(Some(point)));
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct RecoveryIntent {
    schema_version: u32,
    generation_id: String,
    manifest_sha256: String,
    source_device: u64,
    source_inode: u64,
    tombstone_device: u64,
    tombstone_inode: u64,
    frozen_live_length: u64,
    frozen_live_sha256: String,
    genesis_digest: String,
    request_id: String,
    request_digest: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
struct TombstoneFacts {
    device: u64,
    inode: u64,
    owner: u32,
    mode: u32,
    link_count: u64,
    byte_size: u64,
    ctime_seconds: i64,
    ctime_nanoseconds: i64,
}

#[derive(Debug, Serialize)]
struct TombstoneSealObservationV2<'a> {
    schema_version: u32,
    kind: &'static str,
    generation_id: &'a str,
    manifest_blake3: &'a str,
    intent_sha256: &'a str,
    prepared_observation_sha256: &'a str,
    request_digest: &'a str,
    object_type: &'static str,
    tombstone: TombstoneFacts,
}

pub(in crate::coord::generation::recovery) fn tombstone_identity(
    root: &File,
    owner: u32,
) -> Result<(u64, u64), CoordError> {
    let (_, facts) = open_tombstone(root, owner, Some(0o400))?;
    Ok((facts.device, facts.inode))
}

pub(in crate::coord::generation::recovery) fn retain_tombstone(
    root: &File,
    owner: u32,
    expected: (u64, u64),
) -> Result<File, CoordError> {
    let (retained, facts) = open_tombstone(root, owner, Some(0o400))?;
    if (facts.device, facts.inode) != expected {
        return Err(unknown("exchanged tombstone identity differs from intent"));
    }
    Ok(retained)
}

#[allow(clippy::too_many_arguments)]
pub(in crate::coord::generation::recovery) fn write_or_verify_tombstone_observation(
    root: &File,
    owner: u32,
    recovery_dir: &File,
    manifest: &GenerationManifest,
    baseline: &BaselineSubject,
    intent_sha256: &str,
    prepared_observation_sha256: &str,
    retained: &File,
    allow_create: bool,
) -> Result<String, CoordError> {
    let (observed, facts) = open_tombstone(root, owner, Some(0o400))?;
    if identity(retained)? != (facts.device, facts.inode) {
        return Err(unknown(
            "exchanged tombstone identity changed before observation",
        ));
    }
    let pointer = CurrentPointer::for_manifest(manifest)?;
    let bytes = canonical_line(&TombstoneSealObservationV2 {
        schema_version: 2,
        kind: "coord_tombstone_seal_observation_v2",
        generation_id: manifest.generation_id().as_str(),
        manifest_blake3: pointer.manifest_blake3(),
        intent_sha256,
        prepared_observation_sha256,
        request_digest: &baseline.request_digest,
        object_type: "directory",
        tombstone: facts,
    })?;
    write_or_verify_sealed(
        recovery_dir,
        OBSERVATION,
        &bytes,
        allow_create,
        "TOMBSTONE_SEAL_OUTCOME_UNKNOWN",
    )?;
    let (_, after) = open_tombstone(root, owner, Some(0o400))?;
    if after != facts || identity(&observed)? != (after.device, after.inode) {
        return Err(unknown("tombstone metadata changed during observation"));
    }
    Ok(Sha256Digest::for_bytes(&bytes).as_str().to_owned())
}

pub(in crate::coord::generation::recovery) fn write_or_verify_intent(
    parent: &File,
    manifest: &GenerationManifest,
    frozen_live_source: &ContentExpectation,
    source: (u64, u64),
    tombstone: (u64, u64),
    baseline: &BaselineSubject,
    allow_create: bool,
) -> Result<String, CoordError> {
    let expected = canonical_line(&RecoveryIntent {
        schema_version: 2,
        generation_id: manifest.generation_id().as_str().to_owned(),
        manifest_sha256: Sha256Digest::for_bytes(&manifest.canonical_bytes()?)
            .as_str()
            .to_owned(),
        source_device: source.0,
        source_inode: source.1,
        tombstone_device: tombstone.0,
        tombstone_inode: tombstone.1,
        frozen_live_length: frozen_live_source.byte_length,
        frozen_live_sha256: frozen_live_source.sha256.as_str().to_owned(),
        genesis_digest: baseline.genesis_digest.clone(),
        request_id: baseline.request_id.clone(),
        request_digest: baseline.request_digest.clone(),
    })?;
    write_or_verify_sealed(
        parent,
        "intent.json",
        &expected,
        allow_create,
        if allow_create {
            "INVALID_COORD_RECOVERY"
        } else {
            "COORD_RECOVERY_INTENT_OUTCOME_UNKNOWN"
        },
    )?;
    Ok(Sha256Digest::for_bytes(&expected).as_str().to_owned())
}

fn open_tombstone(
    root: &File,
    owner: u32,
    required_mode: Option<u32>,
) -> Result<(File, TombstoneFacts), CoordError> {
    let descriptor = openat2(
        root,
        "events.jsonl",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map_err(|error| unknown(format!("cannot retain sealed tombstone: {error}")))?;
    let file = File::from(descriptor);
    let facts = TombstoneFacts::from_file(&file, owner, required_mode)?;
    super::super::platform_fs::require_empty_descriptor(&file)?;
    Ok((file, facts))
}

pub(in crate::coord::generation::recovery) fn write_or_verify_sealed(
    parent: &File,
    name: &str,
    expected: &[u8],
    allow_create: bool,
    code: &'static str,
) -> Result<(), CoordError> {
    if expected.is_empty() || expected.len() > MAX_EVIDENCE_BYTES {
        return Err(CoordError::new(
            code,
            format!("durable {name} exceeds its closed evidence bound"),
        ));
    }
    if let Some(observed) = read_sealed(parent, name, code)? {
        if observed != expected {
            return Err(CoordError::new(code, format!("durable {name} differs")));
        }
        parent.sync_all().map_err(CoordError::io)?;
        return match read_sealed(parent, name, code)? {
            Some(read_back) if read_back == expected => Ok(()),
            _ => Err(CoordError::new(
                code,
                format!("durable {name} changed during reconciliation"),
            )),
        };
    }
    if !allow_create {
        return Err(CoordError::new(
            code,
            format!("durable {name} is missing; manual reconciliation is required"),
        ));
    }
    let descriptor = openat(
        parent,
        ".",
        OFlags::TMPFILE | OFlags::RDWR | OFlags::CLOEXEC,
        Mode::RUSR | Mode::WUSR,
    )
    .map_err(|error| CoordError::new(code, format!("cannot create anonymous {name}: {error}")))?;
    let mut file = File::from(descriptor);
    write_evidence(&mut file, expected, code)?;
    file.sync_all().map_err(CoordError::io)?;
    #[cfg(test)]
    injected(EvidenceCrash::AfterDataSync, code)?;
    fchmod(&file, Mode::RUSR)
        .map_err(|error| CoordError::new(code, format!("cannot seal {name}: {error}")))?;
    file.sync_all().map_err(CoordError::io)?;
    #[cfg(test)]
    injected(EvidenceCrash::AfterSeal, code)?;
    let metadata = file.metadata().map_err(CoordError::io)?;
    let identity = (metadata.dev(), metadata.ino());
    if !metadata.is_file()
        || metadata.uid() != rustix::process::geteuid().as_raw()
        || metadata.mode() & 0o7777 != 0o400
        || metadata.nlink() != 0
        || metadata.len() != expected.len() as u64
    {
        return Err(CoordError::new(
            code,
            format!("anonymous {name} seal identity is invalid"),
        ));
    }
    let link_outcome = anonymous_link::link(&file, parent, name, identity)
        .map_err(|error| CoordError::new(code, error))?;
    #[cfg(test)]
    if link_outcome == LinkOutcome::Linked {
        injected(EvidenceCrash::AfterLink, code)?;
    }
    parent.sync_all().map_err(CoordError::io)?;
    if link_outcome == LinkOutcome::Linked && named_identity(parent, name, code)? != identity {
        return Err(CoordError::new(
            code,
            format!("linked {name} identity differs from the sealed anonymous inode"),
        ));
    }
    match read_sealed(parent, name, code)? {
        Some(observed) if observed == expected => Ok(()),
        _ => Err(CoordError::new(
            code,
            format!("durable {name} read-back differs"),
        )),
    }
}

fn named_identity(parent: &File, name: &str, code: &'static str) -> Result<(u64, u64), CoordError> {
    let descriptor = openat2(
        parent,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map_err(|error| CoordError::new(code, format!("cannot reopen linked {name}: {error}")))?;
    identity(&File::from(descriptor))
}

fn write_evidence(file: &mut File, expected: &[u8], _code: &'static str) -> Result<(), CoordError> {
    #[cfg(test)]
    if let Some(EvidenceCrash::WriteOffset(offset)) = EVIDENCE_CRASH.with(|selected| selected.get())
    {
        EVIDENCE_CRASH.with(|selected| selected.set(None));
        let offset = offset.min(expected.len());
        file.write_all(&expected[..offset])
            .map_err(CoordError::io)?;
        return Err(CoordError::new(
            _code,
            format!("injected evidence write interruption at byte {offset}"),
        ));
    }
    file.write_all(expected).map_err(CoordError::io)
}

#[cfg(test)]
fn injected(point: EvidenceCrash, code: &'static str) -> Result<(), CoordError> {
    if EVIDENCE_CRASH.with(|selected| selected.get()) == Some(point) {
        EVIDENCE_CRASH.with(|selected| selected.set(None));
        Err(CoordError::new(
            code,
            format!("injected evidence interruption after {point:?}"),
        ))
    } else {
        Ok(())
    }
}

pub(in crate::coord::generation::recovery) fn read_sealed(
    parent: &File,
    name: &str,
    code: &'static str,
) -> Result<Option<Vec<u8>>, CoordError> {
    let resolve = ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS;
    let descriptor = match openat2(
        parent,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        resolve,
    ) {
        Ok(descriptor) => descriptor,
        Err(rustix::io::Errno::NOENT) => return Ok(None),
        Err(error) => {
            return Err(CoordError::new(
                code,
                format!("cannot admit {name}: {error}"),
            ));
        }
    };
    let mut file = File::from(descriptor);
    let (bytes, before, after) = stable_read(&mut file)?;
    let (second, second_before, second_after) = stable_read(&mut file)?;
    let reopened = openat2(
        parent,
        name,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        resolve,
    )
    .map(File::from)
    .map_err(|error| CoordError::new(code, format!("cannot revalidate {name}: {error}")))?;
    if !before.is_file()
        || before.uid() != rustix::process::geteuid().as_raw()
        || before.nlink() != 1
        || before.mode() & 0o7777 != 0o400
        || metadata_signature(&before) != metadata_signature(&after)
        || metadata_signature(&before) != metadata_signature(&second_before)
        || metadata_signature(&before) != metadata_signature(&second_after)
        || bytes != second
        || before.len() != bytes.len() as u64
        || identity(&file)? != identity(&reopened)?
        || identity(&file)? != (after.dev(), after.ino())
    {
        return Err(CoordError::new(
            code,
            format!("durable {name} admission changed"),
        ));
    }
    Ok(Some(bytes))
}

fn stable_read(
    file: &mut File,
) -> Result<(Vec<u8>, std::fs::Metadata, std::fs::Metadata), CoordError> {
    file.rewind().map_err(CoordError::io)?;
    let before = file.metadata().map_err(CoordError::io)?;
    let mut bytes = Vec::new();
    Read::by_ref(file)
        .take(16 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    let after = file.metadata().map_err(CoordError::io)?;
    Ok((bytes, before, after))
}

fn metadata_signature(
    metadata: &std::fs::Metadata,
) -> (u64, u64, u32, u32, u64, u64, i64, i64, i64, i64) {
    (
        metadata.dev(),
        metadata.ino(),
        metadata.uid(),
        metadata.mode() & 0o7777,
        metadata.nlink(),
        metadata.len(),
        metadata.mtime(),
        metadata.mtime_nsec(),
        metadata.ctime(),
        metadata.ctime_nsec(),
    )
}

fn canonical_line(value: &impl Serialize) -> Result<Vec<u8>, CoordError> {
    let mut bytes = bullet_wire::canonical_json(value)
        .map_err(|error| invalid(format!("cannot encode recovery metadata: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

impl TombstoneFacts {
    fn from_file(file: &File, owner: u32, required_mode: Option<u32>) -> Result<Self, CoordError> {
        let metadata = file.metadata().map_err(CoordError::io)?;
        let mode = metadata.mode() & 0o7777;
        if !metadata.is_dir()
            || metadata.uid() != owner
            || required_mode.is_some_and(|required| mode != required)
            || metadata.nlink() != 2
        {
            return Err(unknown(
                "tombstone must be a current-owner, mode-0400, empty directory",
            ));
        }
        Ok(Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            owner: metadata.uid(),
            mode,
            link_count: metadata.nlink(),
            byte_size: metadata.len(),
            ctime_seconds: metadata.ctime(),
            ctime_nanoseconds: metadata.ctime_nsec(),
        })
    }
}

fn identity(file: &File) -> Result<(u64, u64), CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    Ok((metadata.dev(), metadata.ino()))
}

fn unknown(reason: impl Into<String>) -> CoordError {
    CoordError::new("TOMBSTONE_SEAL_OUTCOME_UNKNOWN", reason)
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_RECOVERY", reason)
}

#[cfg(test)]
#[path = "metadata/tests.rs"]
mod tests;
