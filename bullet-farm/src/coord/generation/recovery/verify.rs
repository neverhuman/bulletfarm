use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    os::unix::fs::MetadataExt,
};

use nix::sys::memfd::{MemFdCreateFlag, memfd_create};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::{
    ContentExpectation, RecoveryInput, authority, exchange, platform_fs as io, projection,
};
use crate::coord::{
    CoordError,
    generation::manifest::{ArtifactBinding, GenerationManifest},
    model::Record,
};

const INVENTORY_DOMAIN: &str = "bullet.coord.post-prefix-inventory.v2";

#[path = "verify/process.rs"]
mod process;
pub(super) use process::has_other_writable_fd;

#[path = "verify/lease.rs"]
mod lease;
pub(super) use lease::LegacyReadLease;

#[derive(Serialize)]
struct RangeObservation {
    byte_length: u64,
    sha256: String,
    lf_count: u64,
    ends_with_lf: bool,
}

pub(super) struct Preflight {
    pub(super) interrupted: File,
    pub(super) tainted: File,
    pub(super) legacy: File,
    pub(super) location: exchange::LegacyLocation,
    pub(super) sibling_state: exchange::SiblingState,
    pub(super) source_identity: (u64, u64),
}

pub(super) fn identity(file: &File) -> Result<(u64, u64), CoordError> {
    let metadata = file.metadata().map_err(CoordError::io)?;
    Ok((metadata.dev(), metadata.ino()))
}

pub(super) fn creation_free_preflight(
    input: &RecoveryInput,
    manifest: &GenerationManifest,
) -> Result<Preflight, CoordError> {
    let id = manifest.generation_id().as_str();
    let legacy_path = input.coord_dir.join("events.jsonl");
    let retired = input
        .coord_dir
        .join("recovery")
        .join(id)
        .join("retired-v1.non-authoritative");
    let sibling = exchange::sibling_path(&input.coord_dir, id)?;
    if input.frozen_live_source.path != legacy_path {
        return Err(invalid("frozen source is not the exact legacy pathname"));
    }
    let owner = rustix::process::geteuid().as_raw();
    let interrupted = io::open_source(&input.interrupted_capture, owner)?;
    let tainted = io::open_source(&input.tainted_generation, owner)?;
    let (legacy, location, sibling_state) =
        exchange::open_legacy(&legacy_path, &retired, &sibling, owner)?;
    let source_identity = identity(&legacy)?;
    let mut preflight = Preflight {
        interrupted,
        tainted,
        legacy,
        location,
        sibling_state,
        source_identity,
    };
    revalidate_preflight(&mut preflight, input, manifest)?;
    Ok(preflight)
}

pub(super) fn revalidate_preflight(
    preflight: &mut Preflight,
    input: &RecoveryInput,
    manifest: &GenerationManifest,
) -> Result<(), CoordError> {
    manifest.validate()?;
    verify_input_bindings(input, manifest)?;
    let recovery = manifest.body.recovery()?;
    let owner = rustix::process::geteuid().as_raw();
    if (recovery.legacy_source_device, recovery.legacy_source_inode) != preflight.source_identity {
        return Err(invalid(
            "manifest legacy device/inode differs from retained source",
        ));
    }
    let mut trusted = snapshot_prefix(
        &mut preflight.interrupted,
        &recovery.artifacts.trusted_prefix,
    )?;
    verify_retained_artifacts(
        &mut trusted,
        &mut preflight.interrupted,
        &mut preflight.tainted,
        &mut preflight.legacy,
        manifest,
    )?;
    let legacy_path = input.coord_dir.join("events.jsonl");
    let retired = input
        .coord_dir
        .join("recovery")
        .join(manifest.generation_id().as_str())
        .join("retired-v1.non-authoritative");
    let sibling = exchange::sibling_path(&input.coord_dir, manifest.generation_id().as_str())?;
    exchange::revalidate_topology(
        &legacy_path,
        &retired,
        &sibling,
        owner,
        (
            preflight.location,
            preflight.sibling_state,
            preflight.source_identity,
        ),
    )?;
    io::revalidate_path(
        &preflight.interrupted,
        &input.interrupted_capture.path,
        owner,
        0o400,
        false,
    )?;
    io::revalidate_path(
        &preflight.tainted,
        &input.tainted_generation.path,
        owner,
        0o400,
        false,
    )?;
    io::revalidate_path(
        &preflight.legacy,
        match preflight.location {
            exchange::LegacyLocation::Fresh => &legacy_path,
            exchange::LegacyLocation::TransientSibling => &sibling,
            exchange::LegacyLocation::Retired => &retired,
        },
        owner,
        0o400,
        false,
    )?;
    Ok(())
}

