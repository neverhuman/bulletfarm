use std::fs::File;

use rustix::fs::{FileType, fstat};

use super::{evidence::*, *};
mod publication;
pub(super) use publication::*;

pub(super) fn ensure_creation_plan(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
    sibling_name: &str,
) -> Result<Vec<u8>, CoordError> {
    let expected = creation_plan(lock, generation_id, initialization_intent, sibling_name)?;
    publish_exact_if_missing(&lock.directory, CREATION_PLAN, &expected)?;
    Ok(expected)
}

pub(super) fn require_creation_plan(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
    sibling_name: &str,
) -> Result<Vec<u8>, CoordError> {
    let expected = creation_plan(lock, generation_id, initialization_intent, sibling_name)?;
    exact_named(&lock.directory, CREATION_PLAN, &expected)
        .map_err(|_| unknown("Genesis fence creation plan is missing or inexact"))?;
    Ok(expected)
}

pub(super) fn ensure_creation_observation(
    lock: &CoordLock,
    generation_id: &str,
    sibling_name: &str,
    creation_plan: &[u8],
    sibling: &File,
) -> Result<Vec<u8>, CoordError> {
    let expected = creation_observation_at(
        &lock.directory,
        sibling_name,
        generation_id,
        creation_plan,
        identity(sibling)?,
    )?;
    publish_exact_if_missing(&lock.directory, CREATION_OBSERVATION, &expected)?;
    Ok(expected)
}

pub(super) fn publish_seal_plan(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
    sibling_name: &str,
    sibling: &File,
    empty_inventory_blake3: &str,
) -> Result<Vec<u8>, CoordError> {
    let creation_plan =
        require_creation_plan(lock, generation_id, initialization_intent, sibling_name)?;
    let creation_observation =
        require_creation_observation(lock, generation_id, sibling_name, &creation_plan)?;
    let metadata =
        fstat(sibling).map_err(|error| os_error("cannot inspect Genesis fence sibling", error))?;
    let intent = FencePublishIntent {
        kind: "coord_genesis_fence_publish_intent_v2".to_owned(),
        schema_version: 2,
        generation_id: generation_id.to_owned(),
        initialization_intent_blake3: digest(
            "bullet.coord.genesis-initialization-intent.v2",
            initialization_intent,
        )?,
        creation_observation_blake3: digest(
            "bullet.coord.genesis-fence-create-observation.v2",
            &creation_observation,
        )?,
        empty_inventory_blake3: empty_inventory_blake3.to_owned(),
        sibling_name: sibling_name.to_owned(),
        device: metadata.st_dev,
        inode: metadata.st_ino,
        owner: metadata.st_uid,
        file_type: "directory".to_owned(),
        mode: metadata.st_mode & 0o7777,
        links: metadata.st_nlink,
        size: metadata
            .st_size
            .try_into()
            .map_err(|_| unknown("negative fence size"))?,
        ctime_seconds: metadata.st_ctime,
        ctime_nanoseconds: metadata.st_ctime_nsec,
    };
    validate_preseal(
        &intent,
        generation_id,
        initialization_intent,
        sibling_name,
        &creation_observation,
    )?;
    validate_creation_matches_seal_plan(&creation_observation, &intent)?;
    let bytes = canonical(&intent)?;
    publish_exact_if_missing(&lock.directory, FENCE_INTENT, &bytes)?;
    Ok(bytes)
}

pub(super) fn ensure_seal_observation(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
    name: &str,
) -> Result<Vec<u8>, CoordError> {
    let (_, _, expected) = sealed_state(lock, generation_id, initialization_intent, name)?;
    publish_exact_if_missing(&lock.directory, SEAL_OBSERVATION, &expected)?;
    Ok(expected)
}

pub(super) fn validate_sealed_at(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
    name: &str,
) -> Result<(), CoordError> {
    let (_, _, expected) = sealed_state(lock, generation_id, initialization_intent, name)?;
    exact_named(&lock.directory, SEAL_OBSERVATION, &expected)
        .map_err(|_| unknown("Genesis fence seal observation is missing or inexact"))
}

