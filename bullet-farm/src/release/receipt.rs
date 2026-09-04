//! Canonical signed release-receipt and explicit signer-policy contracts.

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::schema;
use crate::coord::CoordError;

mod verify;
pub(super) use verify::verify;
pub(crate) use verify::verify_detached;

pub const RELEASE_RECEIPT_SCHEMA_VERSION: &str = "1";
pub const RELEASE_RECEIPT_POLICY_SCHEMA_VERSION: &str = "1";
const FAMILY: &str = "bullet-farm";
const SIGNATURE_NAMESPACE: &str = "bullet-farm-release-receipt-v1";
const MAX_RECEIPT_BYTES: u64 = 64 * 1024;
const MAX_POLICY_BYTES: u64 = 64 * 1024;
const MAX_SIGNATURE_BYTES: u64 = 64 * 1024;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const POLICY_DIGEST_DOMAIN: &[u8] = b"bullet-farm.release-receipt-policy.v1\0";

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseReceipt {
    pub release_receipt_schema_version: String,
    pub receipt_kind: ReleaseReceiptKind,
    pub family: String,
    pub tag: String,
    pub hub_commit_oid: String,
    pub hub_tree_oid: String,
    pub tool_name: String,
    pub tool_version: String,
    pub tool_digest: String,
    pub profile: String,
    pub configuration_digest: String,
    pub subject_digest: String,
    pub result: ReleaseReceiptResult,
    pub result_digest: String,
    pub started_at_unix_ms: u64,
    pub completed_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub policy_digest: String,
    pub release_signing_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseReceiptPolicy {
    pub release_receipt_policy_schema_version: String,
    pub family: String,
    pub signature_namespace: String,
    pub signer: Vec<ReleaseReceiptSigner>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseReceiptSigner {
    pub release_signing_identity: String,
    pub public_key: String,
    pub receipt_kind: Vec<ReleaseReceiptKind>,
    pub valid_from_unix_ms: u64,
    pub valid_until_unix_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum ReleaseReceiptKind {
    #[serde(rename = "backup-restore")]
    BackupRestore,
    #[serde(rename = "checksums")]
    Checksums,
    #[serde(rename = "fault-suite")]
    FaultSuite,
    #[serde(rename = "forge-github-app")]
    ForgeGithubApp,
    #[serde(rename = "forge-jeryu")]
    ForgeJeryu,
    #[serde(rename = "installable-lock")]
    InstallableLock,
    #[serde(rename = "installer-twice")]
    InstallerTwice,
    #[serde(rename = "jankurai-90")]
    Jankurai90,
    #[serde(rename = "manifest-non-circular")]
    ManifestNonCircular,
    #[serde(rename = "package-matrix")]
    PackageMatrix,
    #[serde(rename = "platform-containment")]
    PlatformContainment,
    #[serde(rename = "provenance")]
    Provenance,
    #[serde(rename = "provider-antigravity")]
    ProviderAntigravity,
    #[serde(rename = "provider-claude")]
    ProviderClaude,
    #[serde(rename = "provider-codex")]
    ProviderCodex,
    #[serde(rename = "provider-cursor")]
    ProviderCursor,
    #[serde(rename = "rust-msrv-1-95")]
    RustMsrv195,
    #[serde(rename = "rust-pinned-1-97-1")]
    RustPinned1971,
    #[serde(rename = "scan-dependency")]
    ScanDependency,
    #[serde(rename = "scan-license")]
    ScanLicense,
    #[serde(rename = "scan-secret")]
    ScanSecret,
    #[serde(rename = "scan-workflow")]
    ScanWorkflow,
    #[serde(rename = "sbom")]
    Sbom,
    #[serde(rename = "signatures")]
    Signatures,
    #[serde(rename = "transaction-demo")]
    TransactionDemo,
}

impl ReleaseReceiptKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BackupRestore => "backup-restore",
            Self::Checksums => "checksums",
            Self::FaultSuite => "fault-suite",
            Self::ForgeGithubApp => "forge-github-app",
            Self::ForgeJeryu => "forge-jeryu",
            Self::InstallableLock => "installable-lock",
            Self::InstallerTwice => "installer-twice",
            Self::Jankurai90 => "jankurai-90",
            Self::ManifestNonCircular => "manifest-non-circular",
            Self::PackageMatrix => "package-matrix",
            Self::PlatformContainment => "platform-containment",
            Self::Provenance => "provenance",
            Self::ProviderAntigravity => "provider-antigravity",
            Self::ProviderClaude => "provider-claude",
            Self::ProviderCodex => "provider-codex",
            Self::ProviderCursor => "provider-cursor",
            Self::RustMsrv195 => "rust-msrv-1-95",
            Self::RustPinned1971 => "rust-pinned-1-97-1",
            Self::ScanDependency => "scan-dependency",
            Self::ScanLicense => "scan-license",
            Self::ScanSecret => "scan-secret",
            Self::ScanWorkflow => "scan-workflow",
            Self::Sbom => "sbom",
            Self::Signatures => "signatures",
            Self::TransactionDemo => "transaction-demo",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReleaseReceiptResult {
    Verified,
    Failed,
    Unknown,
    Timeout,
    Flaky,
    Unsupported,
    InfraError,
    ZeroTests,
}

impl ReleaseReceiptResult {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "VERIFIED",
            Self::Failed => "FAILED",
            Self::Unknown => "UNKNOWN",
            Self::Timeout => "TIMEOUT",
            Self::Flaky => "FLAKY",
            Self::Unsupported => "UNSUPPORTED",
            Self::InfraError => "INFRA_ERROR",
            Self::ZeroTests => "ZERO_TESTS",
        }
    }
}

impl ReleaseReceipt {
    pub fn parse(bytes: &[u8]) -> Result<Self, CoordError> {
        let receipt: Self =
            parse_toml(bytes, MAX_RECEIPT_BYTES, "release receipt", invalid_receipt)?;
        receipt.validate()?;
        require_canonical(&receipt, bytes, "release receipt", invalid_receipt)?;
        Ok(receipt)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CoordError> {
        self.validate()?;
        encode(self, "release receipt", invalid_receipt)
    }

    fn validate(&self) -> Result<(), CoordError> {
        if self.release_receipt_schema_version != RELEASE_RECEIPT_SCHEMA_VERSION {
            return Err(CoordError::new(
                "UNSUPPORTED_RELEASE_RECEIPT_SCHEMA",
                "release receipt schema is unsupported",
            ));
        }
        validate_subjects(self)?;
        validate_token("tool_name", &self.tool_name, 128, invalid_receipt)?;
        validate_token("tool_version", &self.tool_version, 128, invalid_receipt)?;
        validate_token("profile", &self.profile, 256, invalid_receipt)?;
        for (label, digest) in [
            ("tool_digest", &self.tool_digest),
            ("configuration_digest", &self.configuration_digest),
            ("subject_digest", &self.subject_digest),
            ("result_digest", &self.result_digest),
            ("policy_digest", &self.policy_digest),
        ] {
            validate_digest(label, digest, invalid_receipt)?;
        }
        validate_identity(&self.release_signing_identity, invalid_receipt)?;
        for (label, value) in [
            ("started_at_unix_ms", self.started_at_unix_ms),
            ("completed_at_unix_ms", self.completed_at_unix_ms),
            ("expires_at_unix_ms", self.expires_at_unix_ms),
        ] {
            validate_time(value, label, invalid_receipt)?;
        }
        if self.started_at_unix_ms > self.completed_at_unix_ms
            || self.completed_at_unix_ms >= self.expires_at_unix_ms
        {
            return Err(invalid_receipt(
                "release receipt timestamps are not ordered",
            ));
        }
        Ok(())
    }
}

impl ReleaseReceiptPolicy {
    pub fn parse(bytes: &[u8]) -> Result<Self, CoordError> {
        let policy: Self = parse_toml(
            bytes,
            MAX_POLICY_BYTES,
            "release receipt policy",
            invalid_policy,
        )?;
        policy.validate()?;
        require_canonical(&policy, bytes, "release receipt policy", invalid_policy)?;
        Ok(policy)
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, CoordError> {
        self.validate()?;
        encode(self, "release receipt policy", invalid_policy)
    }

    pub fn digest(&self) -> Result<String, CoordError> {
        Ok(policy_digest(&self.canonical_bytes()?))
    }

    fn validate(&self) -> Result<(), CoordError> {
        if self.release_receipt_policy_schema_version != RELEASE_RECEIPT_POLICY_SCHEMA_VERSION {
            return Err(CoordError::new(
                "UNSUPPORTED_RELEASE_RECEIPT_POLICY_SCHEMA",
                "release receipt policy schema is unsupported",
            ));
        }
        if self.family != FAMILY || self.signature_namespace != SIGNATURE_NAMESPACE {
            return Err(invalid_policy(
                "release receipt policy family or signature namespace is wrong",
            ));
        }
        if self.signer.is_empty() || self.signer.len() > 64 {
            return Err(invalid_policy(
                "release receipt policy needs 1..=64 signers",
            ));
        }
        for signer in &self.signer {
            signer.validate()?;
        }
        if self
            .signer
            .windows(2)
            .any(|pair| pair[0].release_signing_identity >= pair[1].release_signing_identity)
        {
            return Err(invalid_policy(
                "release receipt policy signers must be byte-sorted and unique",
            ));
        }
        Ok(())
    }
}

impl ReleaseReceiptSigner {
    fn validate(&self) -> Result<(), CoordError> {
        validate_identity(&self.release_signing_identity, invalid_policy)?;
        validate_public_key(&self.public_key)?;
        if self.receipt_kind.is_empty()
            || self.receipt_kind.len() > 25
            || self
                .receipt_kind
                .windows(2)
                .any(|pair| pair[0].as_str() >= pair[1].as_str())
        {
            return Err(invalid_policy(
                "signer receipt kinds must be nonempty, byte-sorted, and unique",
            ));
        }
        validate_time(
            self.valid_from_unix_ms,
            "valid_from_unix_ms",
            invalid_policy,
        )?;
        validate_time(
            self.valid_until_unix_ms,
            "valid_until_unix_ms",
            invalid_policy,
        )?;
        if self.valid_from_unix_ms >= self.valid_until_unix_ms {
            return Err(invalid_policy("signer validity interval is empty"));
        }
        Ok(())
    }

    fn identity_parts(&self) -> (&str, &str) {
        identity_parts(&self.release_signing_identity)
            .expect("validated signer identity always has three parts")
    }
}

fn validate_subjects(receipt: &ReleaseReceipt) -> Result<(), CoordError> {
    if receipt.family != FAMILY {
        return Err(invalid_receipt(
            "release receipt family must be bullet-farm",
        ));
    }
    schema::validate_tag(&receipt.tag)
        .map_err(|_| invalid_receipt("release receipt tag is malformed"))?;
    schema::validate_oid("hub_commit_oid", &receipt.hub_commit_oid)
        .map_err(|_| invalid_receipt("release receipt hub commit OID is malformed"))?;
    schema::validate_oid("hub_tree_oid", &receipt.hub_tree_oid)
        .map_err(|_| invalid_receipt("release receipt hub tree OID is malformed"))
}

fn validate_digest(
    label: &str,
    digest: &str,
    invalid: fn(String) -> CoordError,
) -> Result<(), CoordError> {
    schema::validate_digest(digest).map_err(|_| {
        invalid(format!(
            "{label} must be a full algorithm-tagged BLAKE3 digest"
        ))
    })
}

fn validate_token(
    label: &str,
    value: &str,
    maximum: usize,
    invalid: fn(String) -> CoordError,
) -> Result<(), CoordError> {
    if value.is_empty()
        || value.len() > maximum
        || !value.is_ascii()
        || value.bytes().any(|byte| {
            !(byte.is_ascii_alphanumeric()
                || matches!(byte, b'.' | b'_' | b'+' | b':' | b'/' | b'@' | b'-'))
        })
    {
        return Err(invalid(format!("{label} is not a bounded ASCII token")));
    }
    Ok(())
}

fn validate_identity(identity: &str, invalid: fn(String) -> CoordError) -> Result<(), CoordError> {
    let Some((principal, fingerprint)) = identity_parts(identity) else {
        return Err(invalid("release signing identity is malformed".to_owned()));
    };
    validate_token("signer principal", principal, 128, invalid)?;
    let Some(body) = fingerprint.strip_prefix("SHA256:") else {
        return Err(invalid("signer fingerprint must use SHA256".to_owned()));
    };
    if body.len() < 16
        || body.len() > 96
        || body
            .bytes()
            .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'=')))
    {
        return Err(invalid("signer fingerprint is malformed".to_owned()));
    }
    Ok(())
}

