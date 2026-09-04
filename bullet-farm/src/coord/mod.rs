mod anonymous_link;
mod fresh_genesis;
pub(crate) use fresh_genesis::consume_wave0_and_inventory;
mod generation;
#[allow(
    unfulfilled_lint_expectations,
    reason = "recovered W0 now consumes the formerly component-only Git observer"
)]
mod git;
mod model;
mod receipt_state;
mod recovered_wave0;
mod recovery;
mod recovery_adoption_verify;
pub(crate) mod recovery_manifest;
#[allow(
    unused_imports,
    reason = "COMPONENT_ONLY recovered W0 facts await their signed review consumer"
)]
pub(crate) use recovered_wave0::{RecoveredWave0FactsV1, observe_recovered_wave0};
pub(crate) mod sealed;
mod state;
mod store;

use std::{
    ffi::OsString,
    fmt,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub use model::{
    Applied, COORD_SCHEMA_VERSION, ClaimState, ClaimSummary, CommandReceipt, DEFAULT_TTL_SECONDS,
    ForensicRecordRefV1, GenerationId, GenesisInput, MutationEnvelope,
    RECOVERY_ADOPTION_REQUEST_SCHEMA_VERSION, RECOVERY_PRODUCTION_SCHEMA_VERSION,
    RecoveryAdoptionAuthorityClassV1, RecoveryAdoptionClaimV1, RecoveryAdoptionRequestKindV1,
    RecoveryAdoptionSummaryV1, RecoveryAdoptionWatermarkV1, RecoveryForensicArtifactKindV1,
    RecoveryForensicRecordKindV1, RecoveryGenerationRecordKindV1, RecoveryGenerationRecordRefV1,
    RecoveryGitExpectationV1, RecoveryGitLeafStatusV1, RecoveryGitLeafTransitionV1,
    RecoveryGitObjectFormatV1, RecoveryProductionPlanKindV1, RecoveryProductionPlanV1,
    RecoveryProductionSubjectV1, RecoveryProductionWatermarkV1, RecoveryProofObservationV1,
    RecoveryProofRequestKindV1, RecoveryProofRequestV1, RecoveryProofRoleV1,
    RecoveryReceiptAdoptionRequestV1, RecoveryReceiptAdoptionSubjectV1,
    RecoveryReviewApprovalKindV1, RecoveryReviewApprovalV1, RecoveryReviewDecisionV1,
    RecoveryReviewObservationV1, RecoveryReviewRequestKindV1, RecoveryReviewRequestV1,
    RecoveryReviewRoleV1, RequestId, Status, StatusOrigin, Watermark,
};
pub(crate) use model::{
    RecoveryAuthorizationSignatureV1, RecoveryAuthorizationV1, RecoveryBootstrapProvenanceV1,
    RecoveryInspectionV1,
};
pub(crate) use recovery::{RecoveryCommand, RecoveryExecution};
pub use store::CoordStore;

#[derive(Debug)]
pub struct CoordError {
    code: &'static str,
    purpose: &'static str,
    reason: String,
    common_fixes: &'static [&'static str],
    docs_url: &'static str,
    repair_hint: &'static str,
}

impl CoordError {
    pub fn new(code: &'static str, reason: impl Into<String>) -> Self {
        let repair = repair_metadata(code);
        Self {
            code,
            purpose: repair.purpose,
            reason: reason.into(),
            common_fixes: repair.common_fixes,
            docs_url: repair.docs_url,
            repair_hint: repair.repair_hint,
        }
    }

    pub fn io(error: std::io::Error) -> Self {
        Self::new("COORD_IO_FAILED", error.to_string())
    }

