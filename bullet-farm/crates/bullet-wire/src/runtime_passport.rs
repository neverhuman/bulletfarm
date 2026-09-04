//! Strict provider-runtime identity. This source mirror is component-only until
//! the generated contract bundle publishes the same bytes.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    DogfoodProviderProtocolV1, LaunchProvider, RuntimePassportId, canonical_json, decode_canonical,
    hash_framed_bytes,
};

#[cfg(test)]
mod tests;

pub const RUNTIME_PASSPORT_SCHEMA_VERSION: u32 = 1;
pub const RUNTIME_PASSPORT_DOMAIN: &str = "provider-runtime-passport.v1";
pub const RUNTIME_PASSPORT_ID_PREFIX: &str = RuntimePassportId::PREFIX;
pub const RUNTIME_DEPLOYMENT_PREFIX: &str = "/usr/lib/bullet/providers";
pub const MAX_RUNTIME_PASSPORT_BYTES: usize = 64 * 1024;
pub const MAX_RUNTIME_FILES: usize = 256;
pub const MAX_RUNTIME_FILE_BYTES: u64 = 512 * 1024 * 1024;
pub const MAX_RUNTIME_TOTAL_BYTES: u64 = 4 * 1024 * 1024 * 1024;
pub const MAX_RUNTIME_VERSION_BYTES: usize = 64;
pub const MAX_RUNTIME_RELATIVE_PATH_BYTES: usize = 512;

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuntimePassportError {
    Malformed {
        reason: String,
    },
    ProtocolMismatch {
        provider: String,
        expected: String,
        actual: String,
    },
    IdMismatch,
}

impl RuntimePassportError {
    #[must_use]
    pub const fn reason_code(&self) -> &'static str {
        match self {
            Self::Malformed { .. } => "RUNTIME_PASSPORT_MALFORMED",
            Self::ProtocolMismatch { .. } => "RUNTIME_PASSPORT_PROTOCOL_MISMATCH",
            Self::IdMismatch => "RUNTIME_PASSPORT_ID_MISMATCH",
        }
    }
}

impl fmt::Display for RuntimePassportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed { reason } => {
                write!(formatter, "provider runtime passport malformed: {reason}")
            }
            Self::ProtocolMismatch {
                provider,
                expected,
                actual,
            } => write!(
                formatter,
                "provider runtime passport protocol mismatch for {provider}: expected {expected}, got {actual}"
            ),
            Self::IdMismatch => {
                formatter.write_str("provider runtime passport id does not match the external lock")
            }
        }
    }
}

impl std::error::Error for RuntimePassportError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeFileRoleV1 {
    Entrypoint,
    Executable,
    Loader,
    Interpreter,
    NativeLibrary,
    Module,
    Resource,
    ProtocolSchema,
    License,
    Sbom,
}