fn verify_input_bindings(
    input: &RecoveryInput,
    manifest: &GenerationManifest,
) -> Result<(), CoordError> {
    let artifacts = &manifest.body.recovery()?.artifacts;
    for (binding, expected) in [
        (&artifacts.trusted_prefix, &input.trusted_prefix),
        (
            &artifacts.interrupted_capture,
            &input.interrupted_capture.content,
        ),
        (
            &artifacts.tainted_generation,
            &input.tainted_generation.content,
        ),
        (
            &artifacts.frozen_live_source,
            &input.frozen_live_source.content,
        ),
    ] {
        if binding.byte_length != expected.byte_length || binding.sha256 != expected.sha256 {
            return Err(invalid(
                "manifest artifact differs from exact recovery input",
            ));
        }
    }
    Ok(())
}

fn verify_bound_file(
    file: &mut File,
    expected: &ContentExpectation,
    binding: &ArtifactBinding,
) -> Result<(), CoordError> {
    io::verify_open_file(file, expected)?;
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let mut reader = Read::by_ref(file).take(binding.byte_length.saturating_add(1));
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut bytes = 0_u64;
    let mut records = 0_u64;
    let mut last = None;
    loop {
        let read = reader.read(&mut buffer).map_err(CoordError::io)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        bytes = bytes
            .checked_add(read as u64)
            .ok_or_else(|| invalid("artifact byte count overflowed"))?;
        records += buffer[..read].iter().filter(|byte| **byte == b'\n').count() as u64;
        last = Some(buffer[read - 1]);
    }
    if bytes != binding.byte_length
        || binding.record_count != Some(records)
        || binding.ends_with_lf != (last == Some(b'\n'))
        || format!("sha256:{:x}", hasher.finalize()) != binding.sha256.as_str()
    {
        return Err(invalid(
            "retained artifact byte/hash/record/LF shape differs from manifest",
        ));
    }
    file.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    Ok(())
}

pub(crate) fn verify_retained_artifacts(
    trusted: &mut File,
    interrupted: &mut File,
    tainted: &mut File,
    frozen: &mut File,
    manifest: &GenerationManifest,
) -> Result<Vec<Record>, CoordError> {
    let recovery = manifest.body.recovery()?;
    verify_bound_file(
        trusted,
        &expectation(&recovery.artifacts.trusted_prefix),
        &recovery.artifacts.trusted_prefix,
    )?;
    for (file, binding) in [
        (&mut *interrupted, &recovery.artifacts.interrupted_capture),
        (&mut *tainted, &recovery.artifacts.tainted_generation),
        (&mut *frozen, &recovery.artifacts.frozen_live_source),
    ] {
        verify_bound_file(file, &expectation(binding), binding)?;
        io::verify_prefix(file, &expectation(&recovery.artifacts.trusted_prefix))?;
    }
    verify_post_prefix_inventory(interrupted, tainted, frozen, manifest)?;
    let records = crate::coord::store::legacy::read_records(trusted)?;
    if records.len() as u64 != recovery.trusted_record_count {
        return Err(invalid(
            "trusted-prefix replay count differs from the manifest",
        ));
    }
    let claims = crate::coord::state::summaries(&records, recovery.incident_at_unix_ms)?;
    if projection::inventory(&records, &claims)? != recovery.trusted_projection_inventory {
        return Err(invalid(
            "trusted projection inventory differs from retained replay",
        ));
    }
    let baseline = authority::baseline_record(manifest)?;
    authority::baseline_subject(manifest, &baseline)?;
    let mut transitioned = records.clone();
    transitioned.push(baseline);
    crate::coord::state::summaries(&transitioned, recovery.recovered_at_unix_ms)?;
    trusted.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    Ok(records)
}

fn snapshot_prefix(source: &mut File, binding: &ArtifactBinding) -> Result<File, CoordError> {
    source.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    let descriptor = memfd_create(c"bullet-coord-trusted-prefix", MemFdCreateFlag::MFD_CLOEXEC)
        .map_err(|error| invalid(format!("cannot retain trusted prefix: {error}")))?;
    let mut prefix = File::from(descriptor);
    let copied = std::io::copy(
        &mut Read::by_ref(source).take(binding.byte_length),
        &mut prefix,
    )
    .map_err(CoordError::io)?;
    if copied != binding.byte_length {
        return Err(invalid("trusted prefix ended before its manifest bound"));
    }
    prefix.flush().map_err(CoordError::io)?;
    prefix.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    source.seek(SeekFrom::Start(0)).map_err(CoordError::io)?;
    Ok(prefix)
}

fn expectation(binding: &ArtifactBinding) -> ContentExpectation {
    ContentExpectation {
        byte_length: binding.byte_length,
        sha256: binding.sha256.clone(),
    }
}

pub(super) fn verify_post_prefix_inventory(
    interrupted: &mut File,
    tainted: &mut File,
    frozen: &mut File,
    manifest: &GenerationManifest,
) -> Result<(), CoordError> {
    let observed = compute_post_prefix_inventory(interrupted, tainted, frozen, manifest)?;
    if observed != manifest.body.recovery()?.post_prefix_inventory_blake3 {
        return Err(invalid(
            "recomputed post-prefix inventory differs from the manifest binding",
        ));
    }
    Ok(())
}

