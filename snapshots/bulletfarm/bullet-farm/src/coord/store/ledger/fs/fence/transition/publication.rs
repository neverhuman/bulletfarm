use super::*;

pub(in crate::coord::store::ledger::fs::fence) fn ensure_publication_plan(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
    sibling_name: &str,
) -> Result<Vec<u8>, CoordError> {
    validate_sealed_at(lock, generation_id, initialization_intent, sibling_name)?;
    let expected = publication_plan(lock, generation_id, sibling_name)?;
    publish_exact_if_missing(&lock.directory, PUBLICATION_PLAN, &expected)?;
    Ok(expected)
}

pub(in crate::coord::store::ledger::fs::fence) fn require_publication_plan(
    lock: &CoordLock,
    generation_id: &str,
    sibling_name: &str,
) -> Result<Vec<u8>, CoordError> {
    let expected = publication_plan(lock, generation_id, sibling_name)?;
    exact_named(&lock.directory, PUBLICATION_PLAN, &expected)
        .map_err(|_| unknown("Genesis fence publication plan is missing or inexact"))?;
    Ok(expected)
}

pub(in crate::coord::store::ledger::fs::fence) fn validate_published(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
    allow_missing_observation: bool,
) -> Result<(), CoordError> {
    validate_publication_predecessors(lock, generation_id, initialization_intent)?;
    let sibling_name = sibling_name(generation_id);
    let publication_plan = require_publication_plan(lock, generation_id, &sibling_name)?;
    validate_publication_topology(lock, generation_id, &publication_plan)?;
    let expected = publication_observation(lock, generation_id, &publication_plan)?;
    if allow_missing_observation && !child_exists(&lock.directory, PUBLICATION_OBSERVATION)? {
        return Ok(());
    }
    exact_named(&lock.directory, PUBLICATION_OBSERVATION, &expected)
        .map_err(|_| unknown("Genesis fence publication observation is missing or inexact"))
}

pub(in crate::coord::store::ledger::fs::fence) fn ensure_publication_observation(
    lock: &CoordLock,
    generation_id: &str,
    initialization_intent: &[u8],
) -> Result<(), CoordError> {
    validate_publication_predecessors(lock, generation_id, initialization_intent)?;
    let sibling_name = sibling_name(generation_id);
    let publication_plan = require_publication_plan(lock, generation_id, &sibling_name)?;
    validate_publication_topology(lock, generation_id, &publication_plan)?;
    let expected = publication_observation(lock, generation_id, &publication_plan)?;
    publish_exact_if_missing(&lock.directory, PUBLICATION_OBSERVATION, &expected)
}

fn publication_plan(
    lock: &CoordLock,
    generation_id: &str,
    sibling_name: &str,
) -> Result<Vec<u8>, CoordError> {
    let intent_bytes = read_named(&lock.directory, FENCE_INTENT)?;
    let intent: FencePublishIntent = decode(&intent_bytes)?;
    let seal_observation = read_named(&lock.directory, SEAL_OBSERVATION)?;
    let observation: FenceObservation = decode(&seal_observation)?;
    if observation.phase != "SEALED_SIBLING"
        || observation.observed_name != sibling_name
        || (observation.device, observation.inode) != (intent.device, intent.inode)
    {
        return Err(unknown(
            "Genesis fence publication plan has invalid seal history",
        ));
    }
    canonical(&FencePublicationPlan {
        kind: "coord_genesis_fence_publication_plan_v2".to_owned(),
        schema_version: 2,
        generation_id: generation_id.to_owned(),
        fence_intent_blake3: digest(
            "bullet.coord.genesis-fence-publish-intent.v2",
            &intent_bytes,
        )?,
        seal_observation_blake3: digest(
            "bullet.coord.genesis-fence-seal-observation.v2",
            &seal_observation,
        )?,
        empty_inventory_blake3: intent.empty_inventory_blake3,
        sibling_name: sibling_name.to_owned(),
        authority_name: AUTHORITY.to_owned(),
        device: intent.device,
        inode: intent.inode,
        owner: intent.owner,
        file_type: intent.file_type,
        mode: 0,
        links: intent.links,
        size: intent.size,
    })
}

fn validate_publication_topology(
    lock: &CoordLock,
    generation_id: &str,
    publication_plan: &[u8],
) -> Result<(), CoordError> {
    let plan: FencePublicationPlan = decode(publication_plan)?;
    if plan.kind != "coord_genesis_fence_publication_plan_v2"
        || plan.schema_version != 2
        || plan.generation_id != generation_id
        || plan.sibling_name != sibling_name(generation_id)
        || plan.authority_name != AUTHORITY
        || plan.owner != owner()
        || plan.file_type != "directory"
        || plan.mode != 0
        || plan.links != 2
    {
        return Err(unknown("Genesis fence publication plan is invalid"));
    }
    let authority = open_sealed_dir_at(&lock.directory, AUTHORITY)?;
    let metadata = fstat(&authority)
        .map_err(|error| os_error("cannot inspect published Genesis fence", error))?;
    if (
        metadata.st_dev,
        metadata.st_ino,
        metadata.st_uid,
        metadata.st_nlink,
    ) != (plan.device, plan.inode, plan.owner, plan.links)
        || u64::try_from(metadata.st_size).ok() != Some(plan.size)
    {
        return Err(unknown(
            "published Genesis fence differs from its publication plan",
        ));
    }
    Ok(())
}

fn publication_observation(
    lock: &CoordLock,
    generation_id: &str,
    publication_plan: &[u8],
) -> Result<Vec<u8>, CoordError> {
    let plan: FencePublicationPlan = decode(publication_plan)?;
    let intent_bytes = read_named(&lock.directory, FENCE_INTENT)?;
    observation_at(
        &lock.directory,
        AUTHORITY,
        generation_id,
        &intent_bytes,
        Identity(plan.device, plan.inode),
        "PUBLISHED_AUTHORITY",
        &plan.empty_inventory_blake3,
    )
}
