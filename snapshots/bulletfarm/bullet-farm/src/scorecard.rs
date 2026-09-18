//! Generated D2 scorecard. Admitted rows add typed implemented deltas.
//! This module does not make a release gate green.

mod admit;
mod spec;

use crate::coord::CoordError;
use admit::admit_row;
use serde::Serialize;
use spec::{ScorecardSpec, load_spec};
use std::path::Path;

pub use spec::{
    Blend, CriterionRow, DimensionSpec, EvidenceKind, EvidenceReference, ScorecardSpec as Spec,
    ScorecardStatus,
};

/// Scored instrument. Never a release receipt.
#[derive(Clone, Debug, Serialize)]
pub struct ScorecardReport {
    /// Rubric id.
    pub rubric: String,
    /// Always false for `scorecard-v1`.
    pub authoritative: bool,
    /// Exact classification of the published number.
    pub status: ScorecardStatus,
    /// Weighted design score.
    pub design: f64,
    /// Weighted implemented score (floors + admitted rows).
    pub implemented: f64,
    /// Stranger-usable score.
    pub stranger: f64,
    /// Architecture score.
    pub architecture: f64,
    /// 20/60/20 blend, rounded to one decimal.
    pub blended: f64,
    /// Per-dimension implemented scores.
    pub dimensions: Vec<DimensionScore>,
    /// Rows with whether evidence was admitted.
    pub rows: Vec<RowScore>,
}

/// One dimension after scoring.
#[derive(Clone, Debug, Serialize)]
pub struct DimensionScore {
    /// Dimension number.
    pub id: u8,
    /// Name.
    pub name: String,
    /// Design score.
    pub design: u8,
    /// Implemented score used in the blend.
    pub implemented: u8,
}

/// One row after scoring.
#[derive(Clone, Debug, Serialize)]
pub struct RowScore {
    /// Row id.
    pub id: String,
    /// Whether the semantic verifier admitted the subject.
    pub admitted: bool,
    /// Stable reason the row contributes no evidence, or `-` when admitted.
    pub refusal_reason: String,
    /// Claim.
    pub claim: String,
}

/// Load and score the rubric. Missing or refuted evidence keeps the floor.
///
/// # Errors
///
/// Rubric missing, invalid JSON, or blend weights that do not sum to 1.
pub fn evaluate(hub: &Path) -> Result<ScorecardReport, CoordError> {
    let spec = load_spec(hub)?;
    let blend_sum = spec.blend.architecture + spec.blend.implemented + spec.blend.stranger;
    if (blend_sum - 1.0).abs() > 0.001 {
        return Err(CoordError::new(
            "SCORECARD_BLEND",
            format!("blend weights sum to {blend_sum}, not 1.0"),
        ));
    }
    let rows: Vec<RowScore> = spec.rows.iter().map(|row| admit_row(hub, row)).collect();
    Ok(score(&spec, rows))
}

fn score(spec: &ScorecardSpec, rows: Vec<RowScore>) -> ScorecardReport {
    let mut extras = [0_u16; 13];
    for (row, scored) in spec.rows.iter().zip(&rows) {
        if scored.admitted {
            extras[usize::from(row.dimension)] += u16::from(row.implemented_delta);
        }
    }
    let dimensions: Vec<DimensionScore> = spec
        .dimensions
        .iter()
        .map(|dim| {
            let raised = u16::from(dim.implemented_floor) + extras[usize::from(dim.id)];
            DimensionScore {
                id: dim.id,
                name: dim.name.clone(),
                design: dim.design,
                implemented: raised.min(u16::from(dim.design)) as u8,
            }
        })
        .collect();
    let weight_sum: f64 = spec.dimensions.iter().map(|d| f64::from(d.weight)).sum();
    let design = spec
        .dimensions
        .iter()
        .map(|d| f64::from(d.design) * f64::from(d.weight))
        .sum::<f64>()
        / weight_sum;
    let implemented = dimensions
        .iter()
        .zip(&spec.dimensions)
        .map(|(scored, dim)| f64::from(scored.implemented) * f64::from(dim.weight))
        .sum::<f64>()
        / weight_sum;
    let architecture = spec.architecture_implemented_floor;
    let stranger = spec.stranger_implemented_floor;
    let blended = spec.blend.architecture * architecture
        + spec.blend.implemented * implemented
        + spec.blend.stranger * stranger;
    ScorecardReport {
        rubric: spec.rubric.clone(),
        authoritative: false,
        status: spec.status,
        design: round1(design),
        implemented: round1(implemented),
        stranger,
        architecture,
        blended: round1(blended),
        dimensions,
        rows,
    }
}

