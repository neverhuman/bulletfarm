//! Deterministic, non-authoritative reports for the family proof command.

use std::fmt;

use serde::Serialize;

use super::subject::RepositorySubject;

pub const CHECK_REPORT_SCHEMA_VERSION: u32 = 2;
pub const PROFILED_CHECK_REPORT_SCHEMA_VERSION: u32 = 3;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CheckTier {
    Fast,
    Required,
    Release,
}

impl CheckTier {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "FAST",
            Self::Required => "REQUIRED",
            Self::Release => "RELEASE",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GateStatus {
    Pass,
    Fail,
    Blocked,
    Neutral,
    Unknown,
}

impl GateStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Blocked => "BLOCKED",
            Self::Neutral => "NEUTRAL",
            Self::Unknown => "UNKNOWN",
        }
    }

    pub const fn exit_code(self) -> u8 {
        match self {
            Self::Pass => 0,
            Self::Fail | Self::Neutral | Self::Unknown => 1,
            Self::Blocked => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GateClass {
    Component,
    Synthetic,
    Transaction,
    Live,
    Release,
}

impl GateClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Component => "COMPONENT",
            Self::Synthetic => "SYNTHETIC",
            Self::Transaction => "TRANSACTION",
            Self::Live => "LIVE",
            Self::Release => "RELEASE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GateResult {
    id: String,
    status: GateStatus,
    class: GateClass,
    detail: String,
    repair: Option<String>,
    subjects: Vec<RepositorySubject>,
}

impl GateResult {
    fn new(
        id: impl Into<String>,
        status: GateStatus,
        class: GateClass,
        detail: impl Into<String>,
        repair: Option<String>,
    ) -> Result<Self, CheckModelError> {
        let result = Self {
            id: id.into(),
            status,
            class,
            detail: detail.into(),
            repair,
            subjects: Vec::new(),
        };
        result.validate()?;
        Ok(result)
    }

    pub fn pass(
        id: impl Into<String>,
        class: GateClass,
        detail: impl Into<String>,
    ) -> Result<Self, CheckModelError> {
        Self::new(id, GateStatus::Pass, class, detail, None)
    }

    pub fn fail(
        id: impl Into<String>,
        class: GateClass,
        detail: impl Into<String>,
        repair: impl Into<String>,
    ) -> Result<Self, CheckModelError> {
        Self::new(id, GateStatus::Fail, class, detail, Some(repair.into()))
    }

    pub fn blocked(
        id: impl Into<String>,
        class: GateClass,
        detail: impl Into<String>,
        repair: impl Into<String>,
    ) -> Result<Self, CheckModelError> {
        Self::new(id, GateStatus::Blocked, class, detail, Some(repair.into()))
    }

    pub fn unknown(
        id: impl Into<String>,
        class: GateClass,
        detail: impl Into<String>,
        repair: impl Into<String>,
    ) -> Result<Self, CheckModelError> {
        Self::new(id, GateStatus::Unknown, class, detail, Some(repair.into()))
    }

    pub fn optional_live_neutral(
        id: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, CheckModelError> {
        Self::new(id, GateStatus::Neutral, GateClass::Live, detail, None)
    }

    pub fn id(&self) -> &str {
        &self.id
    }
    pub const fn status(&self) -> GateStatus {
        self.status
    }
    pub const fn class(&self) -> GateClass {
        self.class
    }
    pub fn detail(&self) -> &str {
        &self.detail
    }
    pub fn repair(&self) -> Option<&str> {
        self.repair.as_deref()
    }

    pub(super) fn with_subjects(
        mut self,
        mut subjects: Vec<RepositorySubject>,
    ) -> Result<Self, CheckModelError> {
        subjects.sort();
        if subjects.is_empty()
            || subjects
                .windows(2)
                .any(|pair| pair[0].repository() == pair[1].repository())
        {
            return Err(CheckModelError::new(
                "INVALID_GATE_SUBJECTS",
                format!("{} has empty or duplicate subjects", self.id),
            ));
        }
        self.subjects = subjects;
        self.validate()?;
        Ok(self)
    }

    fn validate(&self) -> Result<(), CheckModelError> {
        if self.id.is_empty()
            || self.id.len() > 128
            || !self.id.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'.' | b'-' | b'_')
            })
        {
            return Err(CheckModelError::new(
                "INVALID_GATE_ID",
                "gate ID must be 1..=128 lowercase ASCII identifier bytes",
            ));
        }
        if self.detail.trim().is_empty() {
            return Err(CheckModelError::new(
                "MISSING_GATE_DETAIL",
                format!("{} has no detail", self.id),
            ));
        }
        if matches!(
            self.status,
            GateStatus::Fail | GateStatus::Blocked | GateStatus::Unknown
        ) && self
            .repair
            .as_deref()
            .is_none_or(|repair| repair.trim().is_empty())
        {
            return Err(CheckModelError::new(
                "MISSING_GATE_REPAIR",
                format!(
                    "{} is {} without repair guidance",
                    self.id,
                    self.status.as_str()
                ),
            ));
        }
        if self.status == GateStatus::Neutral && self.class != GateClass::Live {
            return Err(CheckModelError::new(
                "INVALID_NEUTRAL_GATE",
                format!("{} is NEUTRAL but is not an optional LIVE gate", self.id),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CheckReport {
    schema_version: u32,
    command: &'static str,
    tier: CheckTier,
    #[serde(skip_serializing_if = "Option::is_none")]
    profile: Option<&'static str>,
    status: GateStatus,
    gates: Vec<GateResult>,
}

impl CheckReport {
    pub fn new(tier: CheckTier, mut gates: Vec<GateResult>) -> Result<Self, CheckModelError> {
        if gates.is_empty() {
            return Err(CheckModelError::new(
                "EMPTY_CHECK_REPORT",
                "a check report must contain at least one gate",
            ));
        }
        for gate in &gates {
            gate.validate()?;
        }
        gates.sort_by(|left, right| left.id.cmp(&right.id));
        if let Some(pair) = gates.windows(2).find(|pair| pair[0].id == pair[1].id) {
            return Err(CheckModelError::new(
                "DUPLICATE_GATE_ID",
                format!("gate {} appears more than once", pair[0].id),
            ));
        }
        let status = aggregate(tier, &gates);
        Ok(Self {
            schema_version: CHECK_REPORT_SCHEMA_VERSION,
            command: "check",
            tier,
            profile: None,
            status,
            gates,
        })
    }

    pub fn for_profile(
        profile: &'static str,
        gates: Vec<GateResult>,
    ) -> Result<Self, CheckModelError> {
        if profile.is_empty() {
            return Err(CheckModelError::new(
                "MISSING_RELEASE_PROFILE",
                "profiled release report needs a profile name",
            ));
        }
        let mut report = Self::new(CheckTier::Release, gates)?;
        report.schema_version = PROFILED_CHECK_REPORT_SCHEMA_VERSION;
        report.profile = Some(profile);
        Ok(report)
    }

    pub const fn exit_code(&self) -> u8 {
        self.status.exit_code()
    }

    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }
    pub const fn command(&self) -> &'static str {
        self.command
    }

    pub const fn tier(&self) -> CheckTier {
        self.tier
    }

    pub const fn profile(&self) -> Option<&'static str> {
        self.profile
    }

    pub const fn status(&self) -> GateStatus {
        self.status
    }

    pub fn gates(&self) -> &[GateResult] {
        &self.gates
    }

    /// Stable for this field-only DTO and sorted gate list; not RFC 8785 release authority.
    pub fn stable_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn human(&self) -> String {
        let mut output = format!("check {}", self.tier.as_str());
        if let Some(profile) = self.profile {
            output.push_str(&format!(" profile {profile}"));
        }
        output.push_str(&format!(": {}", self.status.as_str()));
        for gate in &self.gates {
            output.push_str(&format!(
                "\n- [{}] {} {}: {}",
                gate.class.as_str(),
                gate.id,
                gate.status.as_str(),
                gate.detail
            ));
            if let Some(repair) = &gate.repair {
                output.push_str(&format!("\n  repair: {repair}"));
            }
        }
        output
    }
}

fn aggregate(tier: CheckTier, gates: &[GateResult]) -> GateStatus {
    if gates.iter().any(|gate| gate.status == GateStatus::Fail) {
        return GateStatus::Fail;
    }
    if gates.iter().any(|gate| gate.status == GateStatus::Unknown) {
        return GateStatus::Unknown;
    }
    if gates.iter().any(|gate| gate.status == GateStatus::Blocked) {
        return GateStatus::Blocked;
    }
    let has_neutral = gates.iter().any(|gate| gate.status == GateStatus::Neutral);
    if tier == CheckTier::Release && has_neutral {
        return GateStatus::Unknown;
    }
    if gates.iter().any(|gate| gate.status == GateStatus::Pass) {
        GateStatus::Pass
    } else {
        GateStatus::Neutral
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckModelError {
    code: &'static str,
    reason: String,
}

impl CheckModelError {
    pub(super) fn new(code: &'static str, reason: impl Into<String>) -> Self {
        Self {
            code,
            reason: reason.into(),
        }
    }

    pub const fn code(&self) -> &'static str {
        self.code
    }
}

impl fmt::Display for CheckModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.reason)
    }
}

impl std::error::Error for CheckModelError {}

#[cfg(test)]
#[path = "model_tests.rs"]
mod tests;