pub(in crate::coord::store::ledger::fs::fence) fn validate_publication_predecessors(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
) -> Result<(), CoordError> {
    let sibling_name = sibling_name(generation_id);
    let creation_plan =
        require_creation_plan(lock, generation_id, initialization_intent, &sibling_name)?;
    let creation_observation =
        require_creation_observation(lock, generation_id, &sibling_name, &creation_plan)?;
    let intent_bytes = read_named(&lock.directory, FENCE_INTENT)?;
    let intent: FencePublishIntent = decode(&intent_bytes)?;
    validate_preseal(
        &intent,
        generation_id,
        initialization_intent,
        &sibling_name,
        &creation_observation,
    )?;
    validate_creation_matches_seal_plan(&creation_observation, &intent)?;
    validate_seal_history(&lock.directory, generation_id, &intent_bytes, &intent)?;
    let authority = open_sealed_dir_at(&lock.directory, AUTHORITY)?;
    let metadata = fstat(&authority)
        .map_err(|error| os_error("cannot inspect published Genesis fence", error))?;
    if identity(&authority)? != Identity(intent.device, intent.inode)
        || metadata.st_uid != intent.owner
        || metadata.st_nlink != intent.links
        || u64::try_from(metadata.st_size).ok() != Some(intent.size)
    {
        return Err(unknown(
            "published Genesis fence differs from its immutable predecessor chain",
        ));
    }
    Ok(())
}

fn creation_plan(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
    sibling_name: &str,
) -> Result<Vec<u8>, CoordError> {
    let parent = fstat(&lock.directory)
        .map_err(|error| os_error("cannot inspect Genesis fence parent", error))?;
    canonical(&FenceCreationPlan {
        kind: "coord_genesis_fence_create_plan_v2".to_owned(),
        schema_version: 2,
        generation_id: generation_id.to_owned(),
        initialization_intent_blake3: digest(
            "bullet.coord.genesis-initialization-intent.v2",
            initialization_intent,
        )?,
        sibling_name: sibling_name.to_owned(),
        parent_device: parent.st_dev,
        parent_inode: parent.st_ino,
        parent_owner: parent.st_uid,
        intended_mode: DIR_MODE,
    })
}

fn creation_observation_at(
    parent: &File,
    name: &str,
    generation_id: &str,
    creation_plan: &[u8],
    expected: Identity,
) -> Result<Vec<u8>, CoordError> {
    let directory = open_dir_at(parent, name, DIR_MODE)?;
    if identity(&directory)? != expected {
        return Err(changed("created Genesis fence pathname identity changed"));
    }
    let empty_inventory_blake3 = capture_empty_inventory(&directory)?;
    let metadata = fstat(&directory)
        .map_err(|error| os_error("cannot inspect created Genesis fence", error))?;
    canonical(&FenceCreationObservation {
        kind: "coord_genesis_fence_create_observation_v2".to_owned(),
        schema_version: 2,
        generation_id: generation_id.to_owned(),
        creation_plan_blake3: digest("bullet.coord.genesis-fence-create-plan.v2", creation_plan)?,
        empty_inventory_blake3,
        observed_name: name.to_owned(),
        device: metadata.st_dev,
        inode: metadata.st_ino,
        owner: metadata.st_uid,
        file_type: "directory".to_owned(),
        mode: metadata.st_mode & 0o7777,
        links: metadata.st_nlink,
        size: metadata
            .st_size
            .try_into()
            .map_err(|_| unknown("negative fence size"))?,
        ctime_seconds: metadata.st_ctime,
        ctime_nanoseconds: metadata.st_ctime_nsec,
    })
}

fn require_creation_observation(
    lock: &CoordLock,
    generation_id: &str,
    sibling_name: &str,
    creation_plan: &[u8],
) -> Result<Vec<u8>, CoordError> {
    let bytes = read_named(&lock.directory, CREATION_OBSERVATION)?;
    let observation: FenceCreationObservation = decode(&bytes)?;
    if observation.kind != "coord_genesis_fence_create_observation_v2"
        || observation.schema_version != 2
        || observation.generation_id != generation_id
        || observation.creation_plan_blake3
            != digest("bullet.coord.genesis-fence-create-plan.v2", creation_plan)?
        || observation.empty_inventory_blake3 != empty_inventory_digest()?
        || observation.observed_name != sibling_name
        || observation.device == 0
        || observation.inode == 0
        || observation.owner != owner()
        || observation.file_type != "directory"
        || observation.mode != DIR_MODE
        || observation.links != 2
    {
        return Err(unknown("Genesis fence creation observation is invalid"));
    }
    Ok(bytes)
}

