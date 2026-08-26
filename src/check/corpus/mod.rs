//! Corpus coverage: the machine-checked disposition inventory over the
//! historical vision corpus (`docs/spec/*`, `git_role.md`, the Gas Town
//! audit, nightshift, POTENTIAL_DRAFT, the paper, evolutionary-control).
//!
//! The policy is `policy/corpus-coverage-v1.json`; the generated page is
//! `docs/assurance/corpus-coverage.generated.md`. This module never makes a
//! release gate green: it only proves that every corpus unit carries one
//! reviewed disposition whose anchor exists at HEAD.

pub mod anchors;
pub mod model;
pub mod render;
pub mod validate;

pub use anchors::{Resolution, resolve};
pub use model::{Anchor, CorpusCoverageSpec, CorpusUnit, Disposition};
pub use render::{DocSummary, render, summarize};
pub use validate::validate;

use crate::coord::CoordError;
use std::path::Path;

pub const POLICY_PATH: &str = "policy/corpus-coverage-v1.json";
pub const PAGE_PATH: &str = "docs/assurance/corpus-coverage.generated.md";
pub const CODE_DRIFT: &str = "CORPUS_COVERAGE_DRIFT";
pub const CODE_ANCHOR: &str = "CORPUS_COVERAGE_ANCHOR";

/// Parse policy bytes strictly (unknown fields rejected).
pub fn parse(bytes: &[u8]) -> Result<CorpusCoverageSpec, CoordError> {
    let value = bullet_wire::decode_unique_value(bytes).map_err(|error| {
        CoordError::new(
            validate::CODE_SCHEMA,
            format!("{POLICY_PATH} is not strict JSON: {error}"),
        )
    })?;
    serde_json::from_value(value)
        .map_err(|error| CoordError::new(validate::CODE_SCHEMA, format!("{POLICY_PATH}: {error}")))
}

/// Load and structurally validate the hub's committed policy.
pub fn load(hub: &Path) -> Result<CorpusCoverageSpec, CoordError> {
    let bytes = std::fs::read(hub.join(POLICY_PATH)).map_err(CoordError::io)?;
    let spec = parse(&bytes)?;
    validate(&spec)?;
    Ok(spec)
}

/// Render the page from the committed policy and compare it byte-for-byte
/// with the committed page. Drift is a typed failure.
pub fn check_page(hub: &Path) -> Result<CorpusCoverageSpec, CoordError> {
    let spec = load(hub)?;
    let expected = render(&spec);
    let committed = std::fs::read_to_string(hub.join(PAGE_PATH)).map_err(|error| {
        CoordError::new(
            CODE_DRIFT,
            format!("{PAGE_PATH} is unreadable ({error}); regenerate with scripts/corpus-coverage.sh write"),
        )
    })?;
    if committed != expected {
        return Err(CoordError::new(
            CODE_DRIFT,
            format!(
                "{PAGE_PATH} differs from the page rendered from {POLICY_PATH}; regenerate with scripts/corpus-coverage.sh write"
            ),
        ));
    }
    Ok(spec)
}

/// Resolve anchors and fail closed on any unresolved anchor. Anchors into an
/// absent sibling checkout are reported, not failed, so a hub-only checkout
/// stays honest about what it could not see.
pub fn check_anchors(
    family_root: &Path,
    spec: &CorpusCoverageSpec,
) -> Result<Resolution, CoordError> {
    let resolution = resolve(family_root, spec);
    if let Some((id, reason)) = resolution.unresolved.first() {
        return Err(CoordError::new(
            CODE_ANCHOR,
            format!(
                "{} anchor(s) do not resolve; first: {id}: {reason}",
                resolution.unresolved.len()
            ),
        ));
    }
    Ok(resolution)
}
