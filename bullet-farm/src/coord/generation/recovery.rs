use std::fs::File;

use super::manifest::{GenerationManifest, Sha256Digest};
use crate::coord::{CoordError, model::Record};

#[path = "recovery/api.rs"]
mod api;
pub(crate) use api::{
    ContentExpectation, RecoveryInput, RecoveryOutcome, RecoveryState, SourceExpectation,
    recover_rollover, verify_recovery_in_progress,
};

#[derive(Clone, Debug)]
pub(crate) struct BaselineIdentity {
    pub(crate) record: Record,
    pub(crate) genesis_digest: String,
    pub(crate) request_id: String,
    pub(crate) request_digest: String,
}

pub(crate) fn verify_retained_artifacts(
    trusted: &mut File,
    interrupted: &mut File,
    tainted: &mut File,
    frozen: &mut File,
    manifest: &GenerationManifest,
) -> Result<Vec<Record>, CoordError> {
    #[cfg(target_os = "linux")]
    {
        verifier::verify_retained_artifacts(trusted, interrupted, tainted, frozen, manifest)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (trusted, interrupted, tainted, frozen, manifest);
        Err(CoordError::new(
            "COORD_RECOVERY_PLATFORM_UNSUPPORTED",
            "recovery corpus verification is implemented only on Linux",
        ))
    }
}

pub(crate) fn baseline_identity(
    manifest: &GenerationManifest,
) -> Result<BaselineIdentity, CoordError> {
    #[cfg(target_os = "linux")]
    {
        let record = authority::baseline_record(manifest)?;
        let subject = authority::baseline_subject(manifest, &record)?;
        Ok(BaselineIdentity {
            record,
            genesis_digest: subject.genesis_digest,
            request_id: subject.request_id,
            request_digest: subject.request_digest,
        })
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = manifest;
        Err(CoordError::new(
            "COORD_RECOVERY_PLATFORM_UNSUPPORTED",
            "recovery baseline identity is implemented only on Linux",
        ))
    }
}

#[cfg(target_os = "linux")]
#[path = "recovery/fs.rs"]
mod platform_fs;

#[cfg(target_os = "linux")]
#[path = "recovery/authority.rs"]
mod authority;

#[cfg(target_os = "linux")]
#[path = "recovery/verify.rs"]
mod verifier;

#[cfg(target_os = "linux")]
#[path = "recovery/projection.rs"]
pub(in crate::coord) mod projection;

#[cfg(target_os = "linux")]
#[path = "recovery/exchange.rs"]
mod exchange;

#[cfg(target_os = "linux")]
#[path = "recovery/tree.rs"]
mod tree;

#[cfg(target_os = "linux")]
#[path = "recovery/finalize.rs"]
mod finalize;

#[cfg(target_os = "linux")]
#[path = "recovery/transition.rs"]
mod transition;

#[cfg(target_os = "linux")]
#[path = "recovery/published.rs"]
mod published;

#[path = "recovery/published_api.rs"]
mod published_api;
pub(crate) use published_api::{
    PublishedRecoveryGuard, reverify as reverify_published_recovery,
    verify as verify_published_recovery,
};

#[cfg(target_os = "linux")]
#[path = "recovery/support.rs"]
mod support;

#[cfg(target_os = "linux")]
mod linux {
    use super::{
        RecoveryInput, RecoveryOutcome, RecoveryState, authority, exchange, finalize,
        platform_fs as io,
        support::{expectation, invalid, outcome, publish_current},
        transition, tree, verifier,
    };
    use crate::coord::{CoordError, generation::manifest::GenerationManifest};
    const RETIRED: &str = "retired-v1.non-authoritative";

    #[cfg(test)]
    pub(super) use transition::{Crash as TransitionCrash, test_crash_at};

    pub(super) fn recover(
        input: &RecoveryInput,
        manifest: &GenerationManifest,
        revalidate_authority: impl FnMut() -> Result<(), CoordError>,
    ) -> Result<RecoveryOutcome, CoordError> {
        recover_with_probes(
            input,
            manifest,
            revalidate_authority,
            verifier::has_other_writable_fd,
            || Ok(()),
        )
    }

    pub(super) fn verify_in_progress(
        input: &RecoveryInput,
        manifest: &GenerationManifest,
    ) -> Result<bool, CoordError> {
        let preflight = verifier::creation_free_preflight(input, manifest)?;
        Ok(preflight.location != exchange::LegacyLocation::Fresh)
    }

    #[cfg(test)]
    pub(in crate::coord) fn recover_with_writer_probe(
        input: &RecoveryInput,
        manifest: &GenerationManifest,
        writer_probe: impl FnMut((u64, u64)) -> Result<bool, CoordError>,
    ) -> Result<RecoveryOutcome, CoordError> {
        recover_with_probes(input, manifest, || Ok(()), writer_probe, || Ok(()))
    }