impl RuntimeFileRoleV1 {
    const fn executable(self) -> bool {
        matches!(
            self,
            Self::Entrypoint | Self::Executable | Self::Loader | Self::Interpreter
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeFileV1 {
    pub path: String,
    pub role: RuntimeFileRoleV1,
    pub mode: u32,
    pub size: u64,
    pub blake3: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeLoaderV1 {
    Static,
    Dynamic { path: String, blake3: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeExecutionV1 {
    Native {
        loader: RuntimeLoaderV1,
    },
    Interpreted {
        interpreter_path: String,
        interpreter_blake3: String,
        loader: RuntimeLoaderV1,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderRuntimePassportV1 {
    pub schema_version: u32,
    pub provider: LaunchProvider,
    pub protocol: DogfoodProviderProtocolV1,
    pub version: String,
    pub deployment_root: String,
    pub entrypoint: String,
    pub execution: RuntimeExecutionV1,
    pub files: Vec<RuntimeFileV1>,
    pub aggregate_file_count: u32,
    pub aggregate_size_bytes: u64,
}

impl ProviderRuntimePassportV1 {
    /// Validate the closed structural subject. Success grants no launch authority.
    pub fn validate(&self) -> Result<(), RuntimePassportError> {
        if self.schema_version != RUNTIME_PASSPORT_SCHEMA_VERSION {
            return Err(malformed("schema_version must be 1"));
        }
        let required = DogfoodProviderProtocolV1::required_for(self.provider);
        if self.protocol != required {
            return Err(RuntimePassportError::ProtocolMismatch {
                provider: self.provider.as_str().to_owned(),
                expected: required.as_str().to_owned(),
                actual: self.protocol.as_str().to_owned(),
            });
        }
        validate_version(&self.version)?;
        let expected_root = format!(
            "{RUNTIME_DEPLOYMENT_PREFIX}/{}/{}",
            self.provider.as_str(),
            self.version
        );
        if self.deployment_root != expected_root {
            return Err(malformed(
                "deployment_root is not the normalized immutable root",
            ));
        }
        validate_relative_path("entrypoint", &self.entrypoint)?;
        self.validate_manifest()?;
        self.validate_execution()
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, RuntimePassportError> {
        self.validate()?;
        let bytes = canonical_json(self).map_err(|_| malformed("RFC 8785 encoding failed"))?;
        if bytes.len() > MAX_RUNTIME_PASSPORT_BYTES {
            return Err(malformed("canonical passport exceeds 64 KiB"));
        }
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, RuntimePassportError> {
        if bytes.len() > MAX_RUNTIME_PASSPORT_BYTES {
            return Err(malformed("canonical passport exceeds 64 KiB"));
        }
        let passport: Self = decode_canonical(bytes)
            .map_err(|_| malformed("document is not strict canonical ProviderRuntimePassportV1"))?;
        passport.validate()?;
        Ok(passport)
    }

    pub fn passport_id(&self) -> Result<RuntimePassportId, RuntimePassportError> {
        let bytes = self.canonical_bytes()?;
        let digest = hash_framed_bytes(RUNTIME_PASSPORT_DOMAIN, &bytes)
            .map_err(|_| malformed("passport digest framing failed"))?;
        Ok(RuntimePassportId::from_digest(digest))
    }

    fn validate_manifest(&self) -> Result<(), RuntimePassportError> {
        if self.files.is_empty() || self.files.len() > MAX_RUNTIME_FILES {
            return Err(malformed("files must contain 1..=256 members"));
        }
        if self.aggregate_file_count != self.files.len() as u32 {
            return Err(malformed("aggregate_file_count does not match files"));
        }
        let mut total = 0_u64;
        let mut previous: Option<&[u8]> = None;
        let mut entrypoints = 0_u32;
        for file in &self.files {
            validate_relative_path("file path", &file.path)?;
            if previous.is_some_and(|path| path >= file.path.as_bytes()) {
                return Err(malformed(
                    "files must be strictly unsigned-byte-sorted and unique",
                ));
            }
            previous = Some(file.path.as_bytes());
            validate_file(file)?;
            total = total
                .checked_add(file.size)
                .ok_or_else(|| malformed("aggregate file size overflowed"))?;
            if file.role == RuntimeFileRoleV1::Entrypoint {
                entrypoints += 1;
                if file.path != self.entrypoint {
                    return Err(malformed("entrypoint role does not match entrypoint"));
                }
            }
        }
        if entrypoints != 1 {
            return Err(malformed("manifest must contain exactly one entrypoint"));
        }
        if total != self.aggregate_size_bytes
            || total > MAX_RUNTIME_TOTAL_BYTES
            || total > MAX_SAFE_INTEGER
        {
            return Err(malformed(
                "aggregate_size_bytes is invalid or does not match files",
            ));
        }
        Ok(())
    }

    fn validate_execution(&self) -> Result<(), RuntimePassportError> {
        match &self.execution {
            RuntimeExecutionV1::Native { loader } => self.validate_loader(loader),
            RuntimeExecutionV1::Interpreted {
                interpreter_path,
                interpreter_blake3,
                loader,
            } => {
                self.require_member(
                    interpreter_path,
                    RuntimeFileRoleV1::Interpreter,
                    interpreter_blake3,
                )?;
                self.validate_loader(loader)
            }
        }
    }

    fn validate_loader(&self, loader: &RuntimeLoaderV1) -> Result<(), RuntimePassportError> {
        match loader {
            RuntimeLoaderV1::Static => Ok(()),
            RuntimeLoaderV1::Dynamic { path, blake3 } => {
                self.require_member(path, RuntimeFileRoleV1::Loader, blake3)
            }
        }
    }

    fn require_member(
        &self,
        path: &str,
        role: RuntimeFileRoleV1,
        digest: &str,
    ) -> Result<(), RuntimePassportError> {
        validate_relative_path("linkage path", path)?;
        let member = self
            .files
            .binary_search_by_key(&path, |file| file.path.as_str())
            .ok()
            .map(|index| &self.files[index])
            .ok_or_else(|| malformed("linkage path is absent from files"))?;
        if member.role != role || member.blake3 != digest {
            return Err(malformed("linkage role or digest does not match files"));
        }
        Ok(())
    }
}

pub fn decode_expected_runtime_passport(
    bytes: &[u8],
    expected: &RuntimePassportId,
) -> Result<ProviderRuntimePassportV1, RuntimePassportError> {
    let passport = ProviderRuntimePassportV1::decode(bytes)?;
    if &passport.passport_id()? != expected {
        return Err(RuntimePassportError::IdMismatch);
    }
    Ok(passport)
}

fn validate_file(file: &RuntimeFileV1) -> Result<(), RuntimePassportError> {
    if !is_lower_hex_64(&file.blake3) {
        return Err(malformed("file blake3 must be 64 lowercase hex characters"));
    }
    if file.size > MAX_RUNTIME_FILE_BYTES
        || file.size > MAX_SAFE_INTEGER
        || (file.role.executable() && file.size == 0)
    {
        return Err(malformed("file size is outside the admitted range"));
    }
    let valid_mode = file.mode <= 0o777
        && file.mode & 0o222 == 0
        && file.mode & 0o444 != 0
        && (!file.role.executable() || file.mode & 0o111 != 0);
    if !valid_mode {
        return Err(malformed(
            "file mode is not read-only or disagrees with its role",
        ));
    }
    Ok(())
}

fn validate_version(version: &str) -> Result<(), RuntimePassportError> {
    let valid = !version.is_empty()
        && version.len() <= MAX_RUNTIME_VERSION_BYTES
        && version != "."
        && version != ".."
        && version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'+' | b'-'));
    if !valid {
        return Err(malformed(
            "version must be one bounded immutable-root segment",
        ));
    }
    Ok(())
}

fn validate_relative_path(field: &str, path: &str) -> Result<(), RuntimePassportError> {
    let valid = !path.is_empty()
        && path.len() <= MAX_RUNTIME_RELATIVE_PATH_BYTES
        && !path.starts_with('/')
        && !path.ends_with('/')
        && path
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b'\\')
        && path
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..");
    if !valid {
        return Err(malformed(&format!(
            "{field} is not a normalized relative path"
        )));
    }
    Ok(())
}

fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn malformed(reason: &str) -> RuntimePassportError {
    RuntimePassportError::Malformed {
        reason: reason.to_owned(),
    }
}
