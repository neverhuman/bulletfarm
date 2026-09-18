//! Immutable pre-dispatch budget reservation for one bounded dogfood turn.
//! Settlement, actual usage, release, and UNKNOWN liability are separate facts.

use serde::{Deserialize, Serialize};

use crate::{
    Blake3Digest, DogfoodBudgetReservationId, DogfoodRunId, LaunchProvider, ProviderEnrollmentId,
    ProviderProfileId, WireError, decode_canonical, hash_canonical, ids::require_exact_wire,
};

use super::{DOGFOOD_SCHEMA_VERSION, DogfoodReadOnlyIntentV1, ProviderEnrollmentClaimsV2};

pub const DOGFOOD_BUDGET_RESERVATION_DIGEST_DOMAIN: &str = "dogfood.budget-reservation.v1alpha1";
pub const MAX_DOGFOOD_BUDGET_CONSUME_WINDOW_MS: u64 = 15_000;

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// One held forecast that may be consumed exactly once before provider spawn.
/// It contains no caller-selected settlement or quota-headroom assertion.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DogfoodBudgetReservationV1 {
    pub schema_version: String,
    pub reservation_id: DogfoodBudgetReservationId,
    pub run_id: DogfoodRunId,
    pub provider: LaunchProvider,
    pub provider_profile_id: ProviderProfileId,
    pub provider_enrollment_id: ProviderEnrollmentId,
    pub budget_policy_digest: Blake3Digest,
    pub reserved_at_unix_ms: u64,
    pub consume_before_unix_ms: u64,
    pub reserved_cost_micro_usd: u64,
    pub reserved_invocations: u64,
    pub reserved_wall_time_ms: u64,
    pub reserved_concurrency: u64,
}

impl DogfoodBudgetReservationV1 {
    /// Validate only the closed immutable component shape.
    pub fn validate(&self) -> Result<(), WireError> {
        require_exact_wire(
            "schema_version",
            &self.schema_version,
            DOGFOOD_SCHEMA_VERSION,
            "DOGFOOD_BUDGET_RESERVATION_INVALID",
        )?;
        for (name, value) in [
            ("reserved_at_unix_ms", self.reserved_at_unix_ms),
            ("consume_before_unix_ms", self.consume_before_unix_ms),
            ("reserved_cost_micro_usd", self.reserved_cost_micro_usd),
            ("reserved_invocations", self.reserved_invocations),
            ("reserved_wall_time_ms", self.reserved_wall_time_ms),
            ("reserved_concurrency", self.reserved_concurrency),
        ] {
            if value > MAX_SAFE_INTEGER {
                return Err(invalid(format!("{name} exceeds the safe integer range")));
            }
        }
        if self.reserved_at_unix_ms >= self.consume_before_unix_ms {
            return Err(invalid("reservation time must precede consume-before time"));
        }
        if self.consume_before_unix_ms - self.reserved_at_unix_ms
            > MAX_DOGFOOD_BUDGET_CONSUME_WINDOW_MS
        {
            return Err(invalid("reservation consumption window exceeds 15 seconds"));
        }
        if self.reserved_cost_micro_usd == 0 || self.reserved_wall_time_ms == 0 {
            return Err(invalid("reserved cost and wall time must be positive"));
        }
        if self.reserved_invocations != 1 || self.reserved_concurrency != 1 {
            return Err(invalid(
                "the first bounded dogfood turn reserves one invocation and one concurrency unit",
            ));
        }
        if self
            .consume_before_unix_ms
            .checked_add(self.reserved_wall_time_ms)
            .is_none_or(|end| end > MAX_SAFE_INTEGER)
        {
            return Err(invalid(
                "reservation completion bound is not a safe integer",
            ));
        }
        Ok(())
    }

    /// Domain-separated digest of every immutable reservation field.
    pub fn reservation_digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate()?;
        hash_canonical(DOGFOOD_BUDGET_RESERVATION_DIGEST_DOMAIN, self)
    }
}

pub fn decode_dogfood_budget_reservation(
    bytes: &[u8],
) -> Result<DogfoodBudgetReservationV1, WireError> {
    let reservation: DogfoodBudgetReservationV1 = decode_canonical(bytes)?;
    reservation.validate()?;
    Ok(reservation)
}

/// Require the held reservation to match the exact intent and enrollment.
/// Success grants no capacity, spend, launch, or settlement authority.
pub fn verify_dogfood_budget_binding(
    reservation: &DogfoodBudgetReservationV1,
    intent: &DogfoodReadOnlyIntentV1,
    enrollment: &ProviderEnrollmentClaimsV2,
) -> Result<(), WireError> {
    reservation.validate()?;
    intent.validate()?;
    enrollment.validate()?;

    if reservation.reservation_id != intent.subject.budget_reservation_id {
        return Err(WireError::new(
            "DOGFOOD_BUDGET_RESERVATION_ID_MISMATCH",
            "reservation id does not match the intent budget subject",
        ));
    }

    let provider = &intent.subject.provider;
    if reservation.run_id != intent.subject.execution.run_id
        || reservation.provider != provider.provider
        || reservation.provider != enrollment.provider
        || reservation.provider_profile_id != provider.provider_profile_id
        || reservation.provider_profile_id != enrollment.provider_profile_id
        || reservation.provider_enrollment_id != provider.provider_enrollment_id
        || reservation.provider_enrollment_id != enrollment.enrollment_id()?
        || reservation.budget_policy_digest != enrollment.budget_policy_digest
    {
        return Err(WireError::new(
            "DOGFOOD_BUDGET_SUBJECT_MISMATCH",
            "reservation does not bind the exact run, provider, profile, enrollment, and policy",
        ));
    }
    Ok(())
}

fn invalid(reason: impl Into<String>) -> WireError {
    WireError::new("DOGFOOD_BUDGET_RESERVATION_INVALID", reason)
}
