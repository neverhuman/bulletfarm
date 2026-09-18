use std::collections::BTreeMap;

use crate::coord::{
    CoordError,
    generation::manifest::{
        TrustedClaimOutcomeCounts, TrustedProjectionInventory, TrustedRecordKindCounts,
    },
    model::{ClaimState, ClaimSummary, Record},
};

pub(in crate::coord) fn inventory(
    records: &[Record],
    claims: &BTreeMap<String, ClaimSummary>,
) -> Result<TrustedProjectionInventory, CoordError> {
    let mut kinds = TrustedRecordKindCounts {
        claim: 0,
        heartbeat: 0,
        handoff: 0,
        commit_receipt: 0,
        commit_receipt_correction: 0,
        commit_receipt_group: 0,
        commit_receipt_group_correction: 0,
    };
    for record in records {
        let count = match record {
            Record::Claim { .. } => &mut kinds.claim,
            Record::Heartbeat { .. } => &mut kinds.heartbeat,
            Record::Handoff { .. } => &mut kinds.handoff,
            Record::CommitReceipt { .. } => &mut kinds.commit_receipt,
            Record::CommitReceiptCorrection { .. } => &mut kinds.commit_receipt_correction,
            Record::CommitReceiptGroup { .. } => &mut kinds.commit_receipt_group,
            Record::CommitReceiptGroupCorrection { .. } => {
                &mut kinds.commit_receipt_group_correction
            }
            Record::GenesisV2 { .. }
            | Record::RecoveryBaselineV2 { .. }
            | Record::RecoveryReceiptAdoptionV1 { .. }
            | Record::RecoveryProofReceiptV1 { .. }
            | Record::RecoveryReviewReceiptV1 { .. } => {
                return Err(invalid(
                    "trusted schema-1 prefix contains a generation-only record",
                ));
            }
        };
        *count = count
            .checked_add(1)
            .ok_or_else(|| invalid("trusted record-kind inventory overflowed"))?;
    }
    let mut outcomes = TrustedClaimOutcomeCounts {
        total: u64::try_from(claims.len())
            .map_err(|_| invalid("trusted claim count exceeds u64"))?,
        active: 0,
        expired: 0,
        handed_off_unreceipted: 0,
        receipted: 0,
    };
    for claim in claims.values() {
        let count = match claim.state {
            ClaimState::Active => &mut outcomes.active,
            ClaimState::Expired => &mut outcomes.expired,
            ClaimState::HandedOff if claim.commit_oid.is_some() => &mut outcomes.receipted,
            ClaimState::HandedOff => &mut outcomes.handed_off_unreceipted,
            ClaimState::FrozenRecovery | ClaimState::RecoveredReceipted => {
                return Err(invalid(
                    "trusted schema-1 prefix contains a generation-only claim outcome",
                ));
            }
        };
        *count = count
            .checked_add(1)
            .ok_or_else(|| invalid("trusted claim-outcome inventory overflowed"))?;
    }
    let inventory = TrustedProjectionInventory {
        record_kinds: kinds,
        claim_outcomes: outcomes,
    };
    inventory.validate(records.len() as u64)?;
    Ok(inventory)
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_COORD_RECOVERY", reason)
}
