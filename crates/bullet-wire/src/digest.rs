use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::WireError;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Blake3Digest([u8; 32]);

impl Blake3Digest {
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(self) -> String {
        blake3::Hash::from_bytes(self.0).to_hex().to_string()
    }

    pub(crate) fn parse_checked(raw: &str) -> Result<Self, WireError> {
        validate_lower_hex(raw, 64, "INVALID_BLAKE3_DIGEST")?;
        let hash = blake3::Hash::from_hex(raw).map_err(|error| {
            WireError::new("INVALID_BLAKE3_DIGEST", format!("invalid digest: {error}"))
        })?;
        Ok(Self(*hash.as_bytes()))
    }
}

impl fmt::Debug for Blake3Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("Blake3Digest")
            .field(&self.to_hex())
            .finish()
    }
}

impl fmt::Display for Blake3Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_hex())
    }
}

impl FromStr for Blake3Digest {
    type Err = WireError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse_checked(raw)
    }
}

impl Serialize for Blake3Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for Blake3Digest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse_checked(&raw).map_err(de::Error::custom)
    }
}

pub fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, WireError> {
    serde_jcs::to_vec(value).map_err(|error| {
        WireError::new(
            "CANONICAL_JSON_FAILED",
            format!("RFC 8785 encoding failed: {error}"),
        )
    })
}

pub fn hash_canonical<T: Serialize>(domain: &str, value: &T) -> Result<Blake3Digest, WireError> {
    validate_domain(domain)?;
    let canonical = canonical_json(value)?;
    hash_framed_bytes(domain, &canonical)
}

pub fn hash_framed_bytes(domain: &str, bytes: &[u8]) -> Result<Blake3Digest, WireError> {
    validate_domain(domain)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"bullet-wire.v1\0");
    frame(&mut hasher, domain.as_bytes());
    frame(&mut hasher, bytes);
    Ok(Blake3Digest::from_bytes(*hasher.finalize().as_bytes()))
}

fn validate_domain(domain: &str) -> Result<(), WireError> {
    if domain.is_empty()
        || !domain
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || b".-_0123456789".contains(&byte))
    {
        return Err(WireError::new(
            "INVALID_HASH_DOMAIN",
            "hash domain must use lowercase ASCII labels",
        ));
    }
    Ok(())
}

fn frame(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(&(bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

pub(crate) fn validate_lower_hex(
    raw: &str,
    length: usize,
    code: &'static str,
) -> Result<(), WireError> {
    if raw.len() != length
        || !raw
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(WireError::new(
            code,
            format!("expected {length} lowercase hexadecimal characters"),
        ));
    }
    Ok(())
}