pub(super) fn compute_post_prefix_inventory(
    interrupted: &mut File,
    tainted: &mut File,
    frozen: &mut File,
    manifest: &GenerationManifest,
) -> Result<String, CoordError> {
    let recovery = manifest.body.recovery()?;
    let start = recovery.artifacts.trusted_prefix.byte_length;
    let interrupted_observation = observe_range(
        interrupted,
        start,
        recovery.artifacts.interrupted_capture.byte_length,
    )?;
    if interrupted_observation.sha256 != recovery.ambiguous_tail_sha256.as_str() {
        return Err(invalid(
            "recomputed ambiguous-tail SHA-256 differs from the manifest",
        ));
    }
    let tainted_observation = observe_range(
        tainted,
        start,
        recovery.artifacts.tainted_generation.byte_length,
    )?;
    let frozen_observation = observe_range(
        frozen,
        start,
        recovery.artifacts.frozen_live_source.byte_length,
    )?;
    let interrupted_tainted = common_prefix(
        interrupted,
        recovery.artifacts.interrupted_capture.byte_length,
        tainted,
        recovery.artifacts.tainted_generation.byte_length,
        start,
    )?;
    if interrupted_tainted != interrupted_observation.byte_length {
        return Err(invalid(
            "interrupted suffix is not an exact prefix of the tainted capture",
        ));
    }
    let interrupted_frozen = common_prefix(
        interrupted,
        recovery.artifacts.interrupted_capture.byte_length,
        frozen,
        recovery.artifacts.frozen_live_source.byte_length,
        start,
    )?;
    let tainted_frozen = common_prefix(
        tainted,
        recovery.artifacts.tainted_generation.byte_length,
        frozen,
        recovery.artifacts.frozen_live_source.byte_length,
        start,
    )?;
    let subject = serde_json::json!({
        "kind": "coord_post_prefix_inventory_v2",
        "schema_version": 2,
        "artifacts": &recovery.artifacts,
        "trusted_range": { "start_inclusive": 0, "end_exclusive": start },
        "discarded_range": &recovery.discarded_range,
        "ambiguous_tail_range": &recovery.ambiguous_tail_range,
        "ambiguous_tail_sha256": &recovery.ambiguous_tail_sha256,
        "suffixes": {
            "interrupted": interrupted_observation,
            "tainted": tainted_observation,
            "frozen_live": frozen_observation,
        },
        "pairwise_suffix_common_prefix_bytes": {
            "interrupted_tainted": interrupted_tainted,
            "interrupted_frozen": interrupted_frozen,
            "tainted_frozen": tainted_frozen,
        },
        "lineage": recovery.lineage,
        "post_prefix_default": recovery.post_prefix_default,
        "implicit_adoptions": recovery.implicit_adoptions,
    });
    let canonical = bullet_wire::canonical_json(&subject)
        .map_err(|error| invalid(format!("cannot encode post-prefix inventory: {error}")))?;
    Ok(format!(
        "blake3:{}",
        bullet_wire::hash_framed_bytes(INVENTORY_DOMAIN, &canonical)
            .map_err(|error| invalid(format!("cannot hash post-prefix inventory: {error}")))?
            .to_hex()
    ))
}

fn observe_range(file: &mut File, start: u64, end: u64) -> Result<RangeObservation, CoordError> {
    let length = end
        .checked_sub(start)
        .ok_or_else(|| invalid("post-prefix range is inverted"))?;
    file.seek(SeekFrom::Start(start)).map_err(CoordError::io)?;
    let mut reader = Read::by_ref(file).take(length);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut count = 0_u64;
    let mut lines = 0_u64;
    let mut last = None;
    loop {
        let read = reader.read(&mut buffer).map_err(CoordError::io)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        count += read as u64;
        lines += buffer[..read].iter().filter(|byte| **byte == b'\n').count() as u64;
        last = Some(buffer[read - 1]);
    }
    if count != length {
        return Err(invalid("post-prefix range ended before its bound"));
    }
    Ok(RangeObservation {
        byte_length: length,
        sha256: format!("sha256:{:x}", hasher.finalize()),
        lf_count: lines,
        ends_with_lf: last == Some(b'\n'),
    })
}

fn common_prefix(
    left: &mut File,
    left_end: u64,
    right: &mut File,
    right_end: u64,
    start: u64,
) -> Result<u64, CoordError> {
    left.seek(SeekFrom::Start(start)).map_err(CoordError::io)?;
    right.seek(SeekFrom::Start(start)).map_err(CoordError::io)?;
    let maximum = left_end.min(right_end).saturating_sub(start);
    let mut matched = 0_u64;
    let mut left_byte = [0_u8; 1];
    let mut right_byte = [0_u8; 1];
    while matched < maximum {
        left.read_exact(&mut left_byte).map_err(CoordError::io)?;
        right.read_exact(&mut right_byte).map_err(CoordError::io)?;
        if left_byte != right_byte {
            break;
        }
        matched += 1;
    }
    Ok(matched)
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_RECOVERY", reason)
}
