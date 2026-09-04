use bullet_wire::v1alpha1::{GateReceiptV1, ReleaseEvidenceKindV1, ReleaseReceiptKindV1};

use super::{Reject, reject};

const TRANSACTION_CORE_KINDS: &[ReleaseEvidenceKindV1] = &[
    ReleaseEvidenceKindV1::Candidate,
    ReleaseEvidenceKindV1::Evidence,
    ReleaseEvidenceKindV1::ProofBundle,
    ReleaseEvidenceKindV1::Effect,
    ReleaseEvidenceKindV1::Check,
    ReleaseEvidenceKindV1::Integration,
    ReleaseEvidenceKindV1::Observation,
    ReleaseEvidenceKindV1::AuditAnchor,
];

pub(super) fn validate_receipt_kind(receipt: &GateReceiptV1) -> Result<(), Reject> {
    let present = receipt
        .evidence_subjects
        .iter()
        .map(|subject| subject.subject_kind)
        .collect::<Vec<_>>();
    validate_kind_set(receipt.receipt_kind, &present)
}

fn validate_kind_set(
    receipt_kind: ReleaseReceiptKindV1,
    present: &[ReleaseEvidenceKindV1],
) -> Result<(), Reject> {
    for required in required_kind_evidence(receipt_kind) {
        let count = present.iter().filter(|kind| *kind == required).count();
        if count == 0 {
            return Err(reject(format!(
                "receipt kind {} is missing required evidence {}",
                receipt_kind_name(receipt_kind),
                evidence_kind_name(*required)
            )));
        }
        if receipt_kind == ReleaseReceiptKindV1::Transaction && count != 1 {
            return Err(reject(format!(
                "receipt kind transaction repeats required evidence {}",
                evidence_kind_name(*required)
            )));
        }
    }
    Ok(())
}

fn required_kind_evidence(kind: ReleaseReceiptKindV1) -> &'static [ReleaseEvidenceKindV1] {
    match kind {
        ReleaseReceiptKindV1::Provider => &[
            ReleaseEvidenceKindV1::Artifact,
            ReleaseEvidenceKindV1::Environment,
            ReleaseEvidenceKindV1::Policy,
            ReleaseEvidenceKindV1::Provider,
            ReleaseEvidenceKindV1::Schema,
            ReleaseEvidenceKindV1::Toolchain,
        ],
        ReleaseReceiptKindV1::ProfileClosure => &[
            ReleaseEvidenceKindV1::Artifact,
            ReleaseEvidenceKindV1::Environment,
            ReleaseEvidenceKindV1::Policy,
            ReleaseEvidenceKindV1::Schema,
            ReleaseEvidenceKindV1::Toolchain,
        ],
        ReleaseReceiptKindV1::Artifact => &[
            ReleaseEvidenceKindV1::Artifact,
            ReleaseEvidenceKindV1::Sbom,
            ReleaseEvidenceKindV1::Provenance,
        ],
        ReleaseReceiptKindV1::Containment => &[
            ReleaseEvidenceKindV1::Sandbox,
            ReleaseEvidenceKindV1::Environment,
            ReleaseEvidenceKindV1::Platform,
        ],
        ReleaseReceiptKindV1::Forge => &[
            ReleaseEvidenceKindV1::Integration,
            ReleaseEvidenceKindV1::Check,
            ReleaseEvidenceKindV1::Observation,
        ],
        ReleaseReceiptKindV1::Operations => &[
            ReleaseEvidenceKindV1::Observation,
            ReleaseEvidenceKindV1::AuditAnchor,
            ReleaseEvidenceKindV1::Configuration,
        ],
        ReleaseReceiptKindV1::RustToolchain => &[
            ReleaseEvidenceKindV1::Toolchain,
            ReleaseEvidenceKindV1::Environment,
            ReleaseEvidenceKindV1::Schema,
        ],
        ReleaseReceiptKindV1::Scanner => &[
            ReleaseEvidenceKindV1::Scanner,
            ReleaseEvidenceKindV1::Sbom,
            ReleaseEvidenceKindV1::Policy,
        ],
        ReleaseReceiptKindV1::Transaction => TRANSACTION_CORE_KINDS,
    }
}

