//! Signed fixture-only mutation permit. Compiled solely under
//! `fixture-authority`. This is not a frozen-contract substitute.

use bullet_git_types::framed_digest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Schema bound into every fixture permit.
pub const FIXTURE_PERMIT_SCHEMA: &str = "v1";
/// Only disposable generation the fixture daemon will admit.
pub const FIXTURE_WORKSPACE_GENERATION: u64 = 1;
/// Marker written after the first admitted clone.
pub const FIXTURE_GENERATION_MARKER: &str = ".bullet-fixture-generation";

/// Claims bound by the fixture MAC. Subjects are taken from these
/// fields; they are not fabricated after the fact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixturePermitClaims {
    /// Always `v1`.
    pub schema_version: String,
    /// Writer attempt.
    pub attempt_id: String,
    /// Writer fence.
    pub attempt_fence: u64,
    /// 64-hex workspace nonce.
    pub workspace_nonce: String,
    /// Must be `1`.
    pub workspace_generation: u64,
    /// Canonical pre-opened fixture root.
    pub fixture_root: String,
    /// Exclusive expiry.
    pub expires_at_unix_ms: u64,
}

/// Signed fixture permit attached to the authority object.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixturePermit {
    /// Exact claims that were MACed.
    pub claims: FixturePermitClaims,
    /// 64-hex keyed digest.
    pub mac: String,
}

/// Parse a 32-byte nonzero fixture key from 64 lowercase hex.
///
/// # Errors
///
/// Malformed or all-zero key.
pub fn parse_fixture_key(hex_text: &str) -> Result<[u8; 32], String> {
    if hex_text.len() != 64 || !hex_text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("fixture key must be 64 hexadecimal characters".into());
    }
    let raw = hex::decode(hex_text).map_err(|error| error.to_string())?;
    let key: [u8; 32] = raw
        .try_into()
        .map_err(|_| "fixture key must be 32 bytes".to_string())?;
    if key.iter().all(|byte| *byte == 0) {
        return Err("fixture key must be nonzero".into());
    }
    Ok(key)
}

/// MAC over the exact claims and local test key.
#[must_use]
pub fn fixture_mac(key: &[u8; 32], claims: &FixturePermitClaims) -> String {
    let payload = serde_json::to_vec(claims).expect("fixture claims encode");
    framed_digest(&[b"bullet-gitd.fixture-permit.mac.v1", key, &payload]).to_hex()
}

/// Sign one fixture permit.
#[must_use]
pub fn mint_fixture_permit(key: &[u8; 32], claims: FixturePermitClaims) -> FixturePermit {
    let mac = fixture_mac(key, &claims);
    FixturePermit { claims, mac }
}

/// Verify a permit against this daemon's key and pre-opened root.
///
/// # Errors
///
/// Unsigned, stale, or unbound permit.
pub fn verify_fixture_permit(
    key: &[u8; 32],
    fixture_root: &Path,
    permit: &FixturePermit,
    now_unix_ms: u64,
) -> Result<(), String> {
    if permit.claims.schema_version != FIXTURE_PERMIT_SCHEMA {
        return Err("fixture permit schema is not v1".into());
    }
    if permit.claims.workspace_generation != FIXTURE_WORKSPACE_GENERATION {
        return Err("fixture permit workspace_generation must be 1".into());
    }
    if permit.mac != fixture_mac(key, &permit.claims) {
        return Err("fixture permit MAC is invalid".into());
    }
    if now_unix_ms >= permit.claims.expires_at_unix_ms {
        return Err("fixture permit expired".into());
    }
    if permit.claims.fixture_root != fixture_root.to_string_lossy() {
        return Err("fixture permit does not bind this pre-opened root".into());
    }
    Ok(())
}

/// Extract and parse `fixture_permit` from the authority object.
///
/// # Errors
///
/// Missing or malformed permit.
pub fn permit_from_authority(authority: &Value) -> Result<FixturePermit, String> {
    let value = authority
        .get("fixture_permit")
        .ok_or_else(|| "unsigned authority: fixture_permit is required".to_string())?;
    serde_json::from_value(value.clone()).map_err(|error| format!("fixture_permit: {error}"))
}

/// Fail closed unless `root` is an already-created private ordinary directory.
///
/// # Errors
///
/// Missing, relative, symlink, or group/other-writable root.
pub fn require_preopened_fixture_root(root: &Path) -> Result<PathBuf, String> {
    if !root.is_absolute() {
        return Err("fixture root must be an absolute pre-opened path".into());
    }
    let metadata = std::fs::symlink_metadata(root)
        .map_err(|error| format!("fixture root must already exist: {error}"))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("fixture root must be an ordinary directory".into());
    }
    let canonical = std::fs::canonicalize(root)
        .map_err(|error| format!("fixture root is not canonical: {error}"))?;
    if canonical != root {
        return Err("fixture root must be the canonical pre-opened path".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err("fixture root must be mode 0700".into());
        }
    }
    Ok(canonical)
}

/// Consume the single disposable generation on this fixture root.
///
/// # Errors
///
/// Root already consumed or marker write failed.
pub fn consume_fixture_generation(root: &Path) -> Result<(), String> {
    let marker = root.join(FIXTURE_GENERATION_MARKER);
    if marker.exists() {
        return Err("fixture workspace generation 1 is already consumed".into());
    }
    std::fs::write(&marker, b"1\n")
        .map_err(|error| format!("write fixture generation marker: {error}"))
}

/// True when `requested` is exactly the pre-opened fixture root.
#[must_use]
pub fn destination_is_fixture_root(requested: &Path, fixture_root: &Path) -> bool {
    requested == fixture_root
}
