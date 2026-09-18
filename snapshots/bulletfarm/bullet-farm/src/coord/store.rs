use std::{path::Path, path::PathBuf, sync::Arc};

use crate::coord::{
    Applied, ClaimSummary, CoordError, GenesisInput, RecoveryCommand, RecoveryExecution,
    RecoveryReceiptAdoptionRequestV1, Status,
    generation::manifest::Sha256Digest,
    model::{RecoveryProductionPlanV1, RecoveryProofRequestV1, RecoveryReviewRequestV1},
    state::normalized_paths,
    validate_commit_oid, validate_field,
};

mod ledger;
pub(in crate::coord) mod legacy;
mod mutations;
mod projection;
pub(in crate::coord) mod subject;

use ledger::{GenesisProvenance, Ledger};

type Clock = Arc<dyn Fn() -> Result<u64, CoordError> + Send + Sync>;

pub struct CoordStore {
    pub(super) root: PathBuf,
    clock: Clock,
}

impl CoordStore {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            clock: Arc::new(crate::coord::unix_millis),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_clock(
        root: PathBuf,
        clock: impl Fn() -> Result<u64, CoordError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            root,
            clock: Arc::new(clock),
        }
    }

    /// Family checkout root this coordinator was opened against.
    #[must_use]
    pub fn family_root(&self) -> &Path {
        &self.root
    }

    pub fn initialize(&self, input: &GenesisInput) -> Result<Status, CoordError> {
        let provenance = genesis_provenance(input)?;
        let view = Ledger::new(&self.root).initialize_genesis(&provenance, || self.now())?;
        let observed = view
            .records
            .last()
            .map(subject::record_time)
            .transpose()?
            .ok_or_else(|| invalid("initialized generation has no transition record"))?;
        projection::status(view, observed)
    }

    pub fn status(&self) -> Result<Status, CoordError> {
        let view = Ledger::new(&self.root).status()?;
        projection::status(view, self.now()?)
    }

    pub fn adopt_recovery_receipts(
        &self,
        request: &RecoveryReceiptAdoptionRequestV1,
    ) -> Result<Applied<Vec<ClaimSummary>>, CoordError> {
        let transaction =
            Ledger::new(&self.root).adopt_recovery_receipts(request, || self.now())?;
        projection::many(transaction, request.request_subject_blake3()?)
    }

    pub(crate) fn derive_recovery_plan(&self) -> Result<RecoveryProductionPlanV1, CoordError> {
        require_recovery_production_platform()?;
        Ledger::new(&self.root).derive_recovery_plan()
    }

    pub(crate) fn record_recovery_proof(
        &self,
        request: &RecoveryProofRequestV1,
    ) -> Result<Applied<String>, CoordError> {
        require_recovery_production_platform()?;
        let (transaction, proof_id) =
            Ledger::new(&self.root).record_recovery_proof(request, || self.now())?;
        projection::producer(
            transaction,
            request.plan.evidence_subject_blake3.clone(),
            proof_id,
        )
    }

    pub(crate) fn record_recovery_review(
        &self,
        request: &RecoveryReviewRequestV1,
    ) -> Result<Applied<String>, CoordError> {
        require_recovery_production_platform()?;
        let (transaction, review_id) =
            Ledger::new(&self.root).record_recovery_review(request, || self.now())?;
        projection::producer(
            transaction,
            request.plan.evidence_subject_blake3.clone(),
            review_id,
        )
    }

    pub(crate) fn build_recovery_adoption_request(
        &self,
        request: &RecoveryReviewRequestV1,
    ) -> Result<RecoveryReceiptAdoptionRequestV1, CoordError> {
        require_recovery_production_platform()?;
        Ledger::new(&self.root).build_recovery_adoption_request(request)
    }

    pub(crate) fn recover_rollover(
        &self,
        command: &RecoveryCommand,
    ) -> Result<RecoveryExecution, CoordError> {
        crate::coord::recovery::execute(&self.root, command)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub(super) fn now(&self) -> Result<u64, CoordError> {
        let now = (self.clock)()?;
        if now == 0 || now > 9_007_199_254_740_991 {
            return Err(CoordError::new(
                "INVALID_COORD_TIME",
                "coordinator time must be a positive JSON-safe integer",
            ));
        }
        Ok(now)
    }
}

fn genesis_provenance(input: &GenesisInput) -> Result<GenesisProvenance, CoordError> {
    validate_field("operator", &input.operator)?;
    validate_commit_oid(&input.bootstrap_commit_oid)?;
    Ok(GenesisProvenance {
        operator: input.operator.clone(),
        policy_sha256: Sha256Digest::parse(&input.policy_sha256)?,
        replay_contract_version: input.replay_contract_version,
        replay_contract_sha256: Sha256Digest::parse(&input.replay_contract_sha256)?,
        bootstrap_commit_oid: input.bootstrap_commit_oid.clone(),
        bootstrap_paths: normalized_paths(&input.bootstrap_paths)?,
    })
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_STATUS", reason)
}

