use std::path::{Path, PathBuf};

use serde::Serialize;

use super::CoordError;

#[cfg(target_os = "linux")]
use std::{fs::File, os::unix::fs::MetadataExt};

#[cfg(target_os = "linux")]
use rustix::fs::{AtFlags, Mode, OFlags, ResolveFlags, open, openat2, statat};

#[cfg(target_os = "linux")]
use super::{
    generation::{
        manifest::{ArtifactBinding, GenerationManifest, Sha256Digest},
        recovery::{
            ContentExpectation, PublishedRecoveryGuard, RecoveryInput, RecoveryOutcome,
            RecoveryState, SourceExpectation, recover_rollover, verify_published_recovery,
            verify_recovery_in_progress,
        },
    },
    model::{
        RecoveryAuthorizationSignatureV1, RecoveryAuthorizationV1, RecoveryBootstrapProvenanceV1,
        RecoveryInspectionV1,
    },
    recovery_manifest::{self, RecoveryInspectionCommand},
};

#[cfg(target_os = "linux")]
const REPORT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecoveryCommand {
    manifest: PathBuf,
    inspection: PathBuf,
    authorization: PathBuf,
    authorization_signature: PathBuf,
    bootstrap_provenance: PathBuf,
    interrupted_capture: PathBuf,
    tainted_generation: PathBuf,
    frozen_live_source: PathBuf,
}

