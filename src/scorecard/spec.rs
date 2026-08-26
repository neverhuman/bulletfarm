//! Typed scorecard rubric. Dimensions, blend, and floors stay frozen.

use crate::coord::CoordError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;

pub(super) const EXPECTED_BLEND: (f64, f64, f64) = (0.2, 0.6, 0.2);
pub(super) const EXPECTED_BASELINES: (f64, f64, f64, f64) = (94.5, 94.5, 100.0, 3.0);
const EXPECTED_WEIGHT_SUM: u16 = 100;
pub(super) const EXPECTED_DIMENSIONS: [(u8, &str, u8, u8, u8); 12] = [
    (1, "Concurrency and authority kernel", 14, 95, 64),
    (2, "Isolation and repository safety", 11, 96, 78),
    (3, "Evidence and verification integrity", 12, 94, 28),
    (4, "Integration and delivery authority", 11, 92, 26),
    (5, "Identity, quota and cost governance", 8, 90, 8),
    (6, "Multi-agent collaboration and roles", 8, 88, 6),
    (7, "Evolutionary optimization", 5, 86, 2),
    (8, "Operator truth and UX", 7, 90, 48),
    (9, "Installability and release engineering", 8, 94, 16),
    (10, "Security posture", 7, 92, 58),
    (11, "Test and assurance depth", 6, 90, 57),
    (12, "Documentation honesty", 3, 94, 86),
];
pub(super) const EXPECTED_ROWS: [(&str, u8, EvidenceKind, &str, u8); 15] = [
    (
        "d1.nonce-ledger",
        1,
        EvidenceKind::CiTest,
        "Durable nonce issue/consume separated",
        8,
    ),
    (
        "d1.signed-transport",
        1,
        EvidenceKind::Receipt,
        "Signed lease transport mounted internally",
        8,
    ),
    (
        "d2.egress-ci",
        2,
        EvidenceKind::CiTest,
        "Three egress proofs run every push",
        6,
    ),
    (
        "d3.proof-root-eight",
        3,
        EvidenceKind::CiTest,
        "ProofRoot over eight inputs with tamper tests",
        12,
    ),
    (
        "d4.attestor",
        4,
        EvidenceKind::Receipt,
        "Attestor binary posts exact-SHA checks",
        8,
    ),
    (
        "d4.jeryu-live",
        4,
        EvidenceKind::Gate,
        "release.forge.jeryu admitted",
        10,
    ),
    (
        "d5.budgets",
        5,
        EvidenceKind::CiTest,
        "Atomic dual-tree reservation/settlement",
        10,
    ),
    (
        "d6.two-providers",
        6,
        EvidenceKind::Receipt,
        "Two providers dispatch through the router",
        12,
    ),
    (
        "d7.evolution-off",
        7,
        EvidenceKind::CiTest,
        "evolutionary_authority remains false until OD-H",
        6,
    ),
    (
        "d8.fifteen-surfaces",
        8,
        EvidenceKind::CiTest,
        "Fifteen portal surfaces render durable subjects",
        10,
    ),
    (
        "d9.schema-3",
        9,
        EvidenceKind::Gate,
        "release.installable-lock admitted",
        20,
    ),
    (
        "d10.jankurai-90",
        10,
        EvidenceKind::Gate,
        "release.jankurai-90 admitted",
        8,
    ),
    (
        "d11.invariants-51",
        11,
        EvidenceKind::CiTest,
        "51/51 invariants enforced",
        8,
    ),
    (
        "d12.signed-jeryu-tags",
        12,
        EvidenceKind::SourceReceipt,
        "Jeryu tags are annotated and signed",
        4,
    ),
    (
        "g2.transaction-proof",
        1,
        EvidenceKind::Gate,
        "release.transaction-demo admitted",
        8,
    ),
];

/// Frozen rubric plus one evidence row per exit criterion.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScorecardSpec {
    /// Document identity.
    pub schema_version: String,
    /// Named rubric (`d2-v1`).
    pub rubric: String,
    /// Live instrument after S-00; frozen-baseline is refused.
    pub status: ScorecardStatus,
    /// True once every row has a typed implemented delta.
    pub criterion_inventory_complete: bool,
    /// 20/60/20 blend.
    pub blend: Blend,
    /// Twelve product dimensions.
    pub dimensions: Vec<DimensionSpec>,
    /// Architecture design score.
    pub architecture_design: f64,
    /// Architecture implemented floor until evidence lands.
    pub architecture_implemented_floor: f64,
    /// Stranger-usable design score.
    pub stranger_design: f64,
    /// Stranger-usable implemented floor.
    pub stranger_implemented_floor: f64,
    /// Exit-criterion rows.
    pub rows: Vec<CriterionRow>,
}

/// Instrument classification.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ScorecardStatus {
    /// Retired lock that forbade admission.
    FrozenBaseline,
    /// Semantic verifier is connected; floors still hold without admission.
    Instrumented,
}

