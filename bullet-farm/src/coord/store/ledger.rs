use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use crate::coord::{
    CoordError,
    generation::{
        manifest::{CurrentPointer, GenerationManifest, GenerationManifestBody},
        segment::{self, AppendRequest, SegmentInspection},
    },
    model::{GENERATION_SCHEMA_VERSION, Record},
};

mod adoption;
mod fs;
mod genesis;
#[cfg(test)]
#[path = "ledger/adoption/tests/git_fixture.rs"]
mod git_fixture_support;
mod recovery_production;
mod replay_authority;
mod transaction;
mod verify;

pub(super) use genesis::GenesisProvenance;

#[cfg(test)]
pub(in crate::coord::store::ledger) use replay_authority::{
    test_mutate_genesis_after_first_validation, test_mutate_recovery_after_first_validation,
    test_rewrite_manifest_before_final_replay, test_swap_subject_before_pending_reconcile,
};

#[cfg(all(test, target_os = "linux"))]
pub(in crate::coord::store::ledger) use transaction::test_swap_subject_before_return;

const COORD_CHILD: &str = ".bullet-family/coord";
const GENERATIONS: &str = "generations";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum GenerationKind {
    Genesis,
    Recovery {
        incident_at_unix_ms: u64,
        recovered_at_unix_ms: u64,
        trusted_records: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct LedgerWatermark {
    pub generation_id: String,
    pub manifest_blake3: String,
    pub kind: GenerationKind,
    pub last_sequence: u64,
    pub next_sequence: u64,
    pub head_envelope_digest: String,
    pub last_record_digest: String,
    pub last_request_id: String,
    pub last_request_digest: String,
    pub byte_length: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RequestReceipt {
    pub generation_id: String,
    pub request_id: String,
    pub sequence: u64,
    pub request_digest: String,
    pub record_digest: String,
    pub envelope_digest: String,
    pub byte_offset: u64,
    pub frame_length: u64,
}

#[derive(Clone, Debug)]
pub(super) struct LedgerView {
    pub records: Vec<Record>,
    pub watermark: LedgerWatermark,
    pub source: PathBuf,
    requests: BTreeMap<String, RequestReceipt>,
}

impl LedgerView {
    pub(super) fn request(&self, request_id: &str) -> Option<&RequestReceipt> {
        self.requests.get(request_id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg(test)]
pub(super) struct AppendOutcome {
    pub receipt: RequestReceipt,
    pub watermark: LedgerWatermark,
}

#[derive(Clone, Debug)]
pub(super) struct RequestTransaction {
    pub existing: bool,
    pub record: Record,
    pub receipt: RequestReceipt,
    pub watermark: LedgerWatermark,
    pub view: LedgerView,
}

pub(super) struct Ledger {
    family_root: PathBuf,
    coord_dir: PathBuf,
}

struct Loaded {
    pointer: CurrentPointer,
    manifest: GenerationManifest,
    genesis_digest: String,
    files: fs::GenerationFiles,
    inspection: SegmentInspection,
    view: LedgerView,
}

impl Ledger {
    pub(super) fn new(family_root: &Path) -> Self {
        Self {
            family_root: family_root.to_owned(),
            coord_dir: family_root.join(COORD_CHILD),
        }
    }

    pub(super) fn status(&self) -> Result<LedgerView, CoordError> {
        let probe = fs::probe(&self.coord_dir)?;
        match probe.presence() {
            fs::Presence::Absent => return Err(uninitialized()),
            fs::Presence::Legacy => return Err(recovery_required()),
            fs::Presence::Retired => return Err(recovery_in_progress()),
            fs::Presence::Current => {}
        }
        let lock = probe.into_lock(&self.coord_dir, false)?;
        Ok(self.load_locked(&lock, None, false)?.view)
    }

    pub(super) fn initialize_genesis<F>(
        &self,
        provenance: &GenesisProvenance,
        clock: F,
    ) -> Result<LedgerView, CoordError>
    where
        F: FnOnce() -> Result<u64, CoordError>,
    {
        let probe = fs::probe(&self.coord_dir)?;
        let observed = probe.presence();
        if observed == fs::Presence::Legacy {
            return Err(recovery_required());
        }
        let lock = if matches!(observed, fs::Presence::Current | fs::Presence::Retired) {
            probe.into_lock(&self.coord_dir, true)?
        } else {
            fs::ensure_layout(&self.family_root, &self.coord_dir)?;
            fs::CoordLock::acquire(&self.coord_dir, true)?
        };
        if let Some(existing) = lock.current()? {
            let loaded = self.load_locked(&lock, Some(&existing), false)?;
            genesis::ensure_manifest_matches(provenance, &loaded.manifest)?;
            return Ok(loaded.view);
        }
        let locked_presence = lock.presence_without_current()?;
        if locked_presence == fs::Presence::Legacy {
            return Err(recovery_required());
        }
        let prepared = match fs::genesis_intent_candidate(&lock)? {
            fs::GenesisIntentCandidate::Published(bytes) => genesis::decode_for_presence(
                &bytes,
                provenance,
                locked_presence == fs::Presence::Retired,
            )?,
            fs::GenesisIntentCandidate::Staged(bytes) => {
                let prepared = genesis::decode_for_presence(
                    &bytes,
                    provenance,
                    locked_presence == fs::Presence::Retired,
                )?;
                fs::publish_genesis_intent(&lock, &bytes)?;
                if fs::published_genesis_intent(&lock)? != bytes {
                    return Err(changed("published Genesis intent read-back differs"));
                }
                prepared
            }
            fs::GenesisIntentCandidate::Absent => {
                if locked_presence == fs::Presence::Retired {
                    return Err(fence_unknown(
                        "sealed Genesis fence exists without a durable initialization intent",
                    ));
                }
                let prepared = genesis::prepare(provenance, clock()?)?;
                fs::publish_genesis_intent(&lock, &prepared.intent_bytes)?;
                if fs::published_genesis_intent(&lock)? != prepared.intent_bytes {
                    return Err(changed("published Genesis intent read-back differs"));
                }
                prepared
            }
        };
        let manifest = &prepared.manifest;
        let pointer = &prepared.current;
        fs::preflight_genesis_fence(
            &lock,
            manifest.generation_id().as_str(),
            &prepared.intent_bytes,
        )?;
        fs::ensure_tombstone(
            &lock,
            manifest.generation_id().as_str(),
            &prepared.intent_bytes,
        )?;
        let genesis_digest = chain_genesis(pointer)?;
        let request_id = genesis_request_id(pointer)?;
        let GenerationManifestBody::Genesis(body) = &manifest.body else {
            return Err(invalid("initialization intent is not GENESIS"));
        };
        let record = Record::GenesisV2 {
            schema_version: GENERATION_SCHEMA_VERSION,
            generation_id: manifest.generation_id().as_str().to_owned(),
            manifest_blake3: pointer.manifest_blake3().to_owned(),
            created_at_unix_ms: body.created_at_unix_ms,
        };
        let request = AppendRequest {
            generation_id: manifest.generation_id().as_str(),
            sequence: 1,
            previous_digest: &genesis_digest,
            request_id: &request_id,
            record: &record,
        };
        segment::validate_append_request(&request, &genesis_digest)?;
        let mut files = fs::create_generation(
            &lock,
            manifest.generation_id().as_str(),
            &manifest.canonical_bytes()?,
        )?;
        segment::append_files(
            &mut files.segment,
            &files.pending,
            &request,
            &genesis_digest,
        )?;
        verify::single_genesis(
            &segment::inspect_files(
                &mut files.segment,
                &files.pending,
                request.generation_id,
                &genesis_digest,
            )?,
            &record,
            &request_id,
        )?;
        files.revalidate(&lock, true)?;
        files = fs::publish_generation(&lock, files)?;
        files.revalidate(&lock, true)?;
        let current_bytes = pointer.canonical_bytes()?;
        fs::publish_current(&lock, &current_bytes)?;
        let published = lock
            .current()?
            .ok_or_else(|| changed("CURRENT disappeared after publication"))?;
        if &published != pointer {
            return Err(changed("CURRENT read-back differs after publication"));
        }
        Ok(self.load_locked(&lock, Some(pointer), false)?.view)
    }

    #[cfg(test)]
    pub(super) fn append(
        &self,
        expected_generation_id: &str,
        request_id: &str,
        record: &Record,
    ) -> Result<AppendOutcome, CoordError> {
        let transaction =
            self.transact(expected_generation_id, request_id, |_| Ok(record.clone()))?;
        if transaction.existing
            && bullet_wire::canonical_json(&transaction.record).map_err(wire)?
                != bullet_wire::canonical_json(record).map_err(wire)?
        {
            return Err(CoordError::new(
                "COORD_REQUEST_CONFLICT",
                "request ID already binds another canonical record subject",
            ));
        }
        Ok(AppendOutcome {
            receipt: transaction.receipt,
            watermark: transaction.watermark,
        })
    }

    fn load_locked(
        &self,
        lock: &fs::CoordLock,
        expected: Option<&CurrentPointer>,
        reconcile: bool,
    ) -> Result<Loaded, CoordError> {
        lock.revalidate()?;
        let pointer = lock
            .current()?
            .ok_or_else(|| changed("CURRENT is absent while the stable LOCK is held"))?;
        if expected.is_some_and(|value| value != &pointer) {
            return Err(changed("CURRENT changed across lock acquisition"));
        }
        let generation_dir = self
            .coord_dir
            .join(GENERATIONS)
            .join(pointer.generation_id().as_str());
        let mut files = lock.generation(pointer.generation_id().as_str(), reconcile)?;
        files.revalidate(lock, reconcile)?;
        let manifest = files.load_manifest(pointer.generation_id())?;
        pointer.verify_manifest(&manifest)?;
        let recovery_guard = match &manifest.body {
            GenerationManifestBody::Genesis(_) => {
                replay_authority::verify_genesis(lock, &manifest, &pointer, || Ok(()))?;
                #[cfg(test)]
                replay_authority::inject_genesis(&self.coord_dir)?;
                None
            }
            GenerationManifestBody::RecoveryBaseline(_) => {
                let guard = crate::coord::generation::recovery::verify_published_recovery(
                    lock.root(),
                    &manifest,
                )?;
                #[cfg(test)]
                replay_authority::inject_recovery(&self.coord_dir, &manifest)?;
                Some(guard)
            }
        };
        files.revalidate(lock, reconcile)?;
        let genesis_digest = chain_genesis(&pointer)?;
        if reconcile {
            match &manifest.body {
                GenerationManifestBody::Genesis(_) => {
                    replay_authority::verify_genesis(lock, &manifest, &pointer, || Ok(()))?;
                }
                GenerationManifestBody::RecoveryBaseline(_) => {
                    let guard = recovery_guard.as_ref().ok_or_else(|| {
                        changed("recovery guard is absent before pending reconciliation")
                    })?;
                    crate::coord::generation::recovery::reverify_published_recovery(
                        lock.root(),
                        &manifest,
                        guard,
                    )?;
                    guard.revalidate()?;
                }
            }
            #[cfg(test)]
            replay_authority::inject_pre_effect_subject_swap()?;
            files.revalidate(lock, reconcile)?;
            let pre_effect_manifest = files.load_manifest(pointer.generation_id())?;
            if pre_effect_manifest != manifest {
                return Err(changed(
                    "generation manifest changed before pending reconciliation",
                ));
            }
            pointer.verify_manifest(&pre_effect_manifest)?;
            lock.revalidate()?;
            let pre_effect_current = lock
                .current()?
                .ok_or_else(|| changed("CURRENT disappeared before pending reconciliation"))?;
            if pre_effect_current != pointer {
                return Err(changed("CURRENT changed before pending reconciliation"));
            }
            if let Some(guard) = recovery_guard.as_ref() {
                guard.revalidate()?;
            }
            segment::reconcile_pending_files(
                &mut files.segment,
                &files.pending,
                pointer.generation_id().as_str(),
                &genesis_digest,
            )?;
        }
        let inspection = segment::inspect_files(
            &mut files.segment,
            &files.pending,
            pointer.generation_id().as_str(),
            &genesis_digest,
        )?;
        let records = verify::generation(&files, &pointer, &manifest, &inspection)?;
        let view = verify::view(&generation_dir, &pointer, &manifest, &inspection, records)?;
        match &manifest.body {
            GenerationManifestBody::Genesis(_) => {
                replay_authority::verify_genesis(lock, &manifest, &pointer, || Ok(()))?;
            }
            GenerationManifestBody::RecoveryBaseline(_) => {
                let guard = recovery_guard
                    .as_ref()
                    .ok_or_else(|| changed("recovery guard is absent after locked replay"))?;
                guard.revalidate()?;
                crate::coord::generation::recovery::reverify_published_recovery(
                    lock.root(),
                    &manifest,
                    guard,
                )?;
            }
        }
        #[cfg(test)]
        replay_authority::inject_final_manifest_rewrite()?;
        let final_manifest = files.load_manifest(pointer.generation_id())?;
        if final_manifest != manifest {
            return Err(changed("generation manifest changed during locked replay"));
        }
        pointer.verify_manifest(&final_manifest)?;
        files.revalidate(lock, reconcile)?;
        lock.revalidate()?;
        let final_pointer = lock
            .current()?
            .ok_or_else(|| changed("CURRENT disappeared during locked replay"))?;
        if final_pointer != pointer {
            return Err(changed("CURRENT changed during locked replay"));
        }
        if let Some(guard) = recovery_guard.as_ref() {
            guard.revalidate()?;
        }
        Ok(Loaded {
            pointer,
            manifest,
            genesis_digest,
            files,
            inspection,
            view,
        })
    }
}

fn chain_genesis(pointer: &CurrentPointer) -> Result<String, CoordError> {
    pointer
        .manifest_blake3()
        .strip_prefix("blake3:")
        .map(ToOwned::to_owned)
        .ok_or_else(|| invalid("CURRENT manifest digest is not tagged BLAKE3"))
}

fn genesis_request_id(pointer: &CurrentPointer) -> Result<String, CoordError> {
    let digest = bullet_wire::hash_framed_bytes(
        "bullet.coord.genesis-request-id.v2",
        pointer.manifest_blake3().as_bytes(),
    )
    .map_err(|error| invalid(format!("cannot derive GENESIS request ID: {error}")))?;
    Ok(format!("req_genesis_{}", digest.to_hex()))
}

fn uninitialized() -> CoordError {
    CoordError::new(
        "COORD_NOT_INITIALIZED",
        "coordination generation has not been initialized",
    )
}

fn recovery_required() -> CoordError {
    CoordError::new(
        "COORD_RECOVERY_REQUIRED",
        "legacy events.jsonl exists without CURRENT; explicit recovery is required",
    )
}

fn recovery_in_progress() -> CoordError {
    CoordError::new(
        "COORD_RECOVERY_IN_PROGRESS",
        "legacy source is retired but CURRENT is not yet durably published",
    )
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_LEDGER", reason)
}

fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_SUBJECT_CHANGED", reason)
}

fn wire(error: bullet_wire::WireError) -> CoordError {
    invalid(format!("canonical coordination record failed: {error}"))
}

fn fence_unknown(reason: impl Into<String>) -> CoordError {
    CoordError::new("COORD_FENCE_UNKNOWN", reason)
}

fn validate_request_id(value: &str) -> Result<(), CoordError> {
    if value.len() == 68
        && value.starts_with("req_")
        && value[4..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(CoordError::new(
            "INVALID_COORD_REQUEST_ID",
            "request ID must be req_ plus 64 lowercase hexadecimal digits",
        ))
    }
}

#[cfg(test)]
#[path = "ledger/tests.rs"]
mod tests;
