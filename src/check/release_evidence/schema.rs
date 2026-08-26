//! Canonical, gate-specific MSRV evidence. Generic result digests are not authority.

use std::path::PathBuf;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::coord::CoordError;

pub(super) const ADMISSION_SCHEMA: &str = "1";
pub(super) const POLICY_SCHEMA: &str = "1";
pub(super) const RECEIPT_SCHEMA: &str = "1";
pub(super) const TIME_SCHEMA: &str = "1";
pub(super) const GATE_ID: &str = "release.rust-msrv-1-95";
pub(super) const FAMILY: &str = "bullet-farm";
pub(super) const TOOLCHAIN: &str = "1.95.0";
pub(super) const RECEIPT_NAMESPACE: &str = "bullet-farm-msrv-receipt-v1";
pub(super) const TIME_NAMESPACE: &str = "bullet-farm-trusted-time-v1";
pub(super) const MAX_INPUT_BYTES: u64 = 256 * 1024;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AdmissionDescriptor {
    pub release_msrv_admission_schema_version: String,
    pub policy_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MsrvPolicy {
    pub release_msrv_policy_schema_version: String,
    pub family: String,
    pub gate_id: String,
    pub evidence_directory: PathBuf,
    pub source_allowed_signers_path: PathBuf,
    pub attestor_allowed_signers_path: PathBuf,
    pub trusted_time_allowed_signers_path: PathBuf,
    pub attestor_identity: String,
    pub trusted_time_identity: String,
    pub rustc: ToolSubject,
    pub cargo: ToolSubject,
    pub maximum_run_duration_ms: u64,
    pub maximum_receipt_age_ms: u64,
    pub maximum_time_observation_age_ms: u64,
    pub maximum_future_skew_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ToolSubject {
    pub path: PathBuf,
    pub version: String,
    pub digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MsrvReceipt {
    pub release_msrv_receipt_schema_version: String,
    pub family: String,
    pub gate_id: String,
    pub toolchain: String,
    pub evidence_nonce: String,
    pub policy_digest: String,
    pub family_lock_digest: String,
    pub rustc: ToolSubject,
    pub cargo: ToolSubject,
    pub subject: Vec<MsrvSubject>,
    pub command: Vec<CommandObservation>,
    pub started_at_unix_ms: u64,
    pub completed_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub attestor_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MsrvSubject {
    pub repository: String,
    pub tag: String,
    pub commit_oid: String,
    pub tree_oid: String,
    pub lockfile_path: String,
    pub lockfile_digest: String,
    pub release_signing_identity: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(super) enum CommandKind {
    Build,
    Test,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CommandObservation {
    pub repository: String,
    pub kind: CommandKind,
    pub program: PathBuf,
    pub argv: Vec<String>,
    pub environment: Vec<EnvironmentBinding>,
    pub build_units: u64,
    pub tests_passed: u64,
    pub tests_failed: u64,
    pub tests_skipped: u64,
    pub exit_code: i32,
    pub output_digest: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EnvironmentBinding {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TrustedTimeObservation {
    pub trusted_time_schema_version: String,
    pub family: String,
    pub gate_id: String,
    pub evidence_nonce: String,
    pub receipt_digest: String,
    pub policy_digest: String,
    pub observed_at_unix_ms: u64,
    pub valid_until_unix_ms: u64,
    pub trusted_time_identity: String,
}

impl AdmissionDescriptor {
    pub(super) fn parse(bytes: &[u8]) -> Result<Self, CoordError> {
        let value: Self = parse(bytes, "MSRV admission descriptor")?;
        if value.release_msrv_admission_schema_version != ADMISSION_SCHEMA {
            return Err(invalid("MSRV admission descriptor schema is unsupported"));
        }
        require_canonical(&value, bytes, "MSRV admission descriptor")?;
        Ok(value)
    }
}

impl MsrvPolicy {
    pub(super) fn parse(bytes: &[u8]) -> Result<Self, CoordError> {
        let value: Self = parse(bytes, "MSRV evidence policy")?;
        value.validate()?;
        require_canonical(&value, bytes, "MSRV evidence policy")?;
        Ok(value)
    }

    #[cfg(test)]
    pub(super) fn canonical_bytes(&self) -> Result<Vec<u8>, CoordError> {
        self.validate()?;
        canonical(self, "MSRV evidence policy")
    }

    fn validate(&self) -> Result<(), CoordError> {
        if self.release_msrv_policy_schema_version != POLICY_SCHEMA
            || self.family != FAMILY
            || self.gate_id != GATE_ID
        {
            return Err(invalid("MSRV evidence policy authority fields are wrong"));
        }
        if identity_fingerprint(&self.attestor_identity)
            == identity_fingerprint(&self.trusted_time_identity)
        {
            return Err(invalid(
                "attestor and trusted-time identities must be independent",
            ));
        }
        validate_identity(&self.attestor_identity)?;
        validate_identity(&self.trusted_time_identity)?;
        validate_tool(&self.rustc, "rustc 1.95.0 ")?;
        validate_tool(&self.cargo, "cargo 1.95.0 ")?;
        if self.rustc.path == self.cargo.path {
            return Err(invalid("rustc and cargo must be distinct admitted files"));
        }
        for value in [
            self.maximum_run_duration_ms,
            self.maximum_receipt_age_ms,
            self.maximum_time_observation_age_ms,
            self.maximum_future_skew_ms,
        ] {
            validate_time(value)?;
        }
        if self.maximum_run_duration_ms > 24 * 60 * 60 * 1_000
            || self.maximum_receipt_age_ms > 31 * 24 * 60 * 60 * 1_000
            || self.maximum_time_observation_age_ms > 60 * 60 * 1_000
            || self.maximum_future_skew_ms > 5 * 60 * 1_000
        {
            return Err(invalid(
                "MSRV evidence freshness limits exceed hard ceilings",
            ));
        }
        Ok(())
    }
}

impl MsrvReceipt {
    pub(super) fn parse(bytes: &[u8]) -> Result<Self, CoordError> {
        let value: Self = parse(bytes, "MSRV receipt")?;
        value.validate()?;
        require_canonical(&value, bytes, "MSRV receipt")?;
        Ok(value)
    }

    #[cfg(test)]
    pub(super) fn canonical_bytes(&self) -> Result<Vec<u8>, CoordError> {
        self.validate()?;
        canonical(self, "MSRV receipt")
    }

    fn validate(&self) -> Result<(), CoordError> {
        if self.release_msrv_receipt_schema_version != RECEIPT_SCHEMA
            || self.family != FAMILY
            || self.gate_id != GATE_ID
            || self.toolchain != TOOLCHAIN
        {
            return Err(invalid("MSRV receipt authority fields are wrong"));
        }
        validate_nonce(&self.evidence_nonce)?;
        validate_digest(&self.policy_digest)?;
        validate_digest(&self.family_lock_digest)?;
        validate_identity(&self.attestor_identity)?;
        validate_time(self.started_at_unix_ms)?;
        validate_time(self.completed_at_unix_ms)?;
        validate_time(self.expires_at_unix_ms)?;
        if self.started_at_unix_ms >= self.completed_at_unix_ms
            || self.completed_at_unix_ms >= self.expires_at_unix_ms
        {
            return Err(invalid("MSRV receipt timestamps are not strictly ordered"));
        }
        if self.subject.len() != 4 || self.command.len() != 6 {
            return Err(invalid(
                "MSRV receipt requires four subjects and six observations",
            ));
        }
        for subject in &self.subject {
            subject.validate()?;
        }
        if self
            .subject
            .windows(2)
            .any(|pair| pair[0].repository >= pair[1].repository)
        {
            return Err(invalid("MSRV subjects must be byte-sorted and unique"));
        }
        for command in &self.command {
            command.validate()?;
        }
        Ok(())
    }
}

impl MsrvSubject {
    fn validate(&self) -> Result<(), CoordError> {
        validate_repository(&self.repository)?;
        validate_tag(&self.tag)?;
        validate_oid(&self.commit_oid)?;
        validate_oid(&self.tree_oid)?;
        validate_repository_path(&self.lockfile_path)?;
        validate_digest(&self.lockfile_digest)?;
        validate_identity(&self.release_signing_identity)
    }
}

impl CommandObservation {
    fn validate(&self) -> Result<(), CoordError> {
        validate_repository(&self.repository)?;
        validate_digest(&self.output_digest)?;
        if self.build_units > MAX_SAFE_INTEGER
            || self.tests_passed > MAX_SAFE_INTEGER
            || self.tests_failed > MAX_SAFE_INTEGER
            || self.tests_skipped > MAX_SAFE_INTEGER
        {
            return Err(invalid(
                "MSRV command counters exceed the exact integer range",
            ));
        }
        Ok(())
    }
}

impl TrustedTimeObservation {
    pub(super) fn parse(bytes: &[u8]) -> Result<Self, CoordError> {
        let value: Self = parse(bytes, "trusted-time observation")?;
        value.validate()?;
        require_canonical(&value, bytes, "trusted-time observation")?;
        Ok(value)
    }

    #[cfg(test)]
    pub(super) fn canonical_bytes(&self) -> Result<Vec<u8>, CoordError> {
        self.validate()?;
        canonical(self, "trusted-time observation")
    }

    fn validate(&self) -> Result<(), CoordError> {
        if self.trusted_time_schema_version != TIME_SCHEMA
            || self.family != FAMILY
            || self.gate_id != GATE_ID
        {
            return Err(invalid("trusted-time authority fields are wrong"));
        }
        validate_nonce(&self.evidence_nonce)?;
        validate_digest(&self.receipt_digest)?;
        validate_digest(&self.policy_digest)?;
        validate_identity(&self.trusted_time_identity)?;
        validate_time(self.observed_at_unix_ms)?;
        validate_time(self.valid_until_unix_ms)?;
        if self.observed_at_unix_ms >= self.valid_until_unix_ms {
            return Err(invalid("trusted-time interval is empty"));
        }
        Ok(())
    }
}

pub(super) fn canonical<T: Serialize>(value: &T, label: &str) -> Result<Vec<u8>, CoordError> {
    toml::to_string(value)
        .map(String::into_bytes)
        .map_err(|error| invalid(format!("could not encode canonical {label}: {error}")))
}

pub(super) fn digest(domain: &[u8], bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(bytes);
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn parse<T: DeserializeOwned>(bytes: &[u8], label: &str) -> Result<T, CoordError> {
    if bytes.is_empty() || bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err(invalid(format!("{label} exceeds its byte boundary")));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| invalid(format!("{label} is not UTF-8")))?;
    if text
        .bytes()
        .any(|byte| byte == 0 || (byte < 0x20 && !matches!(byte, b'\n' | b'\r' | b'\t')))
    {
        return Err(invalid(format!(
            "{label} contains a forbidden control byte"
        )));
    }
    toml::from_str(text).map_err(|error| invalid(format!("invalid {label} TOML: {error}")))
}

fn require_canonical<T: Serialize>(value: &T, bytes: &[u8], label: &str) -> Result<(), CoordError> {
    if canonical(value, label)? != bytes {
        return Err(invalid(format!("{label} is not canonical TOML v1")));
    }
    Ok(())
}

fn validate_tool(tool: &ToolSubject, prefix: &str) -> Result<(), CoordError> {
    if !tool.path.is_absolute() || !tool.version.starts_with(prefix) || tool.version.contains('\n')
    {
        return Err(invalid("MSRV tool subject is malformed or not Rust 1.95.0"));
    }
    validate_digest(&tool.digest)
}

pub(super) fn validate_digest(value: &str) -> Result<(), CoordError> {
    let Some(hex) = value.strip_prefix("blake3:") else {
        return Err(invalid("digest must use algorithm-tagged BLAKE3"));
    };
    if hex.len() != 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid("digest must be full lowercase BLAKE3"));
    }
    Ok(())
}

fn validate_nonce(value: &str) -> Result<(), CoordError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid(
            "evidence nonce must be 256-bit lowercase hexadecimal",
        ));
    }
    Ok(())
}

fn validate_repository(value: &str) -> Result<(), CoordError> {
    if !matches!(
        value,
        "bullet-farm" | "bullet-git" | "bullet-kernel" | "bullet-portal"
    ) {
        return Err(invalid("MSRV subject names a non-canonical repository"));
    }
    Ok(())
}

fn validate_tag(value: &str) -> Result<(), CoordError> {
    if value.len() > 128
        || !value.starts_with('v')
        || !value.as_bytes().get(1).is_some_and(u8::is_ascii_digit)
        || value.ends_with(['.', '-'])
        || value.contains("..")
        || value
            .bytes()
            .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-')))
    {
        return Err(invalid("release tag is malformed"));
    }
    Ok(())
}

fn validate_repository_path(value: &str) -> Result<(), CoordError> {
    if value.is_empty()
        || value.len() > 4096
        || !value.is_ascii()
        || value.starts_with('/')
        || value.ends_with('/')
        || value.contains('\\')
        || value.bytes().any(|byte| byte.is_ascii_control())
        || value.split('/').any(|segment| {
            segment.is_empty()
                || matches!(segment, "." | "..")
                || segment.eq_ignore_ascii_case(".git")
        })
    {
        return Err(invalid("dependency lock path is unsafe"));
    }
    Ok(())
}

fn validate_oid(value: &str) -> Result<(), CoordError> {
    let valid = value
        .strip_prefix("sha1:")
        .is_some_and(|hex| hex.len() == 40)
        || value
            .strip_prefix("sha256:")
            .is_some_and(|hex| hex.len() == 64);
    if !valid
        || value.split_once(':').is_none_or(|(_, hex)| {
            !hex.bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
    {
        return Err(invalid(
            "Git OID must be full lowercase and algorithm-tagged",
        ));
    }
    Ok(())
}

pub(super) fn validate_identity(value: &str) -> Result<(), CoordError> {
    let Some((principal, rest)) = value.split_once("|ed25519|") else {
        return Err(invalid("signing identity is malformed"));
    };
    if principal.is_empty()
        || principal.len() > 128
        || principal
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte == b'|')
        || !rest.starts_with("SHA256:")
        || rest.len() > 128
        || rest.bytes().any(|byte| {
            !(byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'+' | b'/' | b'='))
        })
    {
        return Err(invalid("signing identity is malformed"));
    }
    Ok(())
}

pub(super) fn identity_fingerprint(value: &str) -> Option<&str> {
    value.split_once("|ed25519|").map(|parts| parts.1)
}

fn validate_time(value: u64) -> Result<(), CoordError> {
    if value == 0 || value > MAX_SAFE_INTEGER {
        return Err(invalid("time value is outside the exact integer range"));
    }
    Ok(())
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_MSRV_RELEASE_EVIDENCE", reason)
}