fn identity_parts(identity: &str) -> Option<(&str, &str)> {
    let mut parts = identity.split('|');
    let principal = parts.next()?;
    if parts.next()? != "ed25519" {
        return None;
    }
    let fingerprint = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    Some((principal, fingerprint))
}

fn validate_public_key(public_key: &str) -> Result<(), CoordError> {
    let mut parts = public_key.split(' ');
    if parts.next() != Some("ssh-ed25519") {
        return Err(invalid_policy("signer public key must use Ed25519"));
    }
    let blob = parts.next().unwrap_or_default();
    if parts.next().is_some()
        || blob.len() < 40
        || blob.len() > 256
        || blob
            .bytes()
            .any(|byte| !(byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'=')))
    {
        return Err(invalid_policy("signer public key is malformed"));
    }
    Ok(())
}

fn validate_time(
    value: u64,
    label: &str,
    invalid: fn(String) -> CoordError,
) -> Result<(), CoordError> {
    if value == 0 || value > MAX_SAFE_INTEGER {
        return Err(invalid(format!(
            "{label} is outside the exact integer range"
        )));
    }
    Ok(())
}

fn parse_toml<T: DeserializeOwned>(
    bytes: &[u8],
    maximum: u64,
    label: &str,
    invalid: fn(String) -> CoordError,
) -> Result<T, CoordError> {
    if bytes.is_empty()
        || u64::try_from(bytes.len())
            .ok()
            .is_none_or(|size| size > maximum)
    {
        return Err(invalid(format!("{label} exceeds its byte boundary")));
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|_| invalid(format!("{label} must contain valid UTF-8")))?;
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

fn require_canonical<T: Serialize>(
    value: &T,
    bytes: &[u8],
    label: &str,
    invalid: fn(String) -> CoordError,
) -> Result<(), CoordError> {
    if encode(value, label, invalid)? != bytes {
        return Err(invalid(format!("{label} is not canonical TOML v1")));
    }
    Ok(())
}

fn encode<T: Serialize>(
    value: &T,
    label: &str,
    invalid: fn(String) -> CoordError,
) -> Result<Vec<u8>, CoordError> {
    toml::to_string(value)
        .map(String::into_bytes)
        .map_err(|error| invalid(format!("could not encode canonical {label}: {error}")))
}

fn policy_digest(bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(POLICY_DIGEST_DOMAIN);
    hasher.update(bytes);
    format!("blake3:{}", hasher.finalize().to_hex())
}

fn invalid_receipt(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RELEASE_RECEIPT", reason)
}

fn invalid_policy(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RELEASE_RECEIPT_POLICY", reason)
}

#[cfg(all(test, target_os = "linux"))]
#[path = "receipt/tests.rs"]
mod tests;
