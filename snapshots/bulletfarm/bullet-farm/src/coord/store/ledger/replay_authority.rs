use super::*;

#[cfg(all(test, target_os = "linux"))]
use std::cell::{Cell, RefCell};

#[cfg(test)]
use std::path::PathBuf;

#[cfg(all(test, target_os = "linux"))]
thread_local! {
    static MUTATE_GENESIS_AFTER_FIRST_VALIDATION: Cell<bool> = const { Cell::new(false) };
    static MUTATE_RECOVERY_AFTER_FIRST_VALIDATION: Cell<bool> = const { Cell::new(false) };
    static SWAP_SUBJECT_BEFORE_PENDING_RECONCILE: RefCell<Option<(PathBuf, PathBuf)>> = const {
        RefCell::new(None)
    };
    static REWRITE_MANIFEST_BEFORE_FINAL_REPLAY: RefCell<Option<PathBuf>> = const {
        RefCell::new(None)
    };
}

pub(super) fn verify_genesis<F>(
    lock: &fs::CoordLock,
    manifest: &GenerationManifest,
    pointer: &CurrentPointer,
    after_intent: F,
) -> Result<(), CoordError>
where
    F: FnOnce() -> Result<(), CoordError>,
{
    let intent = fs::published_genesis_intent(lock)?;
    let initialized = genesis::decode_authority(&intent)
        .map_err(|_| fence_unknown("Genesis initialization intent is invalid"))?;
    if initialized.manifest != *manifest || initialized.current != *pointer {
        return Err(fence_unknown(
            "Genesis initialization intent differs from CURRENT authority",
        ));
    }
    after_intent()?;
    fs::validate_genesis_tombstone(lock, manifest.generation_id().as_str(), &intent)
}

#[cfg(all(test, target_os = "linux"))]
pub(super) fn test_mutate_genesis_after_first_validation() {
    MUTATE_GENESIS_AFTER_FIRST_VALIDATION.with(|selected| selected.set(true));
}

#[cfg(all(test, not(target_os = "linux")))]
pub(super) fn test_mutate_genesis_after_first_validation() {}

#[cfg(all(test, target_os = "linux"))]
pub(super) fn test_mutate_recovery_after_first_validation() {
    MUTATE_RECOVERY_AFTER_FIRST_VALIDATION.with(|selected| selected.set(true));
}

#[cfg(all(test, not(target_os = "linux")))]
pub(super) fn test_mutate_recovery_after_first_validation() {}

#[cfg(all(test, target_os = "linux"))]
pub(super) fn test_swap_subject_before_pending_reconcile(canonical: PathBuf, replacement: PathBuf) {
    SWAP_SUBJECT_BEFORE_PENDING_RECONCILE.with(|selected| {
        assert!(
            selected
                .borrow_mut()
                .replace((canonical, replacement))
                .is_none()
        );
    });
}

#[cfg(all(test, target_os = "linux"))]
pub(super) fn test_rewrite_manifest_before_final_replay(path: PathBuf) {
    REWRITE_MANIFEST_BEFORE_FINAL_REPLAY.with(|selected| {
        assert!(selected.borrow_mut().replace(path).is_none());
    });
}

#[cfg(all(test, not(target_os = "linux")))]
pub(super) fn test_rewrite_manifest_before_final_replay(path: PathBuf) {
    let _ = path;
}

#[cfg(all(test, not(target_os = "linux")))]
pub(super) fn test_swap_subject_before_pending_reconcile(canonical: PathBuf, replacement: PathBuf) {
    let _ = (canonical, replacement);
}

#[cfg(all(test, target_os = "linux"))]
pub(super) fn inject_pre_effect_subject_swap() -> Result<(), CoordError> {
    let Some((canonical, replacement)) =
        SWAP_SUBJECT_BEFORE_PENDING_RECONCILE.with(|selected| selected.borrow_mut().take())
    else {
        return Ok(());
    };
    let displaced = canonical.with_file_name(".CURRENT.displaced-reconcile-test");
    std::fs::rename(&canonical, displaced).map_err(CoordError::io)?;
    std::fs::rename(replacement, canonical).map_err(CoordError::io)
}

#[cfg(all(test, not(target_os = "linux")))]
pub(super) fn inject_pre_effect_subject_swap() -> Result<(), CoordError> {
    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
pub(super) fn inject_final_manifest_rewrite() -> Result<(), CoordError> {
    let Some(path) =
        REWRITE_MANIFEST_BEFORE_FINAL_REPLAY.with(|selected| selected.borrow_mut().take())
    else {
        return Ok(());
    };
    rewrite(&path, "test generation manifest is empty")
}

#[cfg(all(test, not(target_os = "linux")))]
pub(super) fn inject_final_manifest_rewrite() -> Result<(), CoordError> {
    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
pub(super) fn inject_genesis(coord_dir: &Path) -> Result<(), CoordError> {
    if !MUTATE_GENESIS_AFTER_FIRST_VALIDATION.with(Cell::take) {
        return Ok(());
    }
    rewrite(
        &coord_dir.join("genesis-init-intent.json"),
        "test Genesis initialization intent is empty",
    )
}

#[cfg(all(test, not(target_os = "linux")))]
pub(super) fn inject_genesis(coord_dir: &Path) -> Result<(), CoordError> {
    let _ = coord_dir;
    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
pub(super) fn inject_recovery(
    coord_dir: &Path,
    manifest: &GenerationManifest,
) -> Result<(), CoordError> {
    if !MUTATE_RECOVERY_AFTER_FIRST_VALIDATION.with(Cell::take) {
        return Ok(());
    }
    rewrite(
        &coord_dir
            .join("recovery")
            .join(manifest.generation_id().as_str())
            .join("tombstone-seal-observation.json"),
        "test tombstone observation is empty",
    )
}

#[cfg(all(test, not(target_os = "linux")))]
pub(super) fn inject_recovery(
    coord_dir: &Path,
    manifest: &GenerationManifest,
) -> Result<(), CoordError> {
    let _ = (coord_dir, manifest);
    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
fn rewrite(path: &Path, empty_reason: &str) -> Result<(), CoordError> {
    use std::{fs, os::unix::fs::PermissionsExt};

    let mut bytes = fs::read(path).map_err(CoordError::io)?;
    let first = bytes.first_mut().ok_or_else(|| invalid(empty_reason))?;
    *first ^= 1;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).map_err(CoordError::io)?;
    fs::write(path, bytes).map_err(CoordError::io)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o400)).map_err(CoordError::io)
}