#[cfg(target_os = "linux")]
fn require_recovery_production_platform() -> Result<(), CoordError> {
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn require_recovery_production_platform() -> Result<(), CoordError> {
    Err(CoordError::new(
        "COORD_RECOVERY_PLATFORM_UNSUPPORTED",
        "recovery production is unavailable until this platform has an exact native proof",
    ))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        sync::{
            Arc,
            atomic::{AtomicU64, AtomicUsize, Ordering},
        },
    };

    use crate::coord::{
        ClaimInput, GenerationId, MutationEnvelope, RequestId, model::GenesisInput,
    };

    use super::CoordStore;

    mod repository_guard;

    fn genesis() -> GenesisInput {
        GenesisInput {
            operator: "operator-a".to_owned(),
            policy_sha256: format!("sha256:{}", "a".repeat(64)),
            replay_contract_version: 1,
            replay_contract_sha256: format!("sha256:{}", "b".repeat(64)),
            bootstrap_commit_oid: "c".repeat(40),
            bootstrap_paths: vec!["Cargo.toml".to_owned(), "src".to_owned()],
        }
    }

    fn request(index: u8) -> RequestId {
        RequestId::parse(format!("req_{index:064x}")).unwrap()
    }

    fn claim(
        request_id: RequestId,
        generation_id: GenerationId,
        agent: &str,
    ) -> MutationEnvelope<ClaimInput> {
        MutationEnvelope {
            request_id,
            expected_generation_id: generation_id,
            command: ClaimInput {
                agent: agent.to_owned(),
                lane: "lane-a".to_owned(),
                repo: "bullet-farm".to_owned(),
                paths: vec!["src".to_owned()],
                ttl_seconds: 600,
            },
        }
    }

    fn store(root: &tempfile::TempDir, now: Arc<AtomicU64>, calls: Arc<AtomicUsize>) -> CoordStore {
        let repo = root.path().join("bullet-farm");
        if !repo.exists() {
            fs::create_dir_all(repo.join(".git/objects")).unwrap();
            let manifest = format!(
                "[[repo]]\nname = \"bullet-farm\"\npath = {}\n",
                serde_json::to_string(repo.to_str().unwrap()).unwrap()
            );
            fs::write(root.path().join("repos.manifest.toml"), manifest).unwrap();
        }
        CoordStore::with_clock(root.path().to_path_buf(), move || {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(now.load(Ordering::SeqCst))
        })
    }

    #[test]
    fn absent_status_and_first_mutation_are_creation_free() {
        let root = tempfile::tempdir().unwrap();
        let now = Arc::new(AtomicU64::new(1_000));
        let calls = Arc::new(AtomicUsize::new(0));
        let store = store(&root, now, calls.clone());
        assert_eq!(store.status().unwrap_err().code(), "COORD_NOT_INITIALIZED");
        let envelope = claim(
            request(1),
            GenerationId::parse(format!("gen_{}", "a".repeat(64))).unwrap(),
            "agent-a",
        );
        assert_eq!(
            store.claim(&envelope).unwrap_err().code(),
            "COORD_NOT_INITIALIZED"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(!root.path().join(".bullet-family").exists());
    }

    #[test]
    fn exact_retry_replays_without_clock_and_changed_intent_never_appends() {
        let root = tempfile::tempdir().unwrap();
        let now = Arc::new(AtomicU64::new(1_000));
        let calls = Arc::new(AtomicUsize::new(0));
        let first_store = store(&root, now.clone(), calls.clone());
        let initialized = first_store.initialize(&genesis()).unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let generation = GenerationId::parse(initialized.generation_id).unwrap();
        let envelope = claim(request(2), generation.clone(), "agent-a");
        let applied = first_store.claim(&envelope).unwrap();
        assert!(!applied.replayed);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let source = root
            .path()
            .join(".bullet-family/coord/generations")
            .join(generation.as_str())
            .join("events.jsonl");
        let length = std::fs::metadata(&source).unwrap().len();

        now.store(500_000, Ordering::SeqCst);
        let replay_store = store(&root, now, calls.clone());
        let replayed = replay_store.claim(&envelope).unwrap();
        assert!(replayed.replayed);
        assert_eq!(replayed.receipt, applied.receipt);
        assert_eq!(replayed.watermark, applied.watermark);
        assert_eq!(replayed.projection, applied.projection);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(std::fs::metadata(&source).unwrap().len(), length);

        let changed = claim(envelope.request_id.clone(), generation, "agent-b");
        assert_eq!(
            replay_store.claim(&changed).unwrap_err().code(),
            "COORD_REQUEST_CONFLICT"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(std::fs::metadata(source).unwrap().len(), length);

        repository_guard::run();
    }

    #[test]
    fn stale_generation_refuses_before_clock_or_append() {
        let root = tempfile::tempdir().unwrap();
        let now = Arc::new(AtomicU64::new(1_000));
        let calls = Arc::new(AtomicUsize::new(0));
        let store = store(&root, now, calls.clone());
        let initialized = store.initialize(&genesis()).unwrap();
        let generation = GenerationId::parse(initialized.generation_id).unwrap();
        let first = store
            .claim(&claim(request(3), generation.clone(), "agent-a"))
            .unwrap();
        let source = root
            .path()
            .join(".bullet-family/coord/generations")
            .join(generation.as_str())
            .join("events.jsonl");
        let length = std::fs::metadata(&source).unwrap().len();
        let before_calls = calls.load(Ordering::SeqCst);
        let stale = claim(
            request(4),
            GenerationId::parse(format!("gen_{}", "f".repeat(64))).unwrap(),
            "agent-b",
        );
        assert_eq!(
            store.claim(&stale).unwrap_err().code(),
            "COORD_SUBJECT_CHANGED"
        );
        assert_eq!(calls.load(Ordering::SeqCst), before_calls);
        assert_eq!(std::fs::metadata(source).unwrap().len(), length);
        assert_eq!(first.watermark.last_sequence, 2);
    }
}
