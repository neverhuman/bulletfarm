use std::fs::File;

#[cfg(test)]
use std::cell::Cell;

use rustix::fs::{Mode, fchmod, mkdirat};

use super::*;
mod evidence;
use evidence::*;
mod transition;
use transition::*;

const AUTHORITY: &str = "events.jsonl";

#[cfg(test)]
thread_local! {
    static CRASH_AFTER: Cell<Option<&'static str>> = const { Cell::new(None) };
    static KILL_AFTER: Cell<Option<&'static str>> = const { Cell::new(None) };
    static INSERT_AFTER: Cell<Option<&'static str>> = const { Cell::new(None) };
}

#[cfg(test)]
pub(super) fn test_crash_after(phase: &'static str) {
    CRASH_AFTER.with(|value| value.set(Some(phase)));
}

#[cfg(test)]
pub(super) fn test_kill_after(phase: &'static str) {
    KILL_AFTER.with(|value| value.set(Some(phase)));
}

#[cfg(test)]
pub(super) fn test_insert_after(phase: &'static str) {
    INSERT_AFTER.with(|value| value.set(Some(phase)));
}

#[cfg(test)]
fn checkpoint(phase: &'static str) -> Result<(), CoordError> {
    let kill = KILL_AFTER.with(|value| {
        let requested = value.get() == Some(phase);
        if requested {
            value.set(None);
        }
        requested
    });
    if kill {
        nix::sys::signal::raise(nix::sys::signal::Signal::SIGKILL)
            .map_err(|error| CoordError::io(error.into()))?;
    }
    CRASH_AFTER.with(|value| {
        if value.get() == Some(phase) {
            value.set(None);
            Err(CoordError::new("COORD_TEST_CRASH", phase))
        } else {
            Ok(())
        }
    })
}

#[cfg(not(test))]
fn checkpoint(_phase: &'static str) -> Result<(), CoordError> {
    Ok(())
}

#[cfg(test)]
fn inject_if_requested(phase: &'static str, directory: &File) -> Result<(), CoordError> {
    INSERT_AFTER.with(|value| {
        if value.get() != Some(phase) {
            return Ok(());
        }
        value.set(None);
        fchmod(directory, Mode::RWXU)
            .map_err(|error| os_error("cannot inject fence mutation", error))?;
        write_new_at(directory, "intruder", b"x", 0o600)?;
        fchmod(directory, Mode::empty())
            .map_err(|error| os_error("cannot reseal injected fence mutation", error))?;
        directory.sync_all().map_err(CoordError::io)
    })
}

#[cfg(not(test))]
fn inject_if_requested(_phase: &'static str, _directory: &File) -> Result<(), CoordError> {
    Ok(())
}

pub(super) fn preflight(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
) -> Result<(), CoordError> {
    match legacy_kind(&lock.directory)? {
        LegacyKind::Absent => Ok(()),
        LegacyKind::Source => Err(CoordError::new(
            "COORD_RECOVERY_REQUIRED",
            "legacy events.jsonl requires explicit recovery before Genesis",
        )),
        LegacyKind::Tombstone => {
            require_published_topology(lock, generation_id)?;
            validate_published(lock, generation_id, initialization_intent, true)
        }
    }
}