    pub fn json(error: serde_json::Error) -> Self {
        Self::new("COORD_JSON_FAILED", error.to_string())
    }

    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Operation the failed boundary was protecting.
    pub const fn purpose(&self) -> &'static str {
        self.purpose
    }

    /// Ordered, bounded repair choices for this error class.
    pub const fn common_fixes(&self) -> &'static [&'static str] {
        self.common_fixes
    }

    /// Repository-relative operator documentation.
    pub const fn docs_url(&self) -> &'static str {
        self.docs_url
    }

    /// Narrow next diagnostic or proof step.
    pub const fn repair_hint(&self) -> &'static str {
        self.repair_hint
    }

    pub fn exit_code(&self) -> u8 {
        match self.code {
            "CLAIM_OVERLAP"
            | "CLAIM_NOT_ACTIVE"
            | "CLAIM_OWNER_MISMATCH"
            | "COORD_RECOVERY_WRITER_WAIT" => 3,
            "CORRUPT_COORD_LOG" | "UNSUPPORTED_SCHEMA" | "PARTIAL_COORD_WRITE" => 4,
            _ => 2,
        }
    }
}

struct RepairMetadata {
    purpose: &'static str,
    common_fixes: &'static [&'static str],
    docs_url: &'static str,
    repair_hint: &'static str,
}

fn repair_metadata(code: &str) -> RepairMetadata {
    if code == "COORD_RECOVERY_WRITER_WAIT" {
        return RepairMetadata {
            purpose: "preserve a live legacy writer fence",
            common_fixes: &[
                "stop the exact legacy writer without replacing the recovery subject",
                "retry the same supervised recovery command after descriptor read-back",
            ],
            docs_url: "docs/errors.md#outcome-unknown",
            repair_hint: "keep recovery frozen and inspect legacy writable descriptors",
        };
    }
    if code.contains("TIMEOUT") || code.ends_with("_UNKNOWN") {
        return RepairMetadata {
            purpose: "preserve an ambiguous mutation outcome",
            common_fixes: &[
                "reconcile by the original request and desired-state identity",
                "do not dispatch a second write or switch providers",
            ],
            docs_url: "docs/errors.md#outcome-unknown",
            repair_hint: "run authoritative read-back and keep the subject frozen",
        };
    }
    if code.contains("CORRUPT") || code.contains("PARTIAL_") {
        return RepairMetadata {
            purpose: "stop replay of incomplete or corrupt durable state",
            common_fixes: &[
                "preserve the bytes before cleanup",
                "restore verified authority state or rebuild only generated projections",
            ],
            docs_url: "docs/errors.md#unsupported-or-corrupt-state",
            repair_hint: "run the owning integrity and recovery proof before mutation",
        };
    }
    if code.contains("UNSUPPORTED_SCHEMA") || code.contains("SCHEMA_MISMATCH") {
        return RepairMetadata {
            purpose: "reject a schema outside the admitted version set",
            common_fixes: &[
                "follow the explicit export or removal procedure",
                "use the binary and lock version that own the subject",
            ],
            docs_url: "docs/errors.md#unsupported-or-corrupt-state",
            repair_hint: "follow the schema-removal or setup-recovery runbook",
        };
    }
    if code.contains("CONFLICT")
        || code.contains("OVERLAP")
        || code.contains("MISMATCH")
        || code.contains("_CHANGED")
        || code.contains("DIRTY_")
        || code.ends_with("_EXISTS")
        || code.ends_with("_NOT_ACTIVE")
    {
        return RepairMetadata {
            purpose: "preserve one-writer and exact-subject transaction boundaries",
            common_fixes: &[
                "read back current state before issuing a new command",
                "preserve foreign bytes and choose a new subject only when intent changed",
            ],
            docs_url: "docs/errors.md#conflict-or-changed-subject",
            repair_hint: "inspect coordinator status and the exact current subject",
        };
    }
    if code.contains("UNAVAILABLE")
        || code.contains("_MISSING")
        || code.ends_with("_NOT_FOUND")
        || code.ends_with("_UNPROBED")
    {
        return RepairMetadata {
            purpose: "refuse substitution for an unavailable exact prerequisite",
            common_fixes: &[
                "run bullet-family doctor --json",
                "provision the pinned prerequisite without using ambient authority",
            ],
            docs_url: "docs/errors.md#dependency-unavailable",
            repair_hint: "rerun doctor and the narrow lane named by the failed prerequisite",
        };
    }
    if code == "USAGE"
        || code.starts_with("INVALID_")
        || code.starts_with("MISSING_")
        || code.starts_with("DUPLICATE_")
        || code.starts_with("UNKNOWN_")
    {
        return RepairMetadata {
            purpose: "reject malformed or noncanonical input before mutation",
            common_fixes: &[
                "repair the exact field named by the stable error code",
                "regenerate derived subjects with their mapped command",
            ],
            docs_url: "docs/errors.md#invalid-input",
            repair_hint: "rerun the owner/test-map command for the rejected subject",
        };
    }
    if code.contains("RECEIPT") || code.ends_with("_GATE_MISSING") {
        return RepairMetadata {
            purpose: "keep component observations separate from release authority",
            common_fixes: &[
                "produce independently signed exact-subject evidence",
                "register it only after kind-specific semantic verification",
            ],
            docs_url: "docs/errors.md#receipt-missing",
            repair_hint: "follow the blocked gate repair from check release --json",
        };
    }
    RepairMetadata {
        purpose: "keep process completion separate from exact-subject verification",
        common_fixes: &[
            "inspect the exact failing subject and raw proof output",
            "produce new evidence after repairing or changing the subject",
        ],
        docs_url: "docs/errors.md#verification-failed",
        repair_hint: "rerun the mapped proof without skip or allow-failure behavior",
    }
}