/// Blend weights. Must sum to 1.0.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Blend {
    /// Architecture share.
    pub architecture: f64,
    /// Implemented-product share.
    pub implemented: f64,
    /// Stranger-usable share.
    pub stranger: f64,
}

/// One D2 dimension.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DimensionSpec {
    /// Dimension number 1..=12.
    pub id: u8,
    /// Display name.
    pub name: String,
    /// D2 weight.
    pub weight: u8,
    /// Design-as-specified score.
    pub design: u8,
    /// Implemented floor when no row has evidence.
    pub implemented_floor: u8,
}

/// One exit criterion.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceKind {
    /// Current-family CI observation catalog key.
    CiTest,
    /// Signed component or transaction receipt.
    Receipt,
    /// Semantically verified release-profile gate.
    Gate,
    /// Signed source/tag/immutability receipt.
    SourceReceipt,
    /// Independently signed external review or stranger trial.
    ExternalReview,
}

/// Closed evidence reference. Presence is not admission.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "source", rename_all = "kebab-case", deny_unknown_fields)]
pub enum EvidenceReference {
    /// Catalog key for a kind-specific CI predicate.
    CiObservation { subject_id: String },
    /// Verified release gate under one explicit profile.
    ReleaseGate {
        gate_id: String,
        profile_id: String,
        receipt_id: String,
    },
    /// Signed typed receipt.
    SignedReceipt { receipt_id: String },
    /// Signed source/tag/immutability receipt.
    SourceReceipt { receipt_id: String },
    /// Signed independent external report.
    ExternalReview { receipt_id: String },
}

/// One exit criterion.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CriterionRow {
    /// Stable row id.
    pub id: String,
    /// Owning dimension.
    pub dimension: u8,
    /// Evidence kind.
    pub kind: EvidenceKind,
    /// Claim text.
    pub claim: String,
    /// Implemented points awarded only after semantic admission.
    pub implemented_delta: u8,
    /// Typed reference. `null` admits nothing.
    pub evidence: Option<EvidenceReference>,
}