fn receipt_kind_name(kind: ReleaseReceiptKindV1) -> &'static str {
    match kind {
        ReleaseReceiptKindV1::Artifact => "artifact",
        ReleaseReceiptKindV1::Containment => "containment",
        ReleaseReceiptKindV1::Forge => "forge",
        ReleaseReceiptKindV1::Operations => "operations",
        ReleaseReceiptKindV1::ProfileClosure => "profile-closure",
        ReleaseReceiptKindV1::Provider => "provider",
        ReleaseReceiptKindV1::RustToolchain => "rust-toolchain",
        ReleaseReceiptKindV1::Scanner => "scanner",
        ReleaseReceiptKindV1::Transaction => "transaction",
    }
}

fn evidence_kind_name(kind: ReleaseEvidenceKindV1) -> &'static str {
    match kind {
        ReleaseEvidenceKindV1::Artifact => "artifact",
        ReleaseEvidenceKindV1::AuditAnchor => "audit-anchor",
        ReleaseEvidenceKindV1::Candidate => "candidate",
        ReleaseEvidenceKindV1::Check => "check",
        ReleaseEvidenceKindV1::Configuration => "configuration",
        ReleaseEvidenceKindV1::Effect => "effect",
        ReleaseEvidenceKindV1::Environment => "environment",
        ReleaseEvidenceKindV1::Evidence => "evidence",
        ReleaseEvidenceKindV1::Integration => "integration",
        ReleaseEvidenceKindV1::Jeryu => "jeryu",
        ReleaseEvidenceKindV1::Observation => "observation",
        ReleaseEvidenceKindV1::Platform => "platform",
        ReleaseEvidenceKindV1::Policy => "policy",
        ReleaseEvidenceKindV1::ProfileGraph => "profile-graph",
        ReleaseEvidenceKindV1::ProofBundle => "proof-bundle",
        ReleaseEvidenceKindV1::Provider => "provider",
        ReleaseEvidenceKindV1::Provenance => "provenance",
        ReleaseEvidenceKindV1::Sandbox => "sandbox",
        ReleaseEvidenceKindV1::Sbom => "sbom",
        ReleaseEvidenceKindV1::Scanner => "scanner",
        ReleaseEvidenceKindV1::Schema => "schema",
        ReleaseEvidenceKindV1::Toolchain => "toolchain",
        ReleaseEvidenceKindV1::Transaction => "transaction",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_core_kinds_are_exact_singletons() {
        validate_kind_set(ReleaseReceiptKindV1::Transaction, TRANSACTION_CORE_KINDS)
            .expect("complete Transaction kind floor");

        for missing in TRANSACTION_CORE_KINDS {
            let present = TRANSACTION_CORE_KINDS
                .iter()
                .copied()
                .filter(|kind| kind != missing)
                .collect::<Vec<_>>();
            let error = validate_kind_set(ReleaseReceiptKindV1::Transaction, &present)
                .expect_err("missing Transaction core kind must refuse");
            assert!(error.detail.contains("is missing required evidence"));
        }

        for repeated in TRANSACTION_CORE_KINDS {
            let mut present = TRANSACTION_CORE_KINDS.to_vec();
            present.push(*repeated);
            let error = validate_kind_set(ReleaseReceiptKindV1::Transaction, &present)
                .expect_err("duplicate Transaction core kind must refuse");
            assert!(error.detail.contains("repeats required evidence"));
        }
    }

    #[test]
    fn wrong_kind_cannot_replace_a_transaction_core_kind() {
        let mut present = TRANSACTION_CORE_KINDS.to_vec();
        present[0] = ReleaseEvidenceKindV1::Jeryu;
        let error = validate_kind_set(ReleaseReceiptKindV1::Transaction, &present)
            .expect_err("wrong-kind substitution must refuse");
        assert_eq!(
            error.detail,
            "receipt kind transaction is missing required evidence candidate"
        );
    }
}