pub(super) fn ensure(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
) -> Result<(), CoordError> {
    validate_name(generation_id)?;
    match legacy_kind(&lock.directory)? {
        LegacyKind::Tombstone => {
            require_published_topology(lock, generation_id)?;
            ensure_publication_observation(lock, generation_id, initialization_intent)?;
            return validate(lock, generation_id, initialization_intent);
        }
        LegacyKind::Source => {
            return Err(CoordError::new(
                "COORD_RECOVERY_REQUIRED",
                "legacy events.jsonl requires explicit recovery before Genesis",
            ));
        }
        LegacyKind::Absent => {}
    }
    let sibling_name = sibling_name(generation_id);
    let sibling = match child_directory_mode(&lock.directory, &sibling_name)? {
        None => {
            reject_evidence_stages(&lock.directory)?;
            if child_exists(&lock.directory, CREATION_OBSERVATION)?
                || child_exists(&lock.directory, FENCE_INTENT)?
                || child_exists(&lock.directory, SEAL_OBSERVATION)?
                || child_exists(&lock.directory, PUBLICATION_PLAN)?
                || child_exists(&lock.directory, PUBLICATION_OBSERVATION)?
            {
                return Err(unknown(
                    "Genesis fence evidence exists without its retained sibling",
                ));
            }
            let creation_plan =
                ensure_creation_plan(lock, generation_id, initialization_intent, &sibling_name)?;
            checkpoint("creation_plan_published")?;
            mkdirat(&lock.directory, sibling_name.as_str(), Mode::RWXU)
                .map_err(|error| os_error("cannot create Genesis fence sibling", error))?;
            lock.directory.sync_all().map_err(CoordError::io)?;
            checkpoint("sibling_created")?;
            let sibling = open_dir_at(&lock.directory, &sibling_name, DIR_MODE)?;
            ensure_creation_observation(
                lock,
                generation_id,
                &sibling_name,
                &creation_plan,
                &sibling,
            )?;
            checkpoint("creation_observation_published")?;
            sibling
        }
        Some(0o700) => {
            if child_exists(&lock.directory, SEAL_OBSERVATION)?
                || child_exists(&lock.directory, PUBLICATION_PLAN)?
                || child_exists(&lock.directory, PUBLICATION_OBSERVATION)?
            {
                return Err(unknown(
                    "unsealed Genesis fence has impossible durable observations",
                ));
            }
            let creation_plan =
                require_creation_plan(lock, generation_id, initialization_intent, &sibling_name)?;
            let sibling = open_dir_at(&lock.directory, &sibling_name, DIR_MODE)?;
            ensure_creation_observation(
                lock,
                generation_id,
                &sibling_name,
                &creation_plan,
                &sibling,
            )?;
            sibling
        }
        Some(0) => {
            if child_exists(&lock.directory, PUBLICATION_OBSERVATION)? {
                return Err(unknown(
                    "unpublished Genesis fence has a publication observation",
                ));
            }
            ensure_seal_observation(lock, generation_id, initialization_intent, &sibling_name)?;
            ensure_publication_plan(lock, generation_id, initialization_intent, &sibling_name)?;
            return publish_sibling(
                lock,
                generation_id,
                initialization_intent,
                &sibling_name,
                None,
            );
        }
        Some(_) => return Err(unknown("Genesis fence sibling has an invalid mode")),
    };
    let sibling_identity = identity(&sibling)?;
    double_empty_dir_at(&lock.directory, &sibling_name, sibling_identity, DIR_MODE)?;
    let empty_inventory_blake3 = capture_empty_inventory(&sibling)?;
    publish_seal_plan(
        lock,
        generation_id,
        initialization_intent,
        &sibling_name,
        &sibling,
        &empty_inventory_blake3,
    )?;
    checkpoint("fence_intent_published")?;
    double_empty_dir_at(&lock.directory, &sibling_name, sibling_identity, DIR_MODE)?;
    if capture_empty_inventory(&sibling)? != empty_inventory_blake3 {
        return Err(changed("Genesis fence inventory changed before seal"));
    }
    revalidate_dir_at(&lock.directory, &sibling_name, sibling_identity, 0o700)?;
    fchmod(&sibling, Mode::empty())
        .map_err(|error| os_error("cannot preseal Genesis fence sibling", error))?;
    sibling.sync_all().map_err(CoordError::io)?;
    lock.directory.sync_all().map_err(CoordError::io)?;
    // The same-UID boundary is cooperative: another owner process can chmod
    // either mode 000 or 0400. We therefore fail closed on every observable
    // identity, mode, inventory, or evidence change; an insert/remove race that
    // restores all observable state requires stronger OS containment.
    if identity(&sibling)? != sibling_identity
        || capture_empty_inventory(&sibling)? != empty_inventory_blake3
    {
        return Err(changed(
            "Genesis fence changed across its irreversible seal",
        ));
    }
    revalidate_dir_at(&lock.directory, &sibling_name, sibling_identity, 0)?;
    checkpoint("fence_sealed")?;
    inject_if_requested("fence_sealed", &sibling)?;
    let seal_inventory_blake3 = capture_empty_inventory(&sibling)?;
    if seal_inventory_blake3 != empty_inventory_blake3 {
        return Err(changed("Genesis fence changed before seal evidence"));
    }
    ensure_seal_observation(lock, generation_id, initialization_intent, &sibling_name)?;
    checkpoint("seal_observation_published")?;
    inject_if_requested("seal_observation_published", &sibling)?;
    if capture_empty_inventory(&sibling)? != seal_inventory_blake3 {
        return Err(changed("Genesis fence changed after seal evidence"));
    }
    validate_sealed_at(lock, generation_id, initialization_intent, &sibling_name)?;
    ensure_publication_plan(lock, generation_id, initialization_intent, &sibling_name)?;
    checkpoint("publication_plan_published")?;
    publish_sibling(
        lock,
        generation_id,
        initialization_intent,
        &sibling_name,
        Some(&sibling),
    )
}

