use std::{collections::BTreeSet, fs::File, os::fd::AsRawFd};

use super::{authority::BaselineSubject, verifier};
use crate::coord::{
    CoordError,
    generation::{
        manifest::{ArtifactBinding, CurrentPointer, GenerationManifest},
        segment::{self, AppendRequest},
    },
    model::Record,
};
use nix::{
    errno::Errno,
    fcntl::{RenameFlags, renameat2},
};

#[path = "tree/io.rs"]
mod io;

const GENERATIONS: &str = "generations";
const RECOVERY: &str = "recovery";
const MANIFEST: &str = "manifest.json";
const SEGMENT: &str = "events.jsonl";
const PENDING: &str = "pending";

pub(super) struct Layout {
    generations: File,
    recovery_parent: File,
    recovery_generation: File,
    identities: [io::Identity; 3],
    owner: u32,
    generation_id: String,
}

impl Layout {
    pub(super) fn ensure(root: &File, id: &str, owner: u32) -> Result<Self, CoordError> {
        let generations = io::ensure_dir(root, GENERATIONS, owner)?;
        let recovery = io::ensure_dir(root, RECOVERY, owner)?;
        let recovery_generation = io::ensure_dir(&recovery, id, owner)?;
        io::require_same_device(root, &generations)?;
        let identities = [
            io::identity(&generations)?,
            io::identity(&recovery)?,
            io::identity(&recovery_generation)?,
        ];
        Ok(Self {
            generations,
            recovery_parent: recovery,
            recovery_generation,
            identities,
            owner,
            generation_id: id.to_owned(),
        })
    }

    pub(super) fn open(root: &File, id: &str, owner: u32) -> Result<Self, CoordError> {
        let generations = io::open_dir(root, GENERATIONS, owner)?;
        let recovery = io::open_dir(root, RECOVERY, owner)?;
        let recovery_generation = io::open_dir(&recovery, id, owner)?;
        let identities = [
            io::identity(&generations)?,
            io::identity(&recovery)?,
            io::identity(&recovery_generation)?,
        ];
        Ok(Self {
            generations,
            recovery_parent: recovery,
            recovery_generation,
            identities,
            owner,
            generation_id: id.to_owned(),
        })
    }

    pub(super) const fn recovery(&self) -> &File {
        &self.recovery_generation
    }

    pub(super) fn retired_source(&self, length: u64) -> Result<File, CoordError> {
        io::open_file(
            &self.recovery_generation,
            "retired-v1.non-authoritative",
            self.owner,
            0o400,
            Some(length),
            false,
        )
    }

    pub(super) fn revalidate(&self, root: &File) -> Result<(), CoordError> {
        let generations = io::open_dir(root, GENERATIONS, self.owner)?;
        let recovery = io::open_dir(root, RECOVERY, self.owner)?;
        let recovery_generation = io::open_dir(&recovery, &self.generation_id, self.owner)?;
        if [
            io::identity(&generations)?,
            io::identity(&recovery)?,
            io::identity(&recovery_generation)?,
        ] != self.identities
            || io::identity(&self.generations)? != self.identities[0]
            || io::identity(&self.recovery_parent)? != self.identities[1]
            || io::identity(&self.recovery_generation)? != self.identities[2]
        {
            return Err(changed("recovery layout descriptor hierarchy changed"));
        }
        Ok(())
    }

    pub(super) fn generation_exists(&self) -> Result<bool, CoordError> {
        let staging_name = format!(".next-{}", self.generation_id);
        let final_generation =
            io::optional_dir(&self.generations, &self.generation_id, self.owner)?;
        let staging = io::optional_dir(&self.generations, &staging_name, self.owner)?;
        match (final_generation.is_some(), staging.is_some()) {
            (true, true) => Err(build_unknown(
                "final generation and deterministic stage both exist",
            )),
            (true, false) => Ok(true),
            (false, _) => Ok(false),
        }
    }

