use serde::{Deserialize, Serialize};

use crate::coord::{CoordError, validate_commit_oid, validate_field, validate_path};

use super::{invalid, validate_prefixed};

const PROVENANCE_KIND: &str = "bullet.coord.recovery-bootstrap-provenance.v1";
const MAX_SOURCE_FILES: usize = 8_192;
const MAX_SOURCE_PATH_BYTES: usize = 512;
const MAX_SOURCE_FILE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_SOURCE_AGGREGATE_BYTES: u64 = 256 * 1024 * 1024;
const MAX_EXECUTABLE_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryBootstrapSourceV1 {
    pub(crate) path: String,
    pub(crate) byte_length: u64,
    pub(crate) sha256: String,
}

impl RecoveryBootstrapSourceV1 {
    fn validate(&self) -> Result<(), CoordError> {
        validate_path(&self.path)?;
        if self.path == "." || self.path.len() > MAX_SOURCE_PATH_BYTES {
            return Err(invalid(
                "bootstrap provenance source path must identify a repository file within 512 bytes",
            ));
        }
        validate_prefixed(&self.sha256, "sha256:", 64, "bootstrap source SHA-256")?;
        if self.byte_length == 0 || self.byte_length > MAX_SOURCE_FILE_BYTES {
            return Err(invalid("bootstrap source byte length is invalid"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryBootstrapProvenanceV1 {
    pub(crate) kind: String,
    pub(crate) schema_version: u32,
    pub(crate) bootstrap_commit_oid: String,
    pub(crate) bootstrap_tree_oid: String,
    pub(crate) archive_sha256: String,
    pub(crate) cargo_lock_sha256: String,
    pub(crate) source_files: Vec<RecoveryBootstrapSourceV1>,
    pub(crate) rustc_version: String,
    pub(crate) cargo_version: String,
    pub(crate) executable_byte_length: u64,
    pub(crate) executable_sha256: String,
}

impl RecoveryBootstrapProvenanceV1 {
    pub(crate) fn from_observations(
        bootstrap_commit_oid: String,
        bootstrap_tree_oid: String,
        archive_sha256: String,
        cargo_lock_sha256: String,
        source_files: Vec<(String, u64, String)>,
        versions: (String, String),
        executable: (u64, String),
    ) -> Result<Self, CoordError> {
        let value = Self {
            kind: PROVENANCE_KIND.to_owned(),
            schema_version: 1,
            bootstrap_commit_oid,
            bootstrap_tree_oid,
            archive_sha256,
            cargo_lock_sha256,
            source_files: source_files
                .into_iter()
                .map(|(path, byte_length, sha256)| RecoveryBootstrapSourceV1 {
                    path,
                    byte_length,
                    sha256,
                })
                .collect(),
            rustc_version: versions.0,
            cargo_version: versions.1,
            executable_byte_length: executable.0,
            executable_sha256: executable.1,
        };
        value.validate()?;
        Ok(value)
    }

    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        if self.kind != PROVENANCE_KIND || self.schema_version != 1 {
            return Err(invalid(
                "bootstrap provenance kind or schema version is unsupported",
            ));
        }
        validate_commit_oid(&self.bootstrap_commit_oid)?;
        validate_commit_oid(&self.bootstrap_tree_oid)?;
        if self.bootstrap_commit_oid.len() != self.bootstrap_tree_oid.len() {
            return Err(invalid(
                "bootstrap commit and tree OIDs must use the same object format",
            ));
        }
        validate_prefixed(&self.archive_sha256, "sha256:", 64, "archive SHA-256")?;
        validate_prefixed(&self.cargo_lock_sha256, "sha256:", 64, "Cargo.lock SHA-256")?;
        validate_prefixed(
            &self.executable_sha256,
            "sha256:",
            64,
            "bootstrap executable SHA-256",
        )?;
        validate_field("rustc_version", &self.rustc_version)?;
        validate_field("cargo_version", &self.cargo_version)?;
        if self.executable_byte_length == 0 || self.executable_byte_length > MAX_EXECUTABLE_BYTES {
            return Err(invalid("bootstrap executable byte length is invalid"));
        }
        if self.source_files.is_empty() || self.source_files.len() > MAX_SOURCE_FILES {
            return Err(invalid(
                "bootstrap provenance source inventory must contain 1..=8,192 files",
            ));
        }
        let mut previous: Option<&str> = None;
        let mut lock_sha256 = None;
        let mut rust_toolchain_seen = false;
        let mut aggregate = 0_u64;
        for source in &self.source_files {
            source.validate()?;
            if previous.is_some_and(|path| path >= source.path.as_str()) {
                return Err(invalid(
                    "bootstrap provenance source paths must be sorted and unique",
                ));
            }
            if source.path == "Cargo.lock" {
                lock_sha256 = Some(source.sha256.as_str());
            }
            if source.path == "rust-toolchain.toml" {
                rust_toolchain_seen = true;
            }
            aggregate = aggregate
                .checked_add(source.byte_length)
                .filter(|total| *total <= MAX_SOURCE_AGGREGATE_BYTES)
                .ok_or_else(|| {
                    invalid("bootstrap provenance sources exceed 256 MiB in aggregate")
                })?;
            previous = Some(&source.path);
        }
        if lock_sha256 != Some(self.cargo_lock_sha256.as_str()) {
            return Err(invalid(
                "bootstrap provenance Cargo.lock source must equal its top-level digest",
            ));
        }
        if !rust_toolchain_seen {
            return Err(invalid(
                "bootstrap provenance must contain rust-toolchain.toml",
            ));
        }
        Ok(())
    }

    pub(crate) fn bootstrap_paths(&self) -> Vec<String> {
        self.source_files
            .iter()
            .map(|source| source.path.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::RecoveryBootstrapProvenanceV1;

    fn valid() -> RecoveryBootstrapProvenanceV1 {
        RecoveryBootstrapProvenanceV1::from_observations(
            "a".repeat(40),
            "b".repeat(40),
            format!("sha256:{}", "c".repeat(64)),
            format!("sha256:{}", "d".repeat(64)),
            vec![
                (
                    "Cargo.lock".to_owned(),
                    1,
                    format!("sha256:{}", "d".repeat(64)),
                ),
                (
                    "rust-toolchain.toml".to_owned(),
                    1,
                    format!("sha256:{}", "f".repeat(64)),
                ),
            ],
            ("rustc 1.95.0".to_owned(), "cargo 1.95.0".to_owned()),
            (1, format!("sha256:{}", "e".repeat(64))),
        )
        .unwrap()
    }

    #[test]
    fn provenance_refuses_mixed_git_object_widths() {
        let mut value = valid();
        value.bootstrap_tree_oid = "b".repeat(64);
        let error = value.validate().unwrap_err();
        assert_eq!(error.code(), "INVALID_RECOVERY_MANIFEST_PRODUCTION");
    }

    #[test]
    fn provenance_refuses_repository_root_as_a_source_file() {
        let mut value = valid();
        value.source_files[0].path = ".".to_owned();
        let error = value.validate().unwrap_err();
        assert_eq!(error.code(), "INVALID_RECOVERY_MANIFEST_PRODUCTION");
    }

    #[test]
    fn provenance_accepts_exact_producer_bounds() {
        let mut value = valid();
        value.source_files.push(super::RecoveryBootstrapSourceV1 {
            path: "z".repeat(512),
            byte_length: 1,
            sha256: format!("sha256:{}", "a".repeat(64)),
        });
        value.validate().unwrap();

        let mut value = valid();
        value.source_files[0].byte_length = 64 * 1024 * 1024;
        value.validate().unwrap();

        let mut value = valid();
        value.executable_byte_length = 512 * 1024 * 1024;
        value.validate().unwrap();

        let mut value = valid();
        value.source_files = ["Cargo.lock", "a", "b", "rust-toolchain.toml"]
            .into_iter()
            .map(|path| super::RecoveryBootstrapSourceV1 {
                path: path.to_owned(),
                byte_length: 64 * 1024 * 1024,
                sha256: if path == "Cargo.lock" {
                    value.cargo_lock_sha256.clone()
                } else {
                    format!("sha256:{}", "a".repeat(64))
                },
            })
            .collect();
        value.validate().unwrap();

        let mut value = valid();
        let lock = value.source_files[0].clone();
        let toolchain = value.source_files[1].clone();
        value.source_files = std::iter::once(lock)
            .chain((0..8_190).map(|index| super::RecoveryBootstrapSourceV1 {
                path: format!("f{index:04}"),
                byte_length: 1,
                sha256: format!("sha256:{}", "a".repeat(64)),
            }))
            .chain(std::iter::once(toolchain))
            .collect();
        assert_eq!(value.source_files.len(), 8_192);
        value.validate().unwrap();
    }

    #[test]
    fn provenance_refuses_values_outside_producer_bounds() {
        let mut value = valid();
        value.source_files[0].path = "a".repeat(513);
        assert_eq!(
            value.validate().unwrap_err().code(),
            "INVALID_RECOVERY_MANIFEST_PRODUCTION"
        );

        let mut value = valid();
        value.source_files[0].byte_length = 64 * 1024 * 1024 + 1;
        assert_eq!(
            value.validate().unwrap_err().code(),
            "INVALID_RECOVERY_MANIFEST_PRODUCTION"
        );

        let mut value = valid();
        value.executable_byte_length = 512 * 1024 * 1024 + 1;
        assert_eq!(
            value.validate().unwrap_err().code(),
            "INVALID_RECOVERY_MANIFEST_PRODUCTION"
        );

        let mut value = valid();
        value.source_files = (0..8_192)
            .map(|index| super::RecoveryBootstrapSourceV1 {
                path: format!("f{index:04}"),
                byte_length: 1,
                sha256: format!("sha256:{}", "a".repeat(64)),
            })
            .collect();
        value.source_files.insert(
            0,
            super::RecoveryBootstrapSourceV1 {
                path: "Cargo.lock".to_owned(),
                byte_length: 1,
                sha256: value.cargo_lock_sha256.clone(),
            },
        );
        assert_eq!(
            value.validate().unwrap_err().code(),
            "INVALID_RECOVERY_MANIFEST_PRODUCTION"
        );

        let mut value = valid();
        value.source_files = ["Cargo.lock", "a", "b", "c", "rust-toolchain.toml"]
            .into_iter()
            .map(|path| super::RecoveryBootstrapSourceV1 {
                path: path.to_owned(),
                byte_length: if path == "rust-toolchain.toml" {
                    1
                } else {
                    64 * 1024 * 1024
                },
                sha256: if path == "Cargo.lock" {
                    value.cargo_lock_sha256.clone()
                } else {
                    format!("sha256:{}", "a".repeat(64))
                },
            })
            .collect();
        assert_eq!(
            value.validate().unwrap_err().code(),
            "INVALID_RECOVERY_MANIFEST_PRODUCTION"
        );

        let mut value = valid();
        value
            .source_files
            .retain(|source| source.path != "rust-toolchain.toml");
        assert_eq!(
            value.validate().unwrap_err().code(),
            "INVALID_RECOVERY_MANIFEST_PRODUCTION"
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn producer_preflight_refuses_model_valid_oversized_canonical_document() {
        let mut value = valid();
        let lock = value.source_files[0].clone();
        let toolchain = value.source_files[1].clone();
        value.source_files = std::iter::once(lock)
            .chain((0..8_190).map(|index| super::RecoveryBootstrapSourceV1 {
                path: format!("f{index:04}-{}", "x".repeat(120)),
                byte_length: 1,
                sha256: format!("sha256:{}", "a".repeat(64)),
            }))
            .chain(std::iter::once(toolchain))
            .collect();
        value.validate().unwrap();
        assert!(
            bullet_wire::canonical_json(&value).unwrap().len()
                > bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES
        );
        let error = crate::coord::recovery_manifest::test_preflight_document(&value).unwrap_err();
        assert_eq!(error.code(), "INVALID_RECOVERY_PRODUCTION");
    }
}