pub(super) fn validate(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
) -> Result<(), CoordError> {
    if legacy_kind(&lock.directory)? != LegacyKind::Tombstone {
        return Err(unknown("published Genesis fence is absent or malformed"));
    }
    require_published_topology(lock, generation_id)?;
    validate_published(lock, generation_id, initialization_intent, false)
}

fn publish_sibling(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
    sibling_name: &str,
    readable: Option<&File>,
) -> Result<(), CoordError> {
    let retained = open_sealed_dir_at(&lock.directory, sibling_name)?;
    let expected = identity(&retained)?;
    revalidate_dir_at(&lock.directory, sibling_name, expected, 0)?;
    let intent_bytes = read_named(&lock.directory, FENCE_INTENT)?;
    let intent: FencePublishIntent = decode(&intent_bytes)?;
    require_publication_plan(lock, generation_id, sibling_name)?;
    let inventory_blake3 = if let Some(directory) = readable {
        if identity(directory)? != expected {
            return Err(changed("retained readable fence identity changed"));
        }
        capture_empty_inventory(directory)?
    } else {
        intent.empty_inventory_blake3.clone()
    };
    if inventory_blake3 != intent.empty_inventory_blake3 {
        return Err(changed(
            "Genesis fence inventory changed before publication",
        ));
    }
    rename_noreplace(&lock.directory, sibling_name, AUTHORITY).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            unknown("Genesis fence publication raced another authority subject")
        } else {
            CoordError::io(error)
        }
    })?;
    checkpoint("fence_renamed")?;
    lock.directory.sync_all().map_err(CoordError::io)?;
    checkpoint("fence_parent_synced")?;
    revalidate_dir_at(&lock.directory, AUTHORITY, expected, 0)?;
    if child_directory_mode(&lock.directory, sibling_name)?.is_some() {
        return Err(changed("Genesis fence sibling remained after publication"));
    }
    if let Some(directory) = readable
        && (identity(directory)? != expected
            || capture_empty_inventory(directory)? != inventory_blake3)
    {
        return Err(changed(
            "Genesis fence changed across same-parent publication",
        ));
    }
    ensure_publication_observation(lock, generation_id, initialization_intent)?;
    checkpoint("publication_observation_published")?;
    validate(lock, generation_id, initialization_intent)
}

fn require_published_topology(lock: &CoordLock, generation_id: &str) -> Result<(), CoordError> {
    let sibling_name = sibling_name(generation_id);
    if child_directory_mode(&lock.directory, &sibling_name)?.is_some() {
        return Err(unknown(
            "published Genesis fence retains a deterministic sibling",
        ));
    }
    reject_evidence_stages(&lock.directory)
}

fn capture_empty_inventory(directory: &File) -> Result<String, CoordError> {
    inventory_empty_dir(directory)?;
    empty_inventory_digest()
}

fn reject_evidence_stages(parent: &File) -> Result<(), CoordError> {
    let prefixes = [
        ".events.jsonl.genesis-next-".to_owned(),
        format!(".{CREATION_PLAN}.next-"),
        format!(".{CREATION_OBSERVATION}.next-"),
        format!(".{FENCE_INTENT}.next-"),
        format!(".{SEAL_OBSERVATION}.next-"),
        format!(".{PUBLICATION_PLAN}.next-"),
        format!(".{PUBLICATION_OBSERVATION}.next-"),
    ];
    let mut entries = rustix::fs::Dir::read_from(parent)
        .map_err(|error| os_error("cannot inventory Genesis fence evidence", error))?;
    while let Some(entry) = entries.read() {
        let entry =
            entry.map_err(|error| os_error("cannot inventory Genesis fence evidence", error))?;
        let name = entry.file_name().to_bytes();
        if prefixes
            .iter()
            .any(|prefix| name.starts_with(prefix.as_bytes()))
        {
            return Err(unknown(
                "Genesis fence retains an unpublished evidence stage",
            ));
        }
    }
    Ok(())
}

fn sibling_name(generation_id: &str) -> String {
    format!(".events.jsonl.genesis-next-{generation_id}")
}

fn unknown(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_FENCE_UNKNOWN", reason)
}