    pub(super) fn build_generation(
        &self,
        interrupted: &mut File,
        tainted: &mut File,
        legacy: &mut File,
        manifest: &GenerationManifest,
        baseline: &Record,
        subject: &BaselineSubject,
    ) -> Result<(), CoordError> {
        let staging_name = format!(".next-{}", self.generation_id);
        let staging = io::ensure_dir(&self.generations, &staging_name, self.owner)?;
        require_subset(&staging, &["archive", MANIFEST, SEGMENT, PENDING])?;
        let staging_identity = io::identity(&staging)?;
        let archive = io::ensure_dir(&staging, "archive", self.owner)?;
        let artifacts = &manifest.body.recovery()?.artifacts;
        require_subset(
            &archive,
            &[
                io::artifact_name(&artifacts.trusted_prefix)?,
                io::artifact_name(&artifacts.interrupted_capture)?,
                io::artifact_name(&artifacts.tainted_generation)?,
                io::artifact_name(&artifacts.frozen_live_source)?,
            ],
        )?;
        io::copy_or_verify(
            &archive,
            &artifacts.trusted_prefix,
            interrupted,
            true,
            self.owner,
        )?;
        io::copy_or_verify(
            &archive,
            &artifacts.interrupted_capture,
            interrupted,
            false,
            self.owner,
        )?;
        io::copy_or_verify(
            &archive,
            &artifacts.tainted_generation,
            tainted,
            false,
            self.owner,
        )?;
        io::copy_or_verify(
            &archive,
            &artifacts.frozen_live_source,
            legacy,
            false,
            self.owner,
        )?;
        io::write_or_verify(
            &staging,
            MANIFEST,
            &manifest.canonical_bytes()?,
            self.owner,
            0o400,
        )?;
        initialize_segment(&staging, self.owner, baseline, subject, manifest)?;
        require_exact(&staging, &["archive", MANIFEST, SEGMENT, PENDING])?;
        io::revalidate_child(&self.generations, &staging_name, &staging, self.owner, true)?;
        if io::identity(&staging)? != staging_identity {
            return Err(changed("generation staging identity changed"));
        }
        verify_at(&staging, self.owner, manifest, baseline, subject)?;
        staging.sync_all().map_err(CoordError::io)?;
        renameat2(
            Some(self.generations.as_raw_fd()),
            staging_name.as_str(),
            Some(self.generations.as_raw_fd()),
            self.generation_id.as_str(),
            RenameFlags::RENAME_NOREPLACE,
        )
        .map_err(|error| match error {
            Errno::EEXIST => CoordError::new(
                "COORD_RECOVERY_COLLISION",
                "generation destination appeared",
            ),
            _ => CoordError::new("COORD_GENERATION_PUBLISH_FAILED", error.to_string()),
        })?;
        self.generations.sync_all().map_err(CoordError::io)?;
        let published = io::open_dir(&self.generations, &self.generation_id, self.owner)?;
        if io::identity(&published)? != staging_identity {
            return Err(changed(
                "published generation identity differs from staging",
            ));
        }
        self.verify_generation(manifest, baseline, subject)
    }

    pub(super) fn verify_generation(
        &self,
        manifest: &GenerationManifest,
        baseline: &Record,
        subject: &BaselineSubject,
    ) -> Result<(), CoordError> {
        let generation = io::optional_dir(&self.generations, &self.generation_id, self.owner)?
            .ok_or_else(|| changed("immutable generation is missing"))?;
        let identity = io::identity(&generation)?;
        require_exact(&generation, &["archive", MANIFEST, SEGMENT, PENDING])?;
        verify_at(&generation, self.owner, manifest, baseline, subject)?;
        io::revalidate_child(
            &self.generations,
            &self.generation_id,
            &generation,
            self.owner,
            true,
        )?;
        if io::identity(&generation)? != identity {
            return Err(changed("generation changed during exact verification"));
        }
        Ok(())
    }
}

pub(super) fn current_is(
    root: &File,
    owner: u32,
    manifest: &GenerationManifest,
) -> Result<bool, CoordError> {
    let expected_stage = format!(".CURRENT.next-{}", manifest.generation_id().as_str());
    let stages = io::list_names(root)?
        .into_iter()
        .filter(|name| name.contains("CURRENT.next"))
        .collect::<Vec<_>>();
    if stages.len() > 1 || stages.first().is_some_and(|name| name != &expected_stage) {
        return Err(CoordError::new(
            "COORD_CURRENT_OUTCOME_UNKNOWN",
            "CURRENT stage inventory contains an unbound subject",
        ));
    }
    let Some(mut file) = io::optional_file(root, "CURRENT", owner, 0o400)? else {
        return Ok(false);
    };
    if !stages.is_empty() {
        return Err(CoordError::new(
            "COORD_CURRENT_OUTCOME_UNKNOWN",
            "CURRENT and its stage both exist",
        ));
    }
    let bytes = io::stable_read(&mut file, bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES)?;
    if io::stable_read(&mut file, bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES)? != bytes {
        return Err(CoordError::new(
            "COORD_CURRENT_OUTCOME_UNKNOWN",
            "CURRENT changed between stable reads",
        ));
    }
    io::revalidate_child(root, "CURRENT", &file, owner, false)?;
    let pointer = CurrentPointer::decode_canonical(&bytes)?;
    pointer.validate()?;
    if pointer.generation_id().as_str() != manifest.generation_id().as_str() {
        return Err(CoordError::new(
            "COORD_CURRENT_CONFLICT",
            "CURRENT names another generation",
        ));
    }
    pointer.verify_manifest(manifest)?;
    Ok(true)
}

