use std::{collections::BTreeMap, fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use unicode_normalization::UnicodeNormalization;

use crate::{AttemptId, Blake3Digest, CheckpointId, ContentId, GateId, WireError};

pub const DEFAULT_MAX_OPERATIONS: usize = 1_024;
pub const DEFAULT_MAX_CONTENT_BYTES: usize = 1_048_576;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepoPath(String);

impl RepoPath {
    pub(crate) fn parse_checked(raw: &str) -> Result<Self, WireError> {
        validate_repo_path(raw)?;
        Ok(Self(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn collision_key(&self) -> String {
        self.0.nfc().flat_map(char::to_lowercase).collect()
    }

    pub fn contains(&self, other: &Self) -> bool {
        self == other || contains_path(&self.0, &other.0)
    }
}

impl FromStr for RepoPath {
    type Err = WireError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse_checked(raw)
    }
}

impl fmt::Display for RepoPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Serialize for RepoPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for RepoPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse_checked(&raw).map_err(de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Preimage {
    Absent,
    Digest { digest: Blake3Digest },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PatchMutation {
    Write { content_utf8: String },
    Delete,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchOperation {
    pub path: RepoPath,
    pub preimage: Preimage,
    pub mutation: PatchMutation,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchProposal {
    pub schema_version: u32,
    pub proposal_id: ContentId,
    pub producing_attempt_id: AttemptId,
    pub base_checkpoint_id: CheckpointId,
    pub base_checkpoint_digest: Blake3Digest,
    pub operations: Vec<PatchOperation>,
    pub gate_ids: Vec<GateId>,
}

impl PatchProposal {
    pub fn validate(&self) -> Result<(), WireError> {
        if self.schema_version != crate::SCHEMA_VERSION {
            return Err(WireError::new(
                "UNSUPPORTED_SCHEMA",
                format!(
                    "PatchProposal schema {} is unsupported",
                    self.schema_version
                ),
            ));
        }
        if self.operations.is_empty() || self.operations.len() > DEFAULT_MAX_OPERATIONS {
            return Err(WireError::new(
                "INVALID_OPERATION_COUNT",
                format!("operations must contain 1..={DEFAULT_MAX_OPERATIONS} entries"),
            ));
        }
        if self.gate_ids.is_empty() {
            return Err(WireError::new(
                "GATE_REQUIRED",
                "PatchProposal must reference at least one admitted gate",
            ));
        }
        reject_duplicate_gates(&self.gate_ids)?;
        validate_operations(&self.operations)
    }
}

fn validate_operations(operations: &[PatchOperation]) -> Result<(), WireError> {
    let mut paths = BTreeMap::new();
    for operation in operations {
        if let PatchMutation::Write { content_utf8 } = &operation.mutation
            && content_utf8.len() > DEFAULT_MAX_CONTENT_BYTES
        {
            return Err(WireError::new(
                "CONTENT_TOO_LARGE",
                format!(
                    "{} exceeds {DEFAULT_MAX_CONTENT_BYTES} bytes",
                    operation.path
                ),
            ));
        }
        if matches!(operation.mutation, PatchMutation::Delete)
            && matches!(operation.preimage, Preimage::Absent)
        {
            return Err(WireError::new(
                "MISSING_PREIMAGE",
                format!("delete {} requires an existing preimage", operation.path),
            ));
        }
        let key = operation.path.collision_key();
        if let Some(existing) = paths.insert(key, operation.path.as_str()) {
            return Err(WireError::new(
                "PATH_COLLISION",
                format!("{} conflicts with {existing}", operation.path),
            ));
        }
    }
    for (index, (left_key, left)) in paths.iter().enumerate() {
        for (right_key, right) in paths.iter().skip(index + 1) {
            if contains_path(left_key, right_key) || contains_path(right_key, left_key) {
                return Err(WireError::new(
                    "PATH_CONFLICT",
                    format!("{left} conflicts with {right}"),
                ));
            }
        }
    }
    Ok(())
}

fn reject_duplicate_gates(gates: &[GateId]) -> Result<(), WireError> {
    let mut sorted = gates.iter().map(GateId::as_str).collect::<Vec<_>>();
    sorted.sort_unstable();
    if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(WireError::new(
            "DUPLICATE_GATE",
            "PatchProposal repeats an admitted gate ID",
        ));
    }
    Ok(())
}

fn validate_repo_path(raw: &str) -> Result<(), WireError> {
    if raw.is_empty() || raw.len() > 4_096 || raw.starts_with('/') || raw.contains('\\') {
        return Err(invalid_path(
            raw,
            "path must be a non-empty repository-relative UTF-8 path",
        ));
    }
    if raw.nfc().collect::<String>() != raw {
        return Err(invalid_path(raw, "path must already be NFC normalized"));
    }
    for segment in raw.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(invalid_path(raw, "empty and dot segments are forbidden"));
        }
        if segment.eq_ignore_ascii_case(".git") {
            return Err(invalid_path(raw, ".git is outside proposal authority"));
        }
        if segment.contains(':')
            || segment.ends_with('.')
            || segment.ends_with(' ')
            || segment.chars().any(char::is_control)
        {
            return Err(invalid_path(
                raw,
                "path contains a platform-unsafe component",
            ));
        }
    }
    Ok(())
}

fn contains_path(parent: &str, child: &str) -> bool {
    child
        .strip_prefix(parent)
        .is_some_and(|suffix| suffix.starts_with('/'))
}

fn invalid_path(raw: &str, reason: &str) -> WireError {
    WireError::new("INVALID_REPO_PATH", format!("{raw:?}: {reason}"))
}