pub(super) fn load_spec(hub: &Path) -> Result<ScorecardSpec, CoordError> {
    let path = hub.join("policy/scorecard-v1.json");
    let file = std::fs::File::open(&path).map_err(CoordError::io)?;
    let mut bytes = Vec::new();
    file.take((bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(CoordError::io)?;
    let spec = decode_spec(&bytes)?;
    if spec.schema_version != "scorecard-v1" {
        return Err(CoordError::new(
            "SCORECARD_SCHEMA",
            format!("unsupported scorecard schema {}", spec.schema_version),
        ));
    }
    validate_spec(&spec)?;
    Ok(spec)
}

pub(super) fn decode_spec(bytes: &[u8]) -> Result<ScorecardSpec, CoordError> {
    let value = bullet_wire::decode_unique_value(bytes)
        .map_err(|error| scorecard_error(format!("scorecard-v1 is not strict JSON: {error}")))?;
    serde_json::from_value(value).map_err(|error| {
        scorecard_error(format!(
            "scorecard-v1 does not match its typed schema: {error}"
        ))
    })
}

pub(super) fn validate_spec(spec: &ScorecardSpec) -> Result<(), CoordError> {
    if spec.rubric != "d2-v1"
        || spec.status != ScorecardStatus::Instrumented
        || !spec.criterion_inventory_complete
    {
        return Err(scorecard_error(
            "scorecard-v1 must be an instrumented rubric with a complete criterion inventory",
        ));
    }
    if (
        spec.blend.architecture.to_bits(),
        spec.blend.implemented.to_bits(),
        spec.blend.stranger.to_bits(),
    ) != (
        EXPECTED_BLEND.0.to_bits(),
        EXPECTED_BLEND.1.to_bits(),
        EXPECTED_BLEND.2.to_bits(),
    ) || (
        spec.architecture_design.to_bits(),
        spec.architecture_implemented_floor.to_bits(),
        spec.stranger_design.to_bits(),
        spec.stranger_implemented_floor.to_bits(),
    ) != (
        EXPECTED_BASELINES.0.to_bits(),
        EXPECTED_BASELINES.1.to_bits(),
        EXPECTED_BASELINES.2.to_bits(),
        EXPECTED_BASELINES.3.to_bits(),
    ) {
        return Err(scorecard_error(
            "scorecard-v1 blend and baseline values differ from the frozen instrument",
        ));
    }
    for (name, value) in [
        ("architecture", spec.blend.architecture),
        ("implemented", spec.blend.implemented),
        ("stranger", spec.blend.stranger),
    ] {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(scorecard_error(format!(
                "blend component {name} is outside 0..=1"
            )));
        }
    }
    for (name, value) in [
        ("architecture_design", spec.architecture_design),
        (
            "architecture_implemented_floor",
            spec.architecture_implemented_floor,
        ),
        ("stranger_design", spec.stranger_design),
        (
            "stranger_implemented_floor",
            spec.stranger_implemented_floor,
        ),
    ] {
        if !value.is_finite() || !(0.0..=100.0).contains(&value) {
            return Err(scorecard_error(format!(
                "baseline score {name} is outside 0..=100"
            )));
        }
    }
    if spec.dimensions.len() != EXPECTED_DIMENSIONS.len() {
        return Err(scorecard_error(
            "scorecard-v1 requires exactly 12 dimensions",
        ));
    }
    let mut dimension_ids = BTreeSet::new();
    let mut weight_sum = 0_u16;
    let mut headroom = [0_u16; 13];
    for (dimension, expected) in spec.dimensions.iter().zip(EXPECTED_DIMENSIONS) {
        if !(1..=12).contains(&dimension.id)
            || !dimension_ids.insert(dimension.id)
            || dimension.name.trim().is_empty()
            || dimension.design > 100
            || dimension.implemented_floor > 100
            || (
                dimension.id,
                dimension.name.as_str(),
                dimension.weight,
                dimension.design,
                dimension.implemented_floor,
            ) != expected
        {
            return Err(scorecard_error(format!(
                "dimension {} has an invalid or duplicate identity, name, or score",
                dimension.id
            )));
        }
        headroom[usize::from(dimension.id)] =
            u16::from(dimension.design.saturating_sub(dimension.implemented_floor));
        weight_sum = weight_sum
            .checked_add(u16::from(dimension.weight))
            .ok_or_else(|| scorecard_error("dimension weight sum overflowed"))?;
    }
    if weight_sum != EXPECTED_WEIGHT_SUM {
        return Err(scorecard_error(format!(
            "dimension weights sum to {weight_sum}, not {EXPECTED_WEIGHT_SUM}"
        )));
    }

    let mut row_ids = BTreeSet::new();
    if spec.rows.len() != EXPECTED_ROWS.len() {
        return Err(scorecard_error(
            "scorecard-v1 row inventory differs from the frozen diagnostic inventory",
        ));
    }
    let mut allocated = [0_u16; 13];
    for (row, expected) in spec.rows.iter().zip(EXPECTED_ROWS) {
        if !dimension_ids.contains(&row.dimension)
            || row.claim.trim().is_empty()
            || !row_ids.insert(row.id.as_str())
            || (
                row.id.as_str(),
                row.dimension,
                row.kind,
                row.claim.as_str(),
                row.implemented_delta,
            ) != expected
        {
            return Err(scorecard_error(format!(
                "row {} has an invalid dimension, claim, delta, or duplicate identity",
                row.id
            )));
        }
        allocated[usize::from(row.dimension)] = allocated[usize::from(row.dimension)]
            .checked_add(u16::from(row.implemented_delta))
            .ok_or_else(|| scorecard_error("implemented delta overflowed"))?;
        if let Some(reference) = &row.evidence {
            validate_reference(row.kind, reference)?;
        }
    }
    for id in 1..=12 {
        if allocated[id] > headroom[id] {
            return Err(scorecard_error(format!(
                "dimension {id} implemented deltas exceed design headroom"
            )));
        }
    }
    Ok(())
}

fn validate_reference(kind: EvidenceKind, reference: &EvidenceReference) -> Result<(), CoordError> {
    let compatible = matches!(
        (kind, reference),
        (
            EvidenceKind::CiTest,
            EvidenceReference::CiObservation { .. }
        ) | (EvidenceKind::Gate, EvidenceReference::ReleaseGate { .. })
            | (
                EvidenceKind::Receipt,
                EvidenceReference::SignedReceipt { .. }
            )
            | (
                EvidenceKind::SourceReceipt,
                EvidenceReference::SourceReceipt { .. }
            )
            | (
                EvidenceKind::ExternalReview,
                EvidenceReference::ExternalReview { .. }
            )
    );
    if !compatible {
        return Err(scorecard_error(
            "scorecard evidence reference does not match its row kind",
        ));
    }
    let fields: Vec<&str> = match reference {
        EvidenceReference::CiObservation { subject_id } => vec![subject_id],
        EvidenceReference::ReleaseGate {
            gate_id,
            profile_id,
            receipt_id,
        } => vec![gate_id, profile_id, receipt_id],
        EvidenceReference::SignedReceipt { receipt_id }
        | EvidenceReference::SourceReceipt { receipt_id }
        | EvidenceReference::ExternalReview { receipt_id } => vec![receipt_id],
    };
    if fields.iter().any(|value| !safe_subject(value)) {
        return Err(scorecard_error(
            "scorecard evidence identifiers must be bounded non-path ASCII subjects",
        ));
    }
    Ok(())
}

fn safe_subject(value: &str) -> bool {
    (3..=160).contains(&value.len())
        && !value.starts_with('.')
        && !value.contains("..")
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'-' | b'_' | b'.' | b':')
        })
}

pub(super) fn scorecard_error(message: impl Into<String>) -> CoordError {
    CoordError::new("SCORECARD_SCHEMA", message.into())
}
