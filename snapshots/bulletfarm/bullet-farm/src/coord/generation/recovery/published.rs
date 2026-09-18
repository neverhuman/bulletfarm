use std::fs::File;

use super::{
    ContentExpectation, authority, exchange, platform_fs as io, support::invalid, tree, verifier,
};
use crate::coord::{
    CoordError,
    generation::manifest::{ArtifactBinding, GenerationManifest},
};

pub(super) struct Guard {
    layout: tree::Layout,
    tombstone: File,
    retired: File,
    lease: verifier::LegacyReadLease,
    source_identity: (u64, u64),
    tombstone_identity: (u64, u64),
    owner: u32,
    sibling: String,
}

impl Guard {
    pub(super) fn revalidate(&self) -> Result<(), CoordError> {
        self.lease.revalidate()
    }
}

pub(super) fn verify(root: &File, manifest: &GenerationManifest) -> Result<Guard, CoordError> {
    manifest.validate()?;
    let recovery = manifest.body.recovery()?;
    let binding = &recovery.artifacts.frozen_live_source;
    let owner = rustix::process::geteuid().as_raw();
    let layout = tree::Layout::open(root, manifest.generation_id().as_str(), owner)?;
    let tombstone_identity = authority::tombstone_identity(root, owner)?;
    let tombstone = authority::retain_tombstone(root, owner, tombstone_identity)?;
    let mut retired = layout.retired_source(binding.byte_length)?;
    let source_identity = verifier::identity(&retired)?;
    if source_identity != (recovery.legacy_source_device, recovery.legacy_source_inode) {
        return Err(invalid(
            "retired source device/inode differs from manifest authority",
        ));
    }
    io::verify_open_file(&mut retired, &expectation(binding))?;
    let lease = verifier::LegacyReadLease::acquire(&retired)?;
    let guard = Guard {
        layout,
        tombstone,
        retired,
        lease,
        source_identity,
        tombstone_identity,
        owner,
        sibling: exchange::sibling_name(manifest.generation_id().as_str())?,
    };
    verify_subject(root, manifest, &guard)?;
    Ok(guard)
}

pub(super) fn reverify(
    root: &File,
    manifest: &GenerationManifest,
    guard: &Guard,
) -> Result<(), CoordError> {
    verify_subject(root, manifest, guard)
}

fn verify_subject(
    root: &File,
    manifest: &GenerationManifest,
    guard: &Guard,
) -> Result<(), CoordError> {
    manifest.validate()?;
    let recovery = manifest.body.recovery()?;
    let binding = &recovery.artifacts.frozen_live_source;
    if guard.owner != rustix::process::geteuid().as_raw()
        || guard.source_identity != (recovery.legacy_source_device, recovery.legacy_source_inode)
    {
        return Err(invalid("published recovery descriptor authority changed"));
    }
    let baseline_record = authority::baseline_record(manifest)?;
    let baseline = authority::baseline_subject(manifest, &baseline_record)?;
    guard.layout.revalidate(root)?;
    if !tree::current_is(root, guard.owner, manifest)? {
        return Err(invalid("published recovery is missing its exact CURRENT"));
    }
    guard.lease.revalidate()?;
    verify_retired(guard, binding)?;
    verify_evidence_chain(root, manifest, &baseline, guard)?;
    revalidate_topology(root, guard)?;
    verify_retired(guard, binding)?;
    verify_evidence_chain(root, manifest, &baseline, guard)?;
    guard.layout.revalidate(root)?;
    exchange::revalidate_final_topology(
        root,
        guard.layout.recovery(),
        &guard.sibling,
        &guard.tombstone,
        &guard.retired,
        guard.owner,
    )?;
    if !tree::current_is(root, guard.owner, manifest)? {
        return Err(invalid(
            "published recovery CURRENT changed during verification",
        ));
    }
    guard.lease.revalidate()
}

fn verify_evidence_chain(
    root: &File,
    manifest: &GenerationManifest,
    baseline: &authority::BaselineSubject,
    guard: &Guard,
) -> Result<(), CoordError> {
    if authority::tombstone_identity(root, guard.owner)? != guard.tombstone_identity {
        return Err(invalid("published recovery tombstone identity changed"));
    }
    let intent_sha256 = authority::write_or_verify_intent(
        guard.layout.recovery(),
        manifest,
        &expectation(&manifest.body.recovery()?.artifacts.frozen_live_source),
        guard.source_identity,
        guard.tombstone_identity,
        baseline,
        false,
    )?;
    let prepared_observation_sha256 = exchange::verify_prepared_observation(
        guard.layout.recovery(),
        manifest,
        baseline,
        &intent_sha256,
        &guard.sibling,
        &guard.tombstone,
        guard.owner,
    )?;
    let tombstone_observation_sha256 = authority::write_or_verify_tombstone_observation(
        root,
        guard.owner,
        guard.layout.recovery(),
        manifest,
        baseline,
        &intent_sha256,
        &prepared_observation_sha256,
        &guard.tombstone,
        false,
    )?;
    exchange::write_or_verify_retirement_observation(
        root,
        guard.layout.recovery(),
        manifest,
        baseline,
        &intent_sha256,
        &prepared_observation_sha256,
        &tombstone_observation_sha256,
        &guard.sibling,
        &guard.tombstone,
        &guard.retired,
        guard.owner,
        false,
    )
}

fn revalidate_topology(root: &File, guard: &Guard) -> Result<(), CoordError> {
    guard.layout.revalidate(root)?;
    exchange::revalidate_final_topology(
        root,
        guard.layout.recovery(),
        &guard.sibling,
        &guard.tombstone,
        &guard.retired,
        guard.owner,
    )
}

fn verify_retired(guard: &Guard, binding: &ArtifactBinding) -> Result<(), CoordError> {
    let mut retired = guard.retired.try_clone().map_err(CoordError::io)?;
    io::verify_open_file(&mut retired, &expectation(binding))?;
    if verifier::identity(&guard.retired)? != guard.source_identity {
        return Err(invalid("retired source descriptor identity changed"));
    }
    Ok(())
}

fn expectation(binding: &ArtifactBinding) -> ContentExpectation {
    ContentExpectation {
        byte_length: binding.byte_length,
        sha256: binding.sha256.clone(),
    }
}