    #[cfg(test)]
    pub(super) fn recover_with_post_publish_probe(
        input: &RecoveryInput,
        manifest: &GenerationManifest,
        writer_probe: impl FnMut((u64, u64)) -> Result<bool, CoordError>,
        post_publish_probe: impl FnMut() -> Result<(), CoordError>,
    ) -> Result<RecoveryOutcome, CoordError> {
        recover_with_probes(input, manifest, || Ok(()), writer_probe, post_publish_probe)
    }

    fn recover_with_probes(
        input: &RecoveryInput,
        manifest: &GenerationManifest,
        revalidate_authority: impl FnMut() -> Result<(), CoordError>,
        mut writer_probe: impl FnMut((u64, u64)) -> Result<bool, CoordError>,
        mut post_publish_probe: impl FnMut() -> Result<(), CoordError>,
    ) -> Result<RecoveryOutcome, CoordError> {
        manifest.validate()?;
        let baseline_record = authority::baseline_record(manifest)?;
        let baseline = authority::baseline_subject(manifest, &baseline_record)?;
        let mut preflight = verifier::creation_free_preflight(input, manifest)?;
        let legacy_lease = verifier::LegacyReadLease::acquire(&preflight.legacy)?;
        let authority =
            authority::Authority::acquire_authorized(&input.coord_dir, revalidate_authority)?;
        let owner = authority.owner();
        legacy_lease.revalidate()?;
        verifier::revalidate_preflight(&mut preflight, input, manifest)?;
        let verifier::Preflight {
            mut interrupted,
            mut tainted,
            mut legacy,
            location,
            sibling_state,
            source_identity,
        } = preflight;
        let resumed = location != exchange::LegacyLocation::Fresh
            || sibling_state == exchange::SiblingState::Sealed;

        let id = manifest.generation_id().as_str().to_owned();
        let recovery_dir = input.coord_dir.join("recovery").join(&id);
        let retired = recovery_dir.join(RETIRED);
        let sibling_name = exchange::sibling_name(&id)?;
        let current_matches = tree::current_is(authority.root(), owner, manifest)?;
        if current_matches {
            if location != exchange::LegacyLocation::Retired {
                return Err(invalid(
                    "CURRENT exists without the retired exact legacy source",
                ));
            }
            let layout = tree::Layout::open(authority.root(), &id, owner)?;
            let recovery_dir_fd = layout.recovery();
            let tombstone_identity = authority.tombstone_identity()?;
            let intent_sha256 = authority::write_or_verify_intent(
                recovery_dir_fd,
                manifest,
                &expectation(&manifest.body.recovery()?.artifacts.frozen_live_source),
                source_identity,
                tombstone_identity,
                &baseline,
                false,
            )?;
            let retained_tombstone = authority.retain_tombstone(tombstone_identity)?;
            let prepared_observation_sha256 = exchange::verify_prepared_observation(
                recovery_dir_fd,
                manifest,
                &baseline,
                &intent_sha256,
                &sibling_name,
                &retained_tombstone,
                owner,
            )?;
            let tombstone_observation_sha256 = authority.write_or_verify_tombstone_observation(
                recovery_dir_fd,
                manifest,
                &baseline,
                &intent_sha256,
                &prepared_observation_sha256,
                &retained_tombstone,
                false,
            )?;
            exchange::write_or_verify_retirement_observation(
                authority.root(),
                recovery_dir_fd,
                manifest,
                &baseline,
                &intent_sha256,
                &prepared_observation_sha256,
                &tombstone_observation_sha256,
                &sibling_name,
                &retained_tombstone,
                &legacy,
                owner,
                false,
            )?;
            layout.verify_generation(manifest, &baseline_record, &baseline)?;
            finalize::revalidate(
                &authority,
                &layout,
                input,
                manifest,
                &baseline_record,
                &baseline,
                &sibling_name,
                &retained_tombstone,
                &mut legacy,
                &legacy_lease,
            )?;
            authority.root().sync_all().map_err(CoordError::io)?;
            return Ok(outcome(RecoveryState::AlreadyCurrent, id, retired));
        }

        let layout = tree::Layout::ensure(authority.root(), &id, owner)?;
        let recovery_dir_fd = layout.recovery();
        let final_exists = layout.generation_exists()?;

        if final_exists {
            layout.verify_generation(manifest, &baseline_record, &baseline)?;
        } else if resumed {
            return Err(invalid(
                "retired legacy source exists without a complete immutable generation",
            ));
        } else {
            layout.build_generation(
                &mut interrupted,
                &mut tainted,
                &mut legacy,
                manifest,
                &baseline_record,
                &baseline,
            )?;
        }
        let prepared_tombstone = if location == exchange::LegacyLocation::Fresh {
            Some(exchange::prepare(
                authority.root(),
                &sibling_name,
                owner,
                sibling_state,
            )?)
        } else {
            None
        };
        let tombstone_identity = match prepared_tombstone.as_ref() {
            Some(tombstone) => verifier::identity(tombstone.retained())?,
            None => authority.tombstone_identity()?,
        };
        let intent_sha256 = authority::write_or_verify_intent(
            recovery_dir_fd,
            manifest,
            &expectation(&manifest.body.recovery()?.artifacts.frozen_live_source),
            source_identity,
            tombstone_identity,
            &baseline,
            location == exchange::LegacyLocation::Fresh
                && sibling_state != exchange::SiblingState::Sealed,
        )?;
        let prepared_observation_sha256 = if let Some(tombstone) = prepared_tombstone.as_ref() {
            exchange::seal(authority.root(), &sibling_name, tombstone, owner)?;
            transition::injected(transition::Crash::Seal)?;
            let digest = exchange::write_or_verify_prepared_observation(
                recovery_dir_fd,
                manifest,
                &baseline,
                &intent_sha256,
                &sibling_name,
                tombstone,
                owner,
                true,
            )?;
            io::verify_open_file(&mut legacy, &input.frozen_live_source.content)?;
            exchange::exchange(authority.root(), &sibling_name, tombstone, &legacy, owner)?;
            transition::injected(transition::Crash::Exchange)?;
            digest
        } else {
            let retained = authority.retain_tombstone(tombstone_identity)?;
            exchange::verify_prepared_observation(
                recovery_dir_fd,
                manifest,
                &baseline,
                &intent_sha256,
                &sibling_name,
                &retained,
                owner,
            )?
        };
        let retained_tombstone = match prepared_tombstone.as_ref() {
            Some(tombstone) => tombstone.retained().try_clone().map_err(CoordError::io)?,
            None => authority.retain_tombstone(tombstone_identity)?,
        };
        let tombstone_observation_sha256 = authority.write_or_verify_tombstone_observation(
            recovery_dir_fd,
            manifest,
            &baseline,
            &intent_sha256,
            &prepared_observation_sha256,
            &retained_tombstone,
            true,
        )?;
        if location != exchange::LegacyLocation::Retired {
            exchange::retire(
                authority.root(),
                recovery_dir_fd,
                &sibling_name,
                &legacy,
                owner,
            )?;
            transition::injected(transition::Crash::Retire)?;
        }
        exchange::write_or_verify_retirement_observation(
            authority.root(),
            recovery_dir_fd,
            manifest,
            &baseline,
            &intent_sha256,
            &prepared_observation_sha256,
            &tombstone_observation_sha256,
            &sibling_name,
            &retained_tombstone,
            &legacy,
            owner,
            true,
        )?;
        if writer_probe(source_identity)? {
            return Ok(outcome(
                RecoveryState::FrozenWaitingForLegacyWriters,
                id,
                retired,
            ));
        }
        io::verify_open_file(&mut legacy, &input.frozen_live_source.content)?;
        if writer_probe(source_identity)? {
            return Ok(outcome(
                RecoveryState::FrozenWaitingForLegacyWriters,
                id,
                retired,
            ));
        }
        io::verify_open_file(&mut legacy, &input.frozen_live_source.content)?;
        legacy_lease.revalidate()?;

        finalize::revalidate(
            &authority,
            &layout,
            input,
            manifest,
            &baseline_record,
            &baseline,
            &sibling_name,
            &retained_tombstone,
            &mut legacy,
            &legacy_lease,
        )?;
        publish_current(&authority, &input.coord_dir, manifest)?;
        post_publish_probe()?;
        finalize::revalidate(
            &authority,
            &layout,
            input,
            manifest,
            &baseline_record,
            &baseline,
            &sibling_name,
            &retained_tombstone,
            &mut legacy,
            &legacy_lease,
        )?;
        if !tree::current_is(authority.root(), owner, manifest)? {
            return Err(invalid("CURRENT changed after final subject revalidation"));
        }
        Ok(outcome(
            if resumed {
                RecoveryState::ResumedAndPublished
            } else {
                RecoveryState::Published
            },
            id,
            retired,
        ))
    }
}

#[cfg(all(test, target_os = "linux"))]
pub(in crate::coord) use linux::recover_with_writer_probe;

#[cfg(all(test, target_os = "linux"))]
pub(in crate::coord) fn test_crash_at_exchange() {
    linux::test_crash_at(linux::TransitionCrash::Exchange);
}

#[cfg(all(test, target_os = "linux"))]
#[path = "recovery/tests.rs"]
mod tests;

#[cfg(all(test, target_os = "linux"))]
pub(in crate::coord) use tests::adoption_fixture;
