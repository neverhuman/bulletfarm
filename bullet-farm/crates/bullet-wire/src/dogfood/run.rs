use serde::{Deserialize, Serialize};

use crate::{
    Blake3Digest, ContentId, DogfoodGrantId, DogfoodIntentId, PrincipalId, WireError,
    canonical_json, decode_canonical, hash_canonical,
};

use super::{
    DOGFOOD_SCHEMA_VERSION, DogfoodBudgetReservationV1, DogfoodRunSubjectV1, validate_run_subject,
};

pub const DOGFOOD_RUN_DIGEST_DOMAIN: &str = "dogfood.run.v1alpha1";
pub const DOGFOOD_BUDGET_SETTLEMENT_DIGEST_DOMAIN: &str = "dogfood.budget-settlement.v1alpha1";
pub const MAX_DOGFOOD_RUN_BYTES: usize = 64 * 1024;
pub const MAX_DOGFOOD_CAPTURE_BYTES: u64 = 1024 * 1024;
pub const MAX_DOGFOOD_RETAINED_ARTIFACTS: usize = 64;
pub const MAX_DOGFOOD_RETAINED_BYTES: u64 = 32 * 1024 * 1024;
pub const MAX_DOGFOOD_PROPOSAL_ARTIFACT_BYTES: u64 = 256 * 1024 * 1024;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "knowledge", rename_all = "snake_case", deny_unknown_fields)]
pub enum DogfoodUsageSettlementV1 {
    Known {
        used: u64,
        released: u64,
        overrun: u64,
    },
    Unknown {
        retained: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DogfoodBudgetSettlementV1 {
    pub schema_version: String,
    pub reservation_id: crate::DogfoodBudgetReservationId,
    pub reservation_digest: Blake3Digest,
    pub settled_at_unix_ms: u64,
    pub cost_micro_usd: DogfoodUsageSettlementV1,
    pub invocations: DogfoodUsageSettlementV1,
    pub wall_time_ms: DogfoodUsageSettlementV1,
    pub concurrency: DogfoodUsageSettlementV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DogfoodProcessStateV1 {
    NotStarted,
    Exited { code: u16 },
    Signaled { signal: u16 },
    TimedOut,
    OutcomeUnknown,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DogfoodProcessObservationV1 {
    pub state: DogfoodProcessStateV1,
    pub started_at_unix_ms: Option<u64>,
    pub ended_at_unix_ms: Option<u64>,
    pub observation_digest: Blake3Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DogfoodArtifactRefV1 {
    pub digest: Blake3Digest,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DogfoodRunArtifactsV1 {
    pub stdout: DogfoodArtifactRefV1,
    pub stderr: DogfoodArtifactRefV1,
    pub events: DogfoodArtifactRefV1,
    pub proxy: DogfoodArtifactRefV1,
    pub containment_receipt_digest: Blake3Digest,
    pub egress_receipt_digest: Blake3Digest,
    pub canary_observation_digest: Blake3Digest,
    pub process_tree_observation_digest: Blake3Digest,
    pub artifact_manifest_digest: Blake3Digest,
    pub retained_artifacts: Vec<DogfoodArtifactRefV1>,
    pub retained_artifact_count: u64,
    pub retained_artifact_size_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DogfoodProposalObservationV1 {
    Absent,
    Rejected {
        artifact: DogfoodArtifactRefV1,
    },
    Validated {
        proposal_id: ContentId,
        proposal_digest: Blake3Digest,
        artifact: DogfoodArtifactRefV1,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DogfoodCleanupObservationV1 {
    ProvedEmpty {
        receipt_digest: Blake3Digest,
        observed_at_unix_ms: u64,
    },
    Quarantined {
        receipt_digest: Blake3Digest,
        residue_manifest_digest: Blake3Digest,
        observed_at_unix_ms: u64,
    },
    Unknown {
        receipt_digest: Blake3Digest,
        observed_at_unix_ms: u64,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DogfoodTerminalStateV1 {
    Quarantined,
    OutcomeUnknown,
    RefusedBeforeSpawn,
    ProposalReady,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DogfoodRunV1 {
    pub schema_version: String,
    pub subject: DogfoodRunSubjectV1,
    pub intent_id: DogfoodIntentId,
    pub launch_grant_id: DogfoodGrantId,
    pub credential_projection_digest: Blake3Digest,
    pub budget_settlement: DogfoodBudgetSettlementV1,
    pub repository_context_post_observation_digest: Blake3Digest,
    pub provider_probe_observation_digest: Blake3Digest,
    pub attestor_principal_id: PrincipalId,
    pub process: DogfoodProcessObservationV1,
    pub artifacts: DogfoodRunArtifactsV1,
    pub proposal: DogfoodProposalObservationV1,
    pub cleanup: DogfoodCleanupObservationV1,
    pub attested_at_unix_ms: u64,
}

impl DogfoodUsageSettlementV1 {
    fn validate(&self) -> Result<(), WireError> {
        let values: &[u64] = match self {
            Self::Known {
                used,
                released,
                overrun,
            } => &[*used, *released, *overrun],
            Self::Unknown { retained } => &[*retained],
        };
        values
            .iter()
            .all(|value| *value <= MAX_SAFE_INTEGER)
            .then_some(())
            .ok_or_else(|| settlement_invalid("usage settlement exceeds the safe integer range"))
    }

    fn matches_reserved(&self, reserved: u64) -> bool {
        match self {
            Self::Known {
                used,
                released,
                overrun,
            } => {
                *released == reserved.saturating_sub(*used)
                    && *overrun == used.saturating_sub(reserved)
            }
            Self::Unknown { retained } => *retained == reserved,
        }
    }

    fn is_zero_known(&self) -> bool {
        matches!(
            self,
            Self::Known {
                used: 0,
                overrun: 0,
                ..
            }
        )
    }
    fn known_without_overrun(&self) -> bool {
        matches!(self, Self::Known { overrun: 0, .. })
    }
    fn used(&self) -> Option<u64> {
        match self {
            Self::Known { used, .. } => Some(*used),
            Self::Unknown { .. } => None,
        }
    }
}

impl DogfoodBudgetSettlementV1 {
    pub fn validate(&self) -> Result<(), WireError> {
        if self.schema_version != DOGFOOD_SCHEMA_VERSION || !safe_time(self.settled_at_unix_ms) {
            return Err(settlement_invalid("invalid schema or settlement time"));
        }
        for row in self.rows() {
            row.validate()?;
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate()?;
        hash_canonical(DOGFOOD_BUDGET_SETTLEMENT_DIGEST_DOMAIN, self)
    }

    pub fn has_unknown_liability(&self) -> bool {
        self.rows().iter().any(|row| row.used().is_none())
    }

    pub fn validate_against(
        &self,
        reservation: &DogfoodBudgetReservationV1,
    ) -> Result<(), WireError> {
        self.validate()?;
        reservation.validate()?;
        if self.reservation_id != reservation.reservation_id
            || self.reservation_digest != reservation.reservation_digest()?
            || self.settled_at_unix_ms < reservation.reserved_at_unix_ms
            || !self
                .cost_micro_usd
                .matches_reserved(reservation.reserved_cost_micro_usd)
            || !self
                .invocations
                .matches_reserved(reservation.reserved_invocations)
            || !self
                .wall_time_ms
                .matches_reserved(reservation.reserved_wall_time_ms)
            || !self
                .concurrency
                .matches_reserved(reservation.reserved_concurrency)
        {
            return Err(run_mismatch(
                "DOGFOOD_RUN_SETTLEMENT_MISMATCH",
                "settlement does not conserve the reservation",
            ));
        }
        Ok(())
    }

    fn rows(&self) -> [&DogfoodUsageSettlementV1; 4] {
        [
            &self.cost_micro_usd,
            &self.invocations,
            &self.wall_time_ms,
            &self.concurrency,
        ]
    }
}

impl DogfoodProcessObservationV1 {
    fn validate(&self) -> Result<(), WireError> {
        for value in [self.started_at_unix_ms, self.ended_at_unix_ms]
            .into_iter()
            .flatten()
        {
            if !safe_time(value) {
                return Err(run_invalid("invalid process observation time"));
            }
        }
        let times_ok = match self.state {
            DogfoodProcessStateV1::NotStarted => {
                self.started_at_unix_ms.is_none() && self.ended_at_unix_ms.is_none()
            }
            DogfoodProcessStateV1::OutcomeUnknown => self
                .started_at_unix_ms
                .zip(self.ended_at_unix_ms)
                .is_none_or(|(start, end)| start <= end),
            _ => self
                .started_at_unix_ms
                .zip(self.ended_at_unix_ms)
                .is_some_and(|(start, end)| start <= end),
        };
        let value_ok = match self.state {
            DogfoodProcessStateV1::Exited { code } => code <= 255,
            DogfoodProcessStateV1::Signaled { signal } => (1..=255).contains(&signal),
            _ => true,
        };
        (times_ok && value_ok)
            .then_some(())
            .ok_or_else(|| run_invalid("invalid process state/time combination"))
    }
}

impl DogfoodArtifactRefV1 {
    fn validate(&self, max: u64) -> Result<(), WireError> {
        (self.size_bytes <= max && self.size_bytes <= MAX_SAFE_INTEGER)
            .then_some(())
            .ok_or_else(|| run_invalid("artifact size exceeds its bound"))
    }
}

impl DogfoodRunArtifactsV1 {
    fn validate(&self) -> Result<(), WireError> {
        for capture in [&self.stdout, &self.stderr, &self.events, &self.proxy] {
            capture.validate(MAX_DOGFOOD_CAPTURE_BYTES)?;
        }
        if self.retained_artifacts.len() > MAX_DOGFOOD_RETAINED_ARTIFACTS
            || self
                .retained_artifacts
                .windows(2)
                .any(|pair| pair[0].digest >= pair[1].digest)
        {
            return Err(run_invalid(
                "retained artifacts are oversized, unsorted, or duplicated",
            ));
        }
        let mut total = 0_u64;
        for artifact in &self.retained_artifacts {
            artifact.validate(MAX_DOGFOOD_RETAINED_BYTES)?;
            total = total
                .checked_add(artifact.size_bytes)
                .ok_or_else(|| run_invalid("artifact size overflow"))?;
        }
        if self.retained_artifact_count != self.retained_artifacts.len() as u64
            || self.retained_artifact_size_bytes != total
            || total > MAX_DOGFOOD_RETAINED_BYTES
        {
            return Err(run_invalid("retained artifact aggregates do not match"));
        }
        Ok(())
    }
}

impl DogfoodProposalObservationV1 {
    fn validate(&self) -> Result<(), WireError> {
        match self {
            Self::Absent => Ok(()),
            Self::Rejected { artifact } => artifact.validate(MAX_DOGFOOD_RETAINED_BYTES),
            Self::Validated { artifact, .. } => {
                artifact.validate(MAX_DOGFOOD_PROPOSAL_ARTIFACT_BYTES)
            }
        }
    }
}
impl DogfoodCleanupObservationV1 {
    fn validate(&self) -> Result<(), WireError> {
        safe_time(self.observed_at())
            .then_some(())
            .ok_or_else(|| run_invalid("invalid cleanup observation time"))
    }
    fn observed_at(&self) -> u64 {
        match self {
            Self::ProvedEmpty {
                observed_at_unix_ms,
                ..
            }
            | Self::Quarantined {
                observed_at_unix_ms,
                ..
            }
            | Self::Unknown {
                observed_at_unix_ms,
                ..
            } => *observed_at_unix_ms,
        }
    }
    fn proved_empty(&self) -> bool {
        matches!(self, Self::ProvedEmpty { .. })
    }
}

impl DogfoodRunV1 {
    pub fn validate(&self) -> Result<(), WireError> {
        if self.schema_version != DOGFOOD_SCHEMA_VERSION || !safe_time(self.attested_at_unix_ms) {
            return Err(run_invalid("invalid schema or attestation time"));
        }
        validate_run_subject(&self.subject, "DOGFOOD_RUN_INVALID")?;
        self.budget_settlement.validate()?;
        self.process.validate()?;
        self.artifacts.validate()?;
        self.proposal.validate()?;
        self.cleanup.validate()?;
        let process_time = self
            .process
            .ended_at_unix_ms
            .or(self.process.started_at_unix_ms)
            .unwrap_or(0);
        if self.budget_settlement.settled_at_unix_ms < process_time
            || self.cleanup.observed_at() < process_time
            || self.attested_at_unix_ms < self.budget_settlement.settled_at_unix_ms
            || self.attested_at_unix_ms < self.cleanup.observed_at()
        {
            return Err(run_mismatch(
                "DOGFOOD_RUN_TIME_MISMATCH",
                "terminal observations are not causal",
            ));
        }
        if matches!(self.process.state, DogfoodProcessStateV1::OutcomeUnknown)
            && (!self
                .budget_settlement
                .rows()
                .iter()
                .all(|row| row.used().is_none())
                || matches!(
                    self.proposal,
                    DogfoodProposalObservationV1::Validated { .. }
                ))
        {
            return Err(run_mismatch(
                "DOGFOOD_RUN_PROCESS_MISMATCH",
                "unknown process requires unknown usage and no validated proposal",
            ));
        }
        if matches!(self.process.state, DogfoodProcessStateV1::NotStarted)
            && (!self
                .budget_settlement
                .rows()
                .iter()
                .all(|row| row.is_zero_known())
                || !matches!(self.proposal, DogfoodProposalObservationV1::Absent))
        {
            return Err(run_mismatch(
                "DOGFOOD_RUN_PROCESS_MISMATCH",
                "not-started requires absent proposal and zero known use",
            ));
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate()?;
        let bytes = canonical_json(self)?;
        if bytes.len() > MAX_DOGFOOD_RUN_BYTES {
            return Err(run_invalid("dogfood run exceeds 64 KiB"));
        }
        hash_canonical(DOGFOOD_RUN_DIGEST_DOMAIN, self)
    }

    pub fn terminal_state(
        &self,
        reservation: &DogfoodBudgetReservationV1,
    ) -> Result<DogfoodTerminalStateV1, WireError> {
        self.validate()?;
        self.budget_settlement.validate_against(reservation)?;
        if !self.cleanup.proved_empty() {
            return Ok(DogfoodTerminalStateV1::Quarantined);
        }
        if matches!(self.process.state, DogfoodProcessStateV1::OutcomeUnknown) {
            return Ok(DogfoodTerminalStateV1::OutcomeUnknown);
        }
        if matches!(self.process.state, DogfoodProcessStateV1::NotStarted) {
            return Ok(DogfoodTerminalStateV1::RefusedBeforeSpawn);
        }
        let ready = matches!(
            self.process.state,
            DogfoodProcessStateV1::Exited { code: 0 }
        ) && matches!(
            self.proposal,
            DogfoodProposalObservationV1::Validated { .. }
        ) && self
            .budget_settlement
            .rows()
            .iter()
            .all(|row| row.known_without_overrun())
            && self.budget_settlement.invocations.used() == Some(1)
            && self.budget_settlement.concurrency.used() == Some(1);
        Ok(if ready {
            DogfoodTerminalStateV1::ProposalReady
        } else {
            DogfoodTerminalStateV1::Failed
        })
    }
}

pub fn decode_dogfood_run(bytes: &[u8]) -> Result<DogfoodRunV1, WireError> {
    if bytes.len() > MAX_DOGFOOD_RUN_BYTES {
        return Err(run_invalid("dogfood run exceeds 64 KiB"));
    }
    let run: DogfoodRunV1 =
        decode_canonical(bytes).map_err(|error| run_invalid(error.to_string()))?;
    run.validate()?;
    Ok(run)
}

fn safe_time(value: u64) -> bool {
    value > 0 && value <= MAX_SAFE_INTEGER
}
fn settlement_invalid(reason: impl Into<String>) -> WireError {
    WireError::new("DOGFOOD_BUDGET_SETTLEMENT_INVALID", reason)
}
fn run_invalid(reason: impl Into<String>) -> WireError {
    WireError::new("DOGFOOD_RUN_INVALID", reason)
}
fn run_mismatch(code: &'static str, reason: impl Into<String>) -> WireError {
    WireError::new(code, reason)
}
