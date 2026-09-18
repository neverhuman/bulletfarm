use serde::{Deserialize, Serialize};

use crate::coord::CoordError;

use super::invalid;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TrustedRecordKindCounts {
    pub(crate) claim: u64,
    pub(crate) heartbeat: u64,
    pub(crate) handoff: u64,
    pub(crate) commit_receipt: u64,
    pub(crate) commit_receipt_correction: u64,
    pub(crate) commit_receipt_group: u64,
    pub(crate) commit_receipt_group_correction: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TrustedClaimOutcomeCounts {
    pub(crate) total: u64,
    pub(crate) active: u64,
    pub(crate) expired: u64,
    pub(crate) handed_off_unreceipted: u64,
    pub(crate) receipted: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TrustedProjectionInventory {
    pub(crate) record_kinds: TrustedRecordKindCounts,
    pub(crate) claim_outcomes: TrustedClaimOutcomeCounts,
}

impl TrustedProjectionInventory {
    pub(crate) fn validate(&self, trusted_record_count: u64) -> Result<(), CoordError> {
        let record_total = checked_sum(
            "trusted record-kind inventory",
            [
                self.record_kinds.claim,
                self.record_kinds.heartbeat,
                self.record_kinds.handoff,
                self.record_kinds.commit_receipt,
                self.record_kinds.commit_receipt_correction,
                self.record_kinds.commit_receipt_group,
                self.record_kinds.commit_receipt_group_correction,
            ],
        )?;
        if trusted_record_count == 0 || record_total != trusted_record_count {
            return Err(invalid(
                "trusted record-kind inventory must sum exactly to the nonzero trusted record count",
            ));
        }

        let outcome_total = checked_sum(
            "trusted claim-outcome inventory",
            [
                self.claim_outcomes.active,
                self.claim_outcomes.expired,
                self.claim_outcomes.handed_off_unreceipted,
                self.claim_outcomes.receipted,
            ],
        )?;
        if self.claim_outcomes.total == 0
            || self.record_kinds.claim != self.claim_outcomes.total
            || outcome_total != self.claim_outcomes.total
        {
            return Err(invalid(
                "trusted claim outcomes must form an exact nonzero partition of claim records",
            ));
        }

        self.validate_record_relationships()
    }

    fn validate_record_relationships(&self) -> Result<(), CoordError> {
        if self.record_kinds.handoff > self.claim_outcomes.total {
            return Err(invalid("handoff records cannot exceed claim records"));
        }
        let handed_off = checked_sum(
            "trusted handed-off outcomes",
            [
                self.claim_outcomes.handed_off_unreceipted,
                self.claim_outcomes.receipted,
            ],
        )?;
        if handed_off != self.record_kinds.handoff {
            return Err(invalid(
                "handed-off outcomes must exactly equal trusted handoff records",
            ));
        }
        let initial_receipts = checked_sum(
            "trusted initial receipt inventory",
            [
                self.record_kinds.commit_receipt,
                self.record_kinds.commit_receipt_group,
            ],
        )?;
        let receipt_corrections = checked_sum(
            "trusted receipt-correction inventory",
            [
                self.record_kinds.commit_receipt_correction,
                self.record_kinds.commit_receipt_group_correction,
            ],
        )?;
        if receipt_corrections != 0 && initial_receipts == 0 {
            return Err(invalid(
                "receipt corrections require at least one initial receipt record",
            ));
        }
        let grouped_minimum = self
            .record_kinds
            .commit_receipt_group
            .checked_mul(2)
            .ok_or_else(|| invalid("trusted grouped-receipt claim minimum overflowed"))?;
        let minimum_receipted = self
            .record_kinds
            .commit_receipt
            .checked_add(grouped_minimum)
            .ok_or_else(|| invalid("trusted receipt claim minimum overflowed"))?;
        if minimum_receipted > self.claim_outcomes.receipted
            || (self.claim_outcomes.receipted != 0
                && self.record_kinds.commit_receipt == 0
                && self.record_kinds.commit_receipt_group == 0)
        {
            return Err(invalid(
                "receipt records cannot account for the trusted receipted outcomes",
            ));
        }
        Ok(())
    }
}

fn checked_sum<const N: usize>(label: &str, values: [u64; N]) -> Result<u64, CoordError> {
    values.into_iter().try_fold(0_u64, |total, value| {
        total
            .checked_add(value)
            .ok_or_else(|| invalid(format!("{label} overflowed")))
    })
}

#[cfg(test)]
pub(super) fn trusted_inventory_fixture() -> TrustedProjectionInventory {
    TrustedProjectionInventory {
        record_kinds: TrustedRecordKindCounts {
            claim: 1_027,
            heartbeat: 2_515,
            handoff: 835,
            commit_receipt: 269,
            commit_receipt_correction: 4,
            commit_receipt_group: 142,
            commit_receipt_group_correction: 0,
        },
        claim_outcomes: TrustedClaimOutcomeCounts {
            total: 1_027,
            active: 20,
            expired: 172,
            handed_off_unreceipted: 106,
            receipted: 729,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::generation::manifest::{GenerationManifestBody, generation_id};

    #[test]
    fn closed_inventory_is_required_and_generation_sensitive() {
        let base_body = super::super::tests::body();
        let base = generation_id(&base_body).unwrap();
        let value = serde_json::to_value(&base_body).unwrap();
        for field in [
            "claim",
            "heartbeat",
            "handoff",
            "commit_receipt",
            "commit_receipt_correction",
            "commit_receipt_group",
            "commit_receipt_group_correction",
        ] {
            assert_nested_field_required(&value, "record_kinds", field);
        }
        for field in [
            "total",
            "active",
            "expired",
            "handed_off_unreceipted",
            "receipted",
        ] {
            assert_nested_field_required(&value, "claim_outcomes", field);
        }
        for parent in ["record_kinds", "claim_outcomes"] {
            let mut missing = value.clone();
            missing["trusted_projection_inventory"]
                .as_object_mut()
                .unwrap()
                .remove(parent);
            assert_rejected(missing);
        }
        let mut unknown = value;
        unknown["trusted_projection_inventory"]["unknown"] = serde_json::json!(1);
        assert_rejected(unknown);

        for candidate in sensitivity_candidates() {
            candidate.validate().unwrap();
            assert_ne!(generation_id(&candidate).unwrap(), base);
        }
    }

    #[test]
    fn inventory_rejects_zero_overflow_inconsistent_and_impossible_counts() {
        trusted_inventory_fixture().validate(4_792).unwrap();

        let mut record_mismatch = trusted_inventory_fixture();
        record_mismatch.record_kinds.heartbeat -= 1;
        assert!(record_mismatch.validate(4_792).is_err());

        let mut outcome_mismatch = trusted_inventory_fixture();
        outcome_mismatch.claim_outcomes.active -= 1;
        assert!(outcome_mismatch.validate(4_792).is_err());

        let mut overflow = trusted_inventory_fixture();
        overflow.record_kinds.heartbeat = u64::MAX;
        assert!(overflow.validate(u64::MAX).is_err());

        let mut outcome_overflow = zero_inventory();
        outcome_overflow.record_kinds.claim = u64::MAX;
        outcome_overflow.claim_outcomes.total = u64::MAX;
        outcome_overflow.claim_outcomes.active = u64::MAX;
        outcome_overflow.claim_outcomes.expired = 1;
        assert!(outcome_overflow.validate(u64::MAX).is_err());

        assert!(zero_inventory().validate(0).is_err());

        let mut too_many_handoffs = trusted_inventory_fixture();
        too_many_handoffs.record_kinds.handoff = 1_028;
        too_many_handoffs.record_kinds.heartbeat -= 193;
        assert!(too_many_handoffs.validate(4_792).is_err());

        let mut orphan_correction = trusted_inventory_fixture();
        orphan_correction.record_kinds.commit_receipt = 0;
        orphan_correction
            .record_kinds
            .commit_receipt_group_correction = 1;
        orphan_correction.record_kinds.commit_receipt_group = 0;
        orphan_correction.record_kinds.heartbeat += 410;
        orphan_correction.claim_outcomes.receipted = 0;
        orphan_correction.claim_outcomes.handed_off_unreceipted += 729;
        assert!(orphan_correction.validate(4_792).is_err());

        let mut undercovered_receipts = trusted_inventory_fixture();
        undercovered_receipts.claim_outcomes.receipted = 1;
        undercovered_receipts.claim_outcomes.handed_off_unreceipted += 728;
        assert!(undercovered_receipts.validate(4_792).is_err());
    }

    fn sensitivity_candidates() -> Vec<GenerationManifestBody> {
        vec![
            changed(|inventory| {
                inventory.record_kinds.claim += 1;
                inventory.record_kinds.heartbeat -= 1;
                inventory.claim_outcomes.total += 1;
                inventory.claim_outcomes.active += 1;
            }),
            shifted_record(
                |counts| counts.heartbeat += 1,
                |counts| counts.commit_receipt_correction -= 1,
            ),
            changed(|inventory| {
                inventory.record_kinds.handoff += 1;
                inventory.record_kinds.heartbeat -= 1;
                inventory.claim_outcomes.handed_off_unreceipted += 1;
                inventory.claim_outcomes.expired -= 1;
            }),
            shifted_record(
                |counts| counts.commit_receipt += 1,
                |counts| counts.heartbeat -= 1,
            ),
            shifted_record(
                |counts| counts.commit_receipt_correction += 1,
                |counts| counts.heartbeat -= 1,
            ),
            shifted_record(
                |counts| counts.commit_receipt_group += 1,
                |counts| counts.heartbeat -= 1,
            ),
            shifted_record(
                |counts| counts.commit_receipt_group_correction += 1,
                |counts| counts.heartbeat -= 1,
            ),
            changed(|inventory| {
                inventory.record_kinds.claim += 1;
                inventory.record_kinds.heartbeat -= 1;
                inventory.claim_outcomes.total += 1;
                inventory.claim_outcomes.expired += 1;
            }),
            shifted_outcome(|counts| counts.active += 1, |counts| counts.expired -= 1),
            shifted_outcome(|counts| counts.expired += 1, |counts| counts.active -= 1),
            changed(|inventory| {
                inventory.claim_outcomes.handed_off_unreceipted += 1;
                inventory.claim_outcomes.expired -= 1;
                inventory.record_kinds.handoff += 1;
                inventory.record_kinds.heartbeat -= 1;
            }),
            changed(|inventory| {
                inventory.claim_outcomes.receipted += 1;
                inventory.claim_outcomes.expired -= 1;
                inventory.record_kinds.handoff += 1;
                inventory.record_kinds.heartbeat -= 1;
            }),
        ]
    }

    fn shifted_record(
        increase: impl FnOnce(&mut TrustedRecordKindCounts),
        decrease: impl FnOnce(&mut TrustedRecordKindCounts),
    ) -> GenerationManifestBody {
        changed(|inventory| {
            increase(&mut inventory.record_kinds);
            decrease(&mut inventory.record_kinds);
        })
    }

    fn shifted_outcome(
        increase: impl FnOnce(&mut TrustedClaimOutcomeCounts),
        decrease: impl FnOnce(&mut TrustedClaimOutcomeCounts),
    ) -> GenerationManifestBody {
        changed(|inventory| {
            increase(&mut inventory.claim_outcomes);
            decrease(&mut inventory.claim_outcomes);
        })
    }

    fn changed(edit: impl FnOnce(&mut TrustedProjectionInventory)) -> GenerationManifestBody {
        let mut candidate = super::super::tests::body();
        edit(&mut super::super::tests::recovery_mut(&mut candidate).trusted_projection_inventory);
        candidate
    }

    fn assert_rejected(value: serde_json::Value) {
        let admitted = serde_json::from_value::<GenerationManifestBody>(value).and_then(|body| {
            body.validate()
                .map(|()| body)
                .map_err(serde::de::Error::custom)
        });
        assert!(admitted.is_err());
    }

    fn assert_nested_field_required(value: &serde_json::Value, parent: &str, field: &str) {
        let mut missing = value.clone();
        missing["trusted_projection_inventory"][parent]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert_rejected(missing);
    }

    fn zero_inventory() -> TrustedProjectionInventory {
        TrustedProjectionInventory {
            record_kinds: TrustedRecordKindCounts {
                claim: 0,
                heartbeat: 0,
                handoff: 0,
                commit_receipt: 0,
                commit_receipt_correction: 0,
                commit_receipt_group: 0,
                commit_receipt_group_correction: 0,
            },
            claim_outcomes: TrustedClaimOutcomeCounts {
                total: 0,
                active: 0,
                expired: 0,
                handed_off_unreceipted: 0,
                receipted: 0,
            },
        }
    }
}
