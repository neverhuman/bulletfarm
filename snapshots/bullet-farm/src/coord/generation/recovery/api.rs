use std::path::PathBuf;

use crate::coord::CoordError;

use super::{GenerationManifest, Sha256Digest, linux};

#[derive(Clone, Debug)]
pub(crate) struct ContentExpectation {
    pub(crate) byte_length: u64,
    pub(crate) sha256: Sha256Digest,
}

#[derive(Clone, Debug)]
pub(crate) struct SourceExpectation {
    pub(crate) path: PathBuf,
    pub(crate) content: ContentExpectation,
}

#[derive(Clone, Debug)]
pub(crate) struct RecoveryInput {
    pub(crate) coord_dir: PathBuf,
    pub(crate) trusted_prefix: ContentExpectation,
    pub(crate) interrupted_capture: SourceExpectation,
    pub(crate) tainted_generation: SourceExpectation,
    pub(crate) frozen_live_source: SourceExpectation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RecoveryState {
    Published,
    ResumedAndPublished,
    AlreadyCurrent,
    FrozenWaitingForLegacyWriters,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecoveryOutcome {
    pub(crate) state: RecoveryState,
    pub(crate) generation_id: String,
    pub(crate) retired_source: PathBuf,
}

pub(crate) fn recover_rollover(
    input: &RecoveryInput,
    manifest: &GenerationManifest,
    revalidate_authority: impl FnMut() -> Result<(), CoordError>,
) -> Result<RecoveryOutcome, CoordError> {
    #[cfg(target_os = "linux")]
    {
        linux::recover(input, manifest, revalidate_authority)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (input, manifest, revalidate_authority);
        Err(CoordError::new(
            "COORD_RECOVERY_PLATFORM_UNSUPPORTED",
            "legacy exchange and writable-descriptor proof are implemented only on Linux",
        ))
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn verify_recovery_in_progress(
    input: &RecoveryInput,
    manifest: &GenerationManifest,
) -> Result<bool, CoordError> {
    linux::verify_in_progress(input, manifest)
}