impl RecoveryCommand {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        manifest: PathBuf,
        inspection: PathBuf,
        authorization: PathBuf,
        authorization_signature: PathBuf,
        bootstrap_provenance: PathBuf,
        interrupted_capture: PathBuf,
        tainted_generation: PathBuf,
        frozen_live_source: PathBuf,
    ) -> Self {
        Self {
            manifest,
            inspection,
            authorization,
            authorization_signature,
            bootstrap_provenance,
            interrupted_capture,
            tainted_generation,
            frozen_live_source,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum RecoveryExecutionState {
    Published,
    ResumedAndPublished,
    AlreadyCurrent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryExecution {
    pub(crate) schema_version: u32,
    pub(crate) state: RecoveryExecutionState,
    pub(crate) generation_id: String,
}

pub(super) fn execute(
    family_root: &Path,
    command: &RecoveryCommand,
) -> Result<RecoveryExecution, CoordError> {
    #[cfg(target_os = "linux")]
    {
        execute_linux(family_root, command)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let RecoveryCommand {
            manifest,
            inspection,
            authorization,
            authorization_signature,
            bootstrap_provenance,
            interrupted_capture,
            tainted_generation,
            frozen_live_source,
        } = command;
        let _ = (
            family_root,
            manifest,
            inspection,
            authorization,
            authorization_signature,
            bootstrap_provenance,
            interrupted_capture,
            tainted_generation,
            frozen_live_source,
        );
        Err(CoordError::new(
            "COORD_RECOVERY_PLATFORM_UNSUPPORTED",
            "supervised coordination recovery is implemented only on Linux",
        ))
    }
}

#[cfg(target_os = "linux")]
fn execute_linux(
    family_root: &Path,
    command: &RecoveryCommand,
) -> Result<RecoveryExecution, CoordError> {
    let authority = prepare_authority(family_root, command)?;
    authority.authorized.require_read_only_replay()?;
    if let Some(execution) = verify_current(&authority.coord_dir, &authority.authorized.manifest)? {
        return Ok(execution);
    }
    authority.authorized.require_active()?;
    let prepared = prepare_mutation(family_root, command, authority)?;
    prepared.authorized.require_active()?;
    let outcome = recover_rollover(&prepared.input, &prepared.manifest, || {
        #[cfg(test)]
        tests::rebind_clock_at_authority();
        prepared.authorized.require_active()
    })?;
    execution(outcome)
}

#[cfg(target_os = "linux")]
fn execution(outcome: RecoveryOutcome) -> Result<RecoveryExecution, CoordError> {
    let state = match outcome.state {
        RecoveryState::Published => RecoveryExecutionState::Published,
        RecoveryState::ResumedAndPublished => RecoveryExecutionState::ResumedAndPublished,
        RecoveryState::AlreadyCurrent => RecoveryExecutionState::AlreadyCurrent,
        RecoveryState::FrozenWaitingForLegacyWriters => {
            return Err(CoordError::new(
                "COORD_RECOVERY_WRITER_WAIT",
                format!(
                    "generation {} remains frozen until legacy writers release the retired source",
                    outcome.generation_id
                ),
            ));
        }
    };
    Ok(RecoveryExecution {
        schema_version: REPORT_SCHEMA_VERSION,
        state,
        generation_id: outcome.generation_id,
    })
}

#[cfg(target_os = "linux")]
struct PreparedRecovery {
    manifest: GenerationManifest,
    input: RecoveryInput,
    authorized: recovery_manifest::AuthorizedManifest,
}

#[cfg(target_os = "linux")]
struct PreparedAuthority {
    authorized: recovery_manifest::AuthorizedManifest,
    inspection: RecoveryInspectionV1,
    inspection_command: RecoveryInspectionCommand,
    coord_dir: PathBuf,
}

#[cfg(target_os = "linux")]
fn prepare_authority(
    family_root: &Path,
    command: &RecoveryCommand,
) -> Result<PreparedAuthority, CoordError> {
    for (path, label) in [
        (&command.manifest, "manifest"),
        (&command.inspection, "inspection"),
        (&command.authorization, "authorization"),
        (&command.authorization_signature, "authorization signature"),
        (&command.bootstrap_provenance, "bootstrap provenance"),
    ] {
        require_command_path(path, label)?;
    }
    require_command_path(&command.interrupted_capture, "interrupted capture")?;
    require_command_path(&command.tainted_generation, "tainted generation")?;
    require_command_path(&command.frozen_live_source, "frozen live source")?;
    let coord_dir = family_root.join(".bullet-family/coord");
    require_command_path(&coord_dir, "coordinator root")?;
    if command.frozen_live_source != coord_dir.join("events.jsonl") {
        return Err(invalid(
            "frozen live source must be the selected family's exact legacy coordinator source",
        ));
    }

    let manifest = super::sealed::read::<GenerationManifest>(&command.manifest)?;
    let inspection = super::sealed::read::<RecoveryInspectionV1>(&command.inspection)?;
    let authorization = super::sealed::read::<RecoveryAuthorizationV1>(&command.authorization)?;
    let signature =
        super::sealed::read::<RecoveryAuthorizationSignatureV1>(&command.authorization_signature)?;
    let provenance =
        super::sealed::read::<RecoveryBootstrapProvenanceV1>(&command.bootstrap_provenance)?;
    let authorized =
        recovery_manifest::authorize(&inspection, &authorization, &signature, &provenance)?;
    if manifest != authorized.manifest {
        return Err(invalid(
            "supplied recovery manifest differs from the authority-derived manifest",
        ));
    }

    Ok(PreparedAuthority {
        authorized,
        inspection,
        inspection_command: RecoveryInspectionCommand {
            interrupted_capture: command.interrupted_capture.clone(),
            tainted_generation: command.tainted_generation.clone(),
            frozen_live_source: command.frozen_live_source.clone(),
        },
        coord_dir,
    })
}

#[cfg(target_os = "linux")]
fn prepare_mutation(
    family_root: &Path,
    command: &RecoveryCommand,
    authority: PreparedAuthority,
) -> Result<PreparedRecovery, CoordError> {
    let manifest = authority.authorized.manifest.clone();
    let recovery = manifest.body.recovery()?;
    for (path, binding, label) in [
        (
            &command.interrupted_capture,
            &recovery.artifacts.interrupted_capture,
            "interrupted capture",
        ),
        (
            &command.tainted_generation,
            &recovery.artifacts.tainted_generation,
            "tainted generation",
        ),
    ] {
        verify_raw_binding(path, binding, label)?;
    }
    let input = RecoveryInput {
        coord_dir: authority.coord_dir,
        trusted_prefix: expectation(&recovery.artifacts.trusted_prefix),
        interrupted_capture: SourceExpectation {
            path: command.interrupted_capture.clone(),
            content: expectation(&recovery.artifacts.interrupted_capture),
        },
        tainted_generation: SourceExpectation {
            path: command.tainted_generation.clone(),
            content: expectation(&recovery.artifacts.tainted_generation),
        },
        frozen_live_source: SourceExpectation {
            path: command.frozen_live_source.clone(),
            content: expectation(&recovery.artifacts.frozen_live_source),
        },
    };
    let fresh_inspection = verify_legacy_binding(
        &command.frozen_live_source,
        &recovery.artifacts.frozen_live_source,
        "frozen live source",
    )
    .and_then(|()| {
        #[cfg(test)]
        if tests::take_fresh_inspection_failure() {
            return Err(changed("injected fresh inspection failure"));
        }
        recovery_manifest::inspect(family_root, &authority.inspection_command)
    });
    match fresh_inspection {
        Ok(observed) if observed != authority.inspection => {
            return Err(changed(
                "sealed inspection differs from complete source rederivation",
            ));
        }
        Err(error) if !verify_recovery_in_progress(&input, &manifest)? => return Err(error),
        Ok(_) | Err(_) => {}
    }
    Ok(PreparedRecovery {
        input,
        manifest,
        authorized: authority.authorized,
    })
}

#[cfg(target_os = "linux")]
fn verify_legacy_binding(
    path: &Path,
    binding: &ArtifactBinding,
    label: &'static str,
) -> Result<(), CoordError> {
    let bytes = super::sealed::read_raw_legacy_live(path, binding.byte_length)?;
    verify_binding_bytes(&bytes, binding, label)
}

#[cfg(target_os = "linux")]
fn verify_raw_binding(
    path: &Path,
    binding: &ArtifactBinding,
    label: &'static str,
) -> Result<(), CoordError> {
    let bytes = super::sealed::read_raw(path, binding.byte_length)?;
    verify_binding_bytes(&bytes, binding, label)
}

#[cfg(target_os = "linux")]
fn verify_binding_bytes(
    bytes: &[u8],
    binding: &ArtifactBinding,
    label: &'static str,
) -> Result<(), CoordError> {
    if bytes.len() as u64 != binding.byte_length || Sha256Digest::for_bytes(bytes) != binding.sha256
    {
        return Err(changed(format!(
            "{label} length or SHA-256 differs from the recovery manifest"
        )));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn expectation(binding: &ArtifactBinding) -> ContentExpectation {
    ContentExpectation {
        byte_length: binding.byte_length,
        sha256: binding.sha256.clone(),
    }
}

#[cfg(target_os = "linux")]
fn verify_current(
    coord_dir: &Path,
    manifest: &GenerationManifest,
) -> Result<Option<RecoveryExecution>, CoordError> {
    let (root, identity) = open_coord_root(coord_dir, true)?;
    match statat(&root, "CURRENT", AtFlags::SYMLINK_NOFOLLOW) {
        Err(rustix::io::Errno::NOENT) => {
            revalidate_coord_root(coord_dir, identity)?;
            Ok(None)
        }
        Err(error) => Err(invalid(format!("cannot inspect recovery CURRENT: {error}"))),
        Ok(_) => {
            identity.require_tightened()?;
            let guard: PublishedRecoveryGuard = verify_published_recovery(&root, manifest)?;
            guard.revalidate()?;
            revalidate_coord_root(coord_dir, identity)?;
            Ok(Some(RecoveryExecution {
                schema_version: REPORT_SCHEMA_VERSION,
                state: RecoveryExecutionState::AlreadyCurrent,
                generation_id: manifest.generation_id().as_str().to_owned(),
            }))
        }
    }
}

#[cfg(target_os = "linux")]
fn open_coord_root(
    path: &Path,
    allow_legacy_mode: bool,
) -> Result<(File, DirectoryIdentity), CoordError> {
    let filesystem = open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map_err(|error| invalid(format!("cannot open filesystem root: {error}")))?;
    let relative = path
        .strip_prefix("/")
        .map_err(|_| invalid("coordinator root must be beneath the filesystem root"))?;
    let descriptor = openat2(
        &filesystem,
        relative,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
        ResolveFlags::BENEATH | ResolveFlags::NO_SYMLINKS | ResolveFlags::NO_MAGICLINKS,
    )
    .map_err(|error| invalid(format!("cannot open coordinator root: {error}")))?;
    let root = File::from(descriptor);
    let identity = DirectoryIdentity::for_file(&root)?;
    identity.validate(allow_legacy_mode)?;
    Ok((root, identity))
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DirectoryIdentity {
    dev: u64,
    ino: u64,
    mode: u32,
    uid: u32,
    gid: u32,
    nlink: u64,
    mtime: i64,
    mtime_nsec: i64,
    ctime: i64,
    ctime_nsec: i64,
}

#[cfg(target_os = "linux")]
impl DirectoryIdentity {
    fn for_file(file: &File) -> Result<Self, CoordError> {
        let metadata = file.metadata().map_err(CoordError::io)?;
        if !metadata.is_dir() {
            return Err(invalid("coordinator root is not a directory"));
        }
        Ok(Self {
            dev: metadata.dev(),
            ino: metadata.ino(),
            mode: metadata.mode() & 0o7777,
            uid: metadata.uid(),
            gid: metadata.gid(),
            nlink: metadata.nlink(),
            mtime: metadata.mtime(),
            mtime_nsec: metadata.mtime_nsec(),
            ctime: metadata.ctime(),
            ctime_nsec: metadata.ctime_nsec(),
        })
    }

    fn validate(self, allow_legacy_mode: bool) -> Result<(), CoordError> {
        let admitted_mode = self.mode == 0o700 || (allow_legacy_mode && self.mode == 0o775);
        if self.uid != rustix::process::geteuid().as_raw() || !admitted_mode {
            return Err(invalid(
                "coordinator root owner or role-specific exact mode is not admitted",
            ));
        }
        Ok(())
    }

    fn require_tightened(self) -> Result<(), CoordError> {
        self.validate(false)
    }
}

#[cfg(target_os = "linux")]
fn revalidate_coord_root(path: &Path, expected: DirectoryIdentity) -> Result<(), CoordError> {
    let (_, observed) = open_coord_root(path, expected.mode == 0o775)?;
    if observed != expected {
        return Err(changed(
            "coordinator root changed during published verification",
        ));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn require_command_path(path: &Path, label: &'static str) -> Result<(), CoordError> {
    if !recovery_manifest::is_normalized_absolute(path) {
        return Err(invalid(format!(
            "{label} path must be normalized absolute lexical bytes"
        )));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_RECOVERY", reason)
}

#[cfg(target_os = "linux")]
fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_RECOVERY_SOURCE_CHANGED", reason)
}

#[cfg(all(test, target_os = "linux"))]
mod tests;