fn initialize_segment(
    generation: &File,
    owner: u32,
    baseline: &Record,
    subject: &BaselineSubject,
    manifest: &GenerationManifest,
) -> Result<(), CoordError> {
    if io::optional_file(generation, SEGMENT, owner, 0o600)?.is_none() {
        io::write_or_verify(generation, SEGMENT, b"", owner, 0o600)?;
    }
    let pending = io::ensure_dir(generation, PENDING, owner)?;
    let mut segment = io::open_file(generation, SEGMENT, owner, 0o600, None, true)?;
    segment::append_files(
        &mut segment,
        &pending,
        &AppendRequest {
            generation_id: manifest.generation_id().as_str(),
            sequence: 1,
            previous_digest: &subject.genesis_digest,
            request_id: &subject.request_id,
            record: baseline,
        },
        &subject.genesis_digest,
    )?;
    verify_segment(generation, owner, baseline, subject, manifest)
}

fn verify_segment(
    generation: &File,
    owner: u32,
    baseline: &Record,
    subject: &BaselineSubject,
    manifest: &GenerationManifest,
) -> Result<(), CoordError> {
    let mut segment = io::open_file(generation, SEGMENT, owner, 0o600, None, true)?;
    let pending = io::open_dir(generation, PENDING, owner)?;
    let inspected = segment::inspect_files(
        &mut segment,
        &pending,
        manifest.generation_id().as_str(),
        &subject.genesis_digest,
    )?;
    if inspected.entries.len() != 1
        || inspected.entries[0].sequence != 1
        || inspected.entries[0].generation_id != manifest.generation_id().as_str()
        || inspected.entries[0].previous_digest != subject.genesis_digest
        || inspected.entries[0].request_id != subject.request_id
        || inspected.entries[0].request_digest != subject.request_digest
        || inspected.position.next_sequence != 2
        || inspected.position.byte_length != inspected.entries[0].receipt.frame_length
        || inspected.position.previous_digest != inspected.entries[0].receipt.envelope_digest
        || inspected
            .receipt_for_request(&subject.request_id)
            .is_none_or(|receipt| receipt != &inspected.entries[0].receipt)
        || bullet_wire::canonical_json(&inspected.entries[0].record).map_err(wire)?
            != bullet_wire::canonical_json(baseline).map_err(wire)?
    {
        return Err(changed("generation segment differs from exact baseline"));
    }
    Ok(())
}

fn verify_at(
    generation: &File,
    owner: u32,
    manifest: &GenerationManifest,
    baseline: &Record,
    subject: &BaselineSubject,
) -> Result<(), CoordError> {
    let mut manifest_file = io::open_file(generation, MANIFEST, owner, 0o400, None, false)?;
    let bytes = io::stable_read(
        &mut manifest_file,
        bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES,
    )?;
    if GenerationManifest::decode_canonical(&bytes)? != *manifest {
        return Err(changed("generation manifest differs from exact request"));
    }
    let archive = io::open_dir(generation, "archive", owner)?;
    let recovery = manifest.body.recovery()?;
    let retained = |binding: &ArtifactBinding| {
        io::open_file(
            &archive,
            io::artifact_name(binding)?,
            owner,
            0o400,
            Some(binding.byte_length),
            false,
        )
    };
    let mut trusted = retained(&recovery.artifacts.trusted_prefix)?;
    let mut interrupted = retained(&recovery.artifacts.interrupted_capture)?;
    let mut tainted = retained(&recovery.artifacts.tainted_generation)?;
    let mut frozen = retained(&recovery.artifacts.frozen_live_source)?;
    verifier::verify_retained_artifacts(
        &mut trusted,
        &mut interrupted,
        &mut tainted,
        &mut frozen,
        manifest,
    )?;
    verify_segment(generation, owner, baseline, subject, manifest)
}

fn require_subset(directory: &File, allowed: &[&str]) -> Result<(), CoordError> {
    let allowed = allowed.iter().copied().collect::<BTreeSet<_>>();
    if io::list_names(directory)?
        .iter()
        .any(|name| !allowed.contains(name.as_str()))
    {
        return Err(changed("staging directory contains an unbound child"));
    }
    Ok(())
}

fn require_exact(directory: &File, expected: &[&str]) -> Result<(), CoordError> {
    let expected = expected.iter().map(|name| (*name).to_owned()).collect();
    if io::list_names(directory)? != expected {
        return Err(changed("directory inventory differs from exact children"));
    }
    Ok(())
}

fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_RECOVERY_SUBJECT_CHANGED", reason)
}

fn build_unknown(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_GENERATION_BUILD_OUTCOME_UNKNOWN", reason)
}

fn wire(error: bullet_wire::WireError) -> CoordError {
    CoordError::new("INVALID_COORD_BASELINE", error.to_string())
}