fn validate_creation_matches_seal_plan(
    creation_observation: &[u8],
    intent: &FencePublishIntent,
) -> Result<(), CoordError> {
    let creation: FenceCreationObservation = decode(creation_observation)?;
    if (
        creation.device,
        creation.inode,
        creation.owner,
        creation.mode,
        creation.links,
        creation.size,
    ) != (
        intent.device,
        intent.inode,
        intent.owner,
        intent.mode,
        intent.links,
        intent.size,
    ) || (creation.ctime_seconds, creation.ctime_nanoseconds)
        != (intent.ctime_seconds, intent.ctime_nanoseconds)
        || creation.empty_inventory_blake3 != intent.empty_inventory_blake3
    {
        return Err(unknown(
            "Genesis fence seal plan does not bind its creation observation",
        ));
    }
    Ok(())
}

fn sealed_state(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
    name: &str,
) -> Result<(Vec<u8>, FencePublishIntent, Vec<u8>), CoordError> {
    let sibling_name = sibling_name(generation_id);
    let creation_plan =
        require_creation_plan(lock, generation_id, initialization_intent, &sibling_name)?;
    let creation_observation =
        require_creation_observation(lock, generation_id, &sibling_name, &creation_plan)?;
    let intent_bytes = read_named(&lock.directory, FENCE_INTENT)?;
    let intent: FencePublishIntent = decode(&intent_bytes)?;
    validate_preseal(
        &intent,
        generation_id,
        initialization_intent,
        &sibling_name,
        &creation_observation,
    )?;
    validate_creation_matches_seal_plan(&creation_observation, &intent)?;
    let sealed = open_sealed_dir_at(&lock.directory, name)?;
    if identity(&sealed)? != Identity(intent.device, intent.inode) {
        return Err(unknown(
            "sealed Genesis fence identity differs from its seal plan",
        ));
    }
    let metadata =
        fstat(&sealed).map_err(|error| os_error("cannot inspect sealed Genesis fence", error))?;
    if metadata.st_uid != intent.owner
        || metadata.st_nlink != intent.links
        || u64::try_from(metadata.st_size).ok() != Some(intent.size)
    {
        return Err(unknown(
            "sealed Genesis fence metadata differs from its seal plan",
        ));
    }
    let expected = observation_at(
        &lock.directory,
        name,
        generation_id,
        &intent_bytes,
        Identity(intent.device, intent.inode),
        "SEALED_SIBLING",
        &intent.empty_inventory_blake3,
    )?;
    Ok((intent_bytes, intent, expected))
}

fn observation_at(
    parent: &File,
    name: &str,
    generation_id: &str,
    fence_intent: &[u8],
    expected: Identity,
    phase: &str,
    empty_inventory_blake3: &str,
) -> Result<Vec<u8>, CoordError> {
    let sealed = open_sealed_dir_at(parent, name)?;
    if identity(&sealed)? != expected {
        return Err(changed("sealed Genesis fence pathname identity changed"));
    }
    let metadata =
        fstat(&sealed).map_err(|error| os_error("cannot inspect sealed Genesis fence", error))?;
    let observation = FenceObservation {
        kind: "coord_genesis_fence_observation_v2".to_owned(),
        schema_version: 2,
        generation_id: generation_id.to_owned(),
        fence_intent_blake3: digest("bullet.coord.genesis-fence-publish-intent.v2", fence_intent)?,
        empty_inventory_blake3: empty_inventory_blake3.to_owned(),
        phase: phase.to_owned(),
        observed_name: name.to_owned(),
        device: metadata.st_dev,
        inode: metadata.st_ino,
        owner: metadata.st_uid,
        file_type: "directory".to_owned(),
        mode: metadata.st_mode & 0o7777,
        links: metadata.st_nlink,
        size: metadata
            .st_size
            .try_into()
            .map_err(|_| unknown("negative fence size"))?,
        ctime_seconds: metadata.st_ctime,
        ctime_nanoseconds: metadata.st_ctime_nsec,
    };
    if !FileType::from_raw_mode(metadata.st_mode).is_dir()
        || observation.owner != owner()
        || observation.mode != 0
        || observation.links != 2
    {
        return Err(unknown("sealed Genesis fence metadata is invalid"));
    }
    canonical(&observation)
}

fn publish_exact_if_missing(parent: &File, name: &str, expected: &[u8]) -> Result<(), CoordError> {
    if child_exists(parent, name)? {
        return exact_named(parent, name, expected)
            .map_err(|_| unknown(format!("Genesis fence evidence {name} is inexact")));
    }
    super::super::publish::publish_file(parent, name, expected, 0o400).map_err(|error| {
        unknown(format!(
            "cannot publish Genesis fence evidence {name}: {error}"
        ))
    })
}