impl fmt::Display for CoordError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.reason)
    }
}

impl std::error::Error for CoordError {}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimInput {
    pub agent: String,
    pub lane: String,
    pub repo: String,
    pub paths: Vec<String>,
    pub ttl_seconds: u64,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct HeartbeatInput {
    pub claim_id: String,
    pub agent: String,
    pub ttl_seconds: u64,
    pub note: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct HandoffInput {
    pub claim_id: String,
    pub agent: String,
    pub proof_command: String,
    pub proof_exit_code: i32,
    pub changed_paths: Vec<String>,
    pub commit_oid: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommitReceiptInput {
    pub claim_id: String,
    pub orchestrator: String,
    pub commit_oid: String,
    pub committed_paths: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptCorrectionInput {
    pub claim_id: String,
    pub orchestrator: String,
    pub previous_commit_oid: String,
    pub commit_oid: String,
    pub committed_paths: Vec<String>,
    pub reason: String,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommitReceiptGroupInput {
    pub claim_ids: Vec<String>,
    pub orchestrator: String,
    pub commit_oid: String,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct GroupReceiptCorrectionInput {
    pub claim_ids: Vec<String>,
    pub orchestrator: String,
    pub previous_commit_oid: String,
    pub commit_oid: String,
    pub reason: String,
}

pub fn validate_repo_name(repo: &str) -> Result<(), CoordError> {
    validate_field("repo", repo)?;
    if repo == "."
        || !repo
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(CoordError::new(
            "INVALID_REPO",
            "repository name must contain lowercase ASCII letters, digits, or hyphens",
        ));
    }
    Ok(())
}

pub fn discover_family_root(
    start: &Path,
    explicit: Option<OsString>,
) -> Result<PathBuf, CoordError> {
    if let Some(root) = explicit {
        let root = PathBuf::from(root);
        return verify_root(&root);
    }
    let start = start.canonicalize().map_err(CoordError::io)?;
    let mut discovered = None;
    for ancestor in start.ancestors() {
        if ancestor.join("repos.manifest.toml").is_file() {
            discovered = Some(ancestor.to_path_buf());
        }
    }
    discovered.ok_or_else(|| {
        CoordError::new(
            "FAMILY_ROOT_NOT_FOUND",
            "no ancestor contains repos.manifest.toml; pass --root <path>",
        )
    })
}

pub fn unix_millis() -> Result<u64, CoordError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            CoordError::new(
                "CLOCK_BEFORE_EPOCH",
                format!("system clock failed: {error}"),
            )
        })?;
    u64::try_from(duration.as_millis())
        .map_err(|_| CoordError::new("CLOCK_OVERFLOW", "system time does not fit u64"))
}

pub fn validate_path(raw: &str) -> Result<String, CoordError> {
    validate_field("path", raw)?;
    if raw == "." {
        return Ok(raw.to_owned());
    }
    if raw.starts_with('/') || raw.contains('\\') {
        return Err(CoordError::new(
            "INVALID_PATH",
            format!("path must be repository-relative: {raw}"),
        ));
    }
    if raw
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return Err(CoordError::new(
            "INVALID_PATH",
            format!("path has an empty or dot segment: {raw}"),
        ));
    }
    Ok(raw.trim_end_matches('/').to_owned())
}

pub fn validate_field(name: &str, value: &str) -> Result<(), CoordError> {
    if value.is_empty() || value.len() > 1_024 || value.chars().any(char::is_control) {
        return Err(CoordError::new(
            "INVALID_FIELD",
            format!("{name} must contain 1..=1024 non-control UTF-8 bytes"),
        ));
    }
    Ok(())
}

pub fn validate_ttl(ttl_seconds: u64) -> Result<(), CoordError> {
    if !(30..=86_400).contains(&ttl_seconds) {
        return Err(CoordError::new(
            "INVALID_TTL",
            "TTL must be between 30 and 86400 seconds",
        ));
    }
    Ok(())
}

pub fn validate_commit_oid(oid: &str) -> Result<(), CoordError> {
    if !matches!(oid.len(), 40 | 64)
        || !oid
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(CoordError::new(
            "INVALID_COMMIT_OID",
            "commit OID must be 40 or 64 lowercase hexadecimal characters",
        ));
    }
    Ok(())
}

