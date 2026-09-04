use std::path::Path;

use super::{ContentExpectation, RecoveryOutcome, RecoveryState, authority, tree};
use crate::coord::{
    CoordError,
    generation::manifest::{ArtifactBinding, CurrentPointer, GenerationManifest},
};

pub(super) fn expectation(binding: &ArtifactBinding) -> ContentExpectation {
    ContentExpectation {
        byte_length: binding.byte_length,
        sha256: binding.sha256.clone(),
    }
}

pub(super) fn publish_current(
    authority: &authority::Authority,
    coord: &Path,
    manifest: &GenerationManifest,
) -> Result<(), CoordError> {
    if tree::current_is(authority.root(), authority.owner(), manifest)? {
        return Ok(());
    }
    let pointer = CurrentPointer::for_manifest(manifest)?;
    authority.publish_current(
        coord,
        manifest.generation_id().as_str(),
        &pointer.canonical_bytes()?,
    )?;
    if !tree::current_is(authority.root(), authority.owner(), manifest)? {
        return Err(invalid("CURRENT read-back differs after publication"));
    }
    Ok(())
}

pub(super) fn outcome(
    state: RecoveryState,
    generation_id: String,
    retired_source: std::path::PathBuf,
) -> RecoveryOutcome {
    RecoveryOutcome {
        state,
        generation_id,
        retired_source,
    }
}

pub(super) fn invalid(reason: &'static str) -> CoordError {
    CoordError::new("INVALID_COORD_RECOVERY", reason)
}
