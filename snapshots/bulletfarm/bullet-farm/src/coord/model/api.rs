use serde::{Deserialize, Serialize};

use crate::coord::CoordError;

pub const COORD_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RequestId(String);

impl RequestId {
    pub fn parse(value: impl Into<String>) -> Result<Self, CoordError> {
        let value = value.into();
        validate_prefixed_hex(&value, "req_", "request ID")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        validate_prefixed_hex(self.as_str(), "req_", "request ID")
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct GenerationId(String);

impl GenerationId {
    pub fn parse(value: impl Into<String>) -> Result<Self, CoordError> {
        let value = value.into();
        validate_prefixed_hex(&value, "gen_", "generation ID")?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        validate_prefixed_hex(self.as_str(), "gen_", "generation ID")
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MutationEnvelope<T> {
    pub request_id: RequestId,
    pub expected_generation_id: GenerationId,
    pub command: T,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GenesisInput {
    pub operator: String,
    pub policy_sha256: String,
    pub replay_contract_version: u32,
    pub replay_contract_sha256: String,
    pub bootstrap_commit_oid: String,
    pub bootstrap_paths: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommandReceipt {
    pub generation_id: String,
    pub request_id: String,
    pub command_subject_blake3: String,
    pub stored_request_blake3: String,
    pub sequence: u64,
    pub record_blake3: String,
    pub envelope_blake3: String,
    pub byte_offset: u64,
    pub frame_length: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Watermark {
    pub generation_id: String,
    pub manifest_blake3: String,
    pub last_sequence: u64,
    pub next_sequence: u64,
    pub head_envelope_blake3: String,
    pub last_record_blake3: String,
    pub last_request_id: String,
    pub last_request_blake3: String,
    pub byte_length: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Applied<T> {
    pub receipt: CommandReceipt,
    pub watermark: Watermark,
    pub projection: T,
    pub replayed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum StatusOrigin {
    Genesis,
    Recovered {
        incident_at_unix_ms: u64,
        recovered_at_unix_ms: u64,
        trusted_records: u64,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub schema_version: u32,
    pub generation_id: String,
    pub manifest_blake3: String,
    pub origin: StatusOrigin,
    pub as_of_sequence: u64,
    pub next_sequence: u64,
    pub last_request_id: String,
    pub last_request_blake3: String,
    pub last_record_blake3: String,
    pub last_envelope_blake3: String,
    pub byte_length: u64,
    pub observed_at_unix_ms: u64,
    pub source: String,
    pub claims: Vec<super::ClaimSummary>,
}

fn validate_prefixed_hex(value: &str, prefix: &str, label: &str) -> Result<(), CoordError> {
    if value.len() == prefix.len() + 64
        && value.starts_with(prefix)
        && value[prefix.len()..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(CoordError::new(
            "INVALID_COORD_ID",
            format!("{label} must be {prefix} plus 64 lowercase hexadecimal digits"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{GenerationId, RequestId};

    #[test]
    fn typed_ids_are_full_width_lowercase() {
        assert!(RequestId::parse(format!("req_{}", "a".repeat(64))).is_ok());
        assert!(GenerationId::parse(format!("gen_{}", "f".repeat(64))).is_ok());
        for value in [
            "req_short".to_owned(),
            format!("req_{}", "A".repeat(64)),
            format!("gen_{}", "g".repeat(64)),
        ] {
            assert!(
                RequestId::parse(value.clone()).is_err() || GenerationId::parse(value).is_err()
            );
        }
    }
}