fn verify_root(root: &Path) -> Result<PathBuf, CoordError> {
    let root = root.canonicalize().map_err(CoordError::io)?;
    if !root.join("repos.manifest.toml").is_file() {
        return Err(CoordError::new(
            "INVALID_ROOT",
            format!("{} has no repos.manifest.toml", root.display()),
        ));
    }
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::{CoordError, validate_commit_oid, validate_path};

    #[test]
    fn validates_repository_relative_paths() {
        assert_eq!(validate_path("src/main.rs").unwrap(), "src/main.rs");
        assert_eq!(validate_path(".").unwrap(), ".");
        for denied in ["/etc/passwd", "../src", "src/../main", "src\\main"] {
            assert!(validate_path(denied).is_err(), "accepted {denied}");
        }
    }

    #[test]
    fn validates_commit_oids() {
        assert!(validate_commit_oid(&"a".repeat(40)).is_ok());
        assert!(validate_commit_oid(&"f".repeat(64)).is_ok());
        assert!(validate_commit_oid(&"A".repeat(40)).is_err());
    }

    #[test]
    fn every_error_class_carries_agent_readable_repair_metadata() {
        for error in [
            CoordError::new("INVALID_PATH", "bad path"),
            CoordError::new("CLAIM_OVERLAP", "owned"),
            CoordError::new("UNSUPPORTED_SCHEMA", "old state"),
            CoordError::new("CORRUPT_COORD_LOG", "bad checksum"),
            CoordError::new("SETUP_TOOL_UNAVAILABLE", "missing tool"),
            CoordError::new("COMMAND_TIMEOUT", "response lost"),
            CoordError::new("MSRV_GATE_MISSING", "no receipt"),
            CoordError::new("PROOF_FAILED", "red proof"),
        ] {
            assert!(!error.purpose().is_empty());
            assert!(error.common_fixes().len() >= 2);
            assert!(error.docs_url().starts_with("docs/errors.md#"));
            assert!(!error.repair_hint().is_empty());
            assert_eq!(
                error.to_string(),
                format!("{}: {}", error.code(), error.reason)
            );
        }
    }
}
