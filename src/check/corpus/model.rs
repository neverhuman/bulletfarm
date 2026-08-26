//! Strict data model for `policy/corpus-coverage-v1.json`.
//!
//! One row per normative unit of the historical vision corpus. Every row
//! carries exactly one disposition; unknown fields, unknown dispositions,
//! and anchor shapes that do not match the disposition are rejected.

use serde::{Deserialize, Serialize};

/// Exact schema identifier the policy file must declare.
pub const SCHEMA: &str = "bullet-farm.corpus-coverage.v1";

/// The seven corpus documents, in page order. Keys are fixed by contract.
pub const CORPUS_KEYS: [&str; 7] = [
    "spec",
    "git_role",
    "gastown",
    "nightshift",
    "potential",
    "paper",
    "evo",
];

/// Repositories an anchor may point into.
pub const REPOS: [&str; 4] = [
    "bullet-farm",
    "bullet-kernel",
    "bullet-git",
    "bullet-portal",
];

/// Highest roadmap wave a `PLANNED` row may cite.
pub const MAX_WAVE: u8 = 11;

pub const MAX_ID_LEN: usize = 64;
pub const MAX_UNIT_LEN: usize = 120;
pub const MAX_NOTE_LEN: usize = 160;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusCoverageSpec {
    pub schema: String,
    pub corpus: Vec<CorpusDocument>,
    pub units: Vec<CorpusUnit>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusDocument {
    pub key: String,
    /// Family-root-relative path of the historical document.
    pub path: String,
    pub title: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CorpusUnit {
    pub id: String,
    pub doc: String,
    #[serde(rename = "ref")]
    pub reference: String,
    pub unit: String,
    pub disposition: Disposition,
    pub anchor: Anchor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial: Option<Anchor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum Disposition {
    /// Code exists and a named test in a proof lane proves it.
    Implemented,
    /// Not (fully) implemented; owned by a roadmap wave.
    Planned,
    /// Replaced by a reviewed decision (ADR).
    Superseded,
    /// Rejected by a reviewed decision (ADR).
    Refused,
}

impl Disposition {
    pub const ALL: [Self; 4] = [
        Self::Implemented,
        Self::Planned,
        Self::Superseded,
        Self::Refused,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Implemented => "IMPLEMENTED",
            Self::Planned => "PLANNED",
            Self::Superseded => "SUPERSEDED",
            Self::Refused => "REFUSED",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Anchor {
    /// A test function (Rust) or test title (TypeScript) in a file.
    Test {
        repo: String,
        path: String,
        symbol: String,
    },
    /// A non-test symbol in a file.
    Symbol {
        repo: String,
        path: String,
        symbol: String,
    },
    /// A closure-roadmap wave, `W0`…`W11`.
    Wave { value: String },
    /// An ADR file name under `docs/decisions/`.
    Adr { value: String },
}

impl Anchor {
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Test { .. } => "test",
            Self::Symbol { .. } => "symbol",
            Self::Wave { .. } => "wave",
            Self::Adr { .. } => "adr",
        }
    }

    pub const fn is_code(&self) -> bool {
        matches!(self, Self::Test { .. } | Self::Symbol { .. })
    }

    /// Short, deterministic rendering for the generated page.
    pub fn render(&self) -> String {
        match self {
            Self::Test { repo, path, symbol } => format!("test `{repo}/{path}::{symbol}`"),
            Self::Symbol { repo, path, symbol } => format!("symbol `{repo}/{path}::{symbol}`"),
            Self::Wave { value } => format!("wave `{value}`"),
            Self::Adr { value } => format!("ADR `{value}`"),
        }
    }
}

/// Parse a wave label of the form `W<n>` with `0 <= n <= MAX_WAVE`.
pub fn parse_wave(value: &str) -> Option<u8> {
    let digits = value.strip_prefix('W')?;
    if digits.is_empty() || digits.len() > 2 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if digits.len() == 2 && digits.starts_with('0') {
        return None;
    }
    let wave = digits.bytes().fold(0u8, |acc, b| {
        acc.saturating_mul(10).saturating_add(b - b'0')
    });
    (wave <= MAX_WAVE).then_some(wave)
}
