//! Kind-specific semantic admission from committed Hub bytes. A readable
//! sibling checkout is neither immutable nor an evidence subject.

use super::RowScore;
use super::spec::{CriterionRow, EvidenceKind, EvidenceReference};
use serde_json::Value;
use std::path::Path;

pub(super) fn admit_row(hub: &Path, row: &CriterionRow) -> RowScore {
    let Some(reference) = row.evidence.as_ref() else {
        return refused(row, "NO_EVIDENCE_REFERENCE");
    };
    match (row.kind, reference) {
        (EvidenceKind::CiTest, EvidenceReference::CiObservation { subject_id }) => {
            admit_ci_test(hub, row, subject_id)
        }
        (EvidenceKind::Gate, EvidenceReference::ReleaseGate { .. }) => {
            refused(row, "RELEASE_GATE_NOT_ADMITTED")
        }
        (EvidenceKind::Receipt, EvidenceReference::SignedReceipt { .. }) => {
            refused(row, "SIGNED_RECEIPT_UNAVAILABLE")
        }
        (EvidenceKind::SourceReceipt, EvidenceReference::SourceReceipt { .. }) => {
            refused(row, "SOURCE_RECEIPT_UNAVAILABLE")
        }
        (EvidenceKind::ExternalReview, EvidenceReference::ExternalReview { .. }) => {
            refused(row, "EXTERNAL_REVIEW_UNAVAILABLE")
        }
        _ => refused(row, "EVIDENCE_KIND_MISMATCH"),
    }
}

fn admit_ci_test(hub: &Path, row: &CriterionRow, subject_id: &str) -> RowScore {
    let expected = match row.id.as_str() {
        "d1.nonce-ledger" => "scorecard.d1.nonce-ledger",
        "d3.proof-root-eight" => "scorecard.d3.proof-root-eight",
        "d5.budgets" => "scorecard.d5.budgets",
        "d7.evolution-off" => "scorecard.d7.evolution-off",
        _ => return refused(row, "SUBJECT_UNKNOWN"),
    };
    if subject_id != expected {
        return refused(row, "SUBJECT_MISMATCH");
    }
    match row.id.as_str() {
        "d1.nonce-ledger" | "d3.proof-root-eight" | "d5.budgets" => {
            refused(row, "PINNED_FAMILY_SUBJECT_UNAVAILABLE")
        }
        "d7.evolution-off" => require(row, evolutionary_authority_is_off(hub)),
        _ => refused(row, "SUBJECT_UNKNOWN"),
    }
}

fn require(row: &CriterionRow, result: Result<(), &'static str>) -> RowScore {
    match result {
        Ok(()) => RowScore {
            id: row.id.clone(),
            admitted: true,
            refusal_reason: "-".into(),
            claim: row.claim.clone(),
        },
        Err(reason) => refused(row, reason),
    }
}

fn refused(row: &CriterionRow, reason: &str) -> RowScore {
    RowScore {
        id: row.id.clone(),
        admitted: false,
        refusal_reason: reason.into(),
        claim: row.claim.clone(),
    }
}

fn evolutionary_authority_is_off(hub: &Path) -> Result<(), &'static str> {
    let bytes = std::fs::read(hub.join("policy/v1alpha1/policy.json"))
        .map_err(|_| "POLICY_BYTES_UNAVAILABLE")?;
    if bytes.len() > bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES {
        return Err("POLICY_BYTES_TOO_LARGE");
    }
    let value = bullet_wire::decode_unique_value(&bytes).map_err(|_| "POLICY_NOT_STRICT_JSON")?;
    match value
        .get("route_policy")
        .and_then(|policy| policy.get("evolutionary_authority"))
    {
        Some(Value::Bool(false)) => Ok(()),
        Some(Value::Bool(true)) => Err("EVOLUTIONARY_AUTHORITY_ENABLED"),
        _ => Err("EVOLUTIONARY_AUTHORITY_NOT_OFF"),
    }
}

#[cfg(test)]
mod tests {
    use super::evolutionary_authority_is_off;

    #[test]
    fn enabled_evolution_flag_is_refused() {
        let directory = tempfile::tempdir().unwrap();
        let policy = directory.path().join("policy/v1alpha1");
        std::fs::create_dir_all(&policy).unwrap();
        std::fs::write(
            policy.join("policy.json"),
            r#"{"route_policy":{"evolutionary_authority":true}}"#,
        )
        .unwrap();
        assert_eq!(
            evolutionary_authority_is_off(directory.path()),
            Err("EVOLUTIONARY_AUTHORITY_ENABLED")
        );
    }
}