/// Render the portable markdown page.
#[must_use]
pub fn render_markdown(report: &ScorecardReport) -> String {
    let mut out = String::from("# Scorecard (generated)\n\n");
    out.push_str("Status: **instrumented estimate; not release authority.**\n\n");
    out.push_str(&format!(
        "Rubric `{}`. Blended **{}** (architecture {}, implemented {}, stranger {}). Frozen baseline was **43.3**.\n\n",
        report.rubric, report.blended, report.architecture, report.implemented, report.stranger
    ));
    out.push_str("| # | Dimension | Design | Implemented |\n| --- | --- | ---: | ---: |\n");
    for dim in &report.dimensions {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            dim.id, dim.name, dim.design, dim.implemented
        ));
    }
    out.push_str("\n| Row | Admitted | Refusal | Claim |\n| --- | --- | --- | --- |\n");
    for row in &report.rows {
        out.push_str(&format!(
            "| `{}` | {} | `{}` | {} |\n",
            row.id,
            if row.admitted { "yes" } else { "no" },
            row.refusal_reason,
            row.claim
        ));
    }
    out.push_str(
        "\nA row adds its typed implemented delta only when the kind-specific verifier re-derives the claim from committed Hub bytes or an exact pinned family subject. Mutable sibling checkouts, file presence, unsigned receipts, ignored tests, and `check release` gates do not admit. This page is not release authority.\n",
    );
    out
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod strict_json_tests {
    use super::evaluate;
    use super::spec::decode_spec;

    #[test]
    fn scorecard_refuses_precision_losing_and_duplicate_members() {
        let baseline = include_str!("../policy/scorecard-v1.json");
        let precision = baseline.replacen(
            "\"architecture\": 0.2",
            "\"architecture\": 0.20000000000000001",
            1,
        );
        assert_ne!(
            baseline, precision,
            "hostile lexeme must replace the baseline"
        );
        let error = decode_spec(precision.as_bytes())
            .expect_err("a precision-losing blend lexeme must fail closed");
        assert_eq!(error.code(), "SCORECARD_SCHEMA");
        assert!(error.to_string().contains("JSON_NUMBER_PRECISION_LOSS"));

        let duplicate = baseline.replacen(
            "\"architecture\": 0.2,",
            "\"architecture\": 0.2, \"architecture\": 0.2,",
            1,
        );
        let error = decode_spec(duplicate.as_bytes())
            .expect_err("a duplicate blend member must fail closed");
        assert!(error.to_string().contains("DUPLICATE_JSON_KEY"));

        let directory = tempfile::tempdir().expect("oversized scorecard root");
        let policy = directory.path().join("policy");
        std::fs::create_dir(&policy).expect("scorecard policy directory");
        std::fs::File::create(policy.join("scorecard-v1.json"))
            .expect("oversized scorecard")
            .set_len(bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES as u64 + 1)
            .expect("size oversized scorecard");
        let error = evaluate(directory.path()).expect_err("oversized scorecard must fail closed");
        assert_eq!(error.code(), "SCORECARD_SCHEMA");
        assert!(error.to_string().contains("DOCUMENT_TOO_LARGE"));
    }
}
