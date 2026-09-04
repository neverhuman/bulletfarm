use serde::{Deserialize, Serialize};

use crate::coord::CoordError;

use super::FrozenClaimSubject;
use crate::coord::generation::manifest::{
    ArtifactBinding, ByteRange, RecoveryArtifacts, Sha256Digest, TrustedProjectionInventory,
};

pub(crate) const RECOVERY_INSPECTION_SCHEMA_VERSION: u32 = 1;
const INSPECTION_KIND: &str = "bullet.coord.recovery-inspection.v1";
const INSPECTION_DOMAIN: &str = "bullet-family.coord.recovery-inspection.v1";

mod authorization;
mod bootstrap_build;
#[allow(
    unused_imports,
    reason = "COMPONENT_ONLY build contracts await their coord::model export"
)]
pub(in crate::coord) use bootstrap_build::bootstrap_contract::{
    RecoveryBootstrapBuilderContractV1, RecoveryBootstrapCommandContractV1,
    RecoveryBootstrapToolchainContractV1, ToolchainArtifactKindV1, ToolchainMemberV1,
    ToolchainRoleV1,
};
#[allow(
    unused_imports,
    reason = "COMPONENT_ONLY build contracts await their coord::model export"
)]
pub(in crate::coord) use bootstrap_build::{
    CargoOfflineCacheManifestV1, RecoveryBootstrapBuildObservationV1,
};
mod provenance;
pub(crate) use authorization::{
    RecoveryAuthorizationDecisionV1, RecoveryAuthorizationSignatureV1, RecoveryAuthorizationV1,
    validate_linux_boot_id,
};
pub(crate) use provenance::RecoveryBootstrapProvenanceV1;
#[cfg(test)]
pub(crate) use provenance::RecoveryBootstrapSourceV1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryFileIdentityV1 {
    pub(crate) path: String,
    pub(crate) device: u64,
    pub(crate) inode: u64,
    pub(crate) owner_uid: u32,
    pub(crate) owner_gid: u32,
    pub(crate) mode: u32,
    pub(crate) link_count: u64,
    pub(crate) byte_length: u64,
    pub(crate) mtime_seconds: i64,
    pub(crate) mtime_nanoseconds: i64,
    pub(crate) ctime_seconds: i64,
    pub(crate) ctime_nanoseconds: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoverySourceInspectionV1 {
    pub(crate) binding: ArtifactBinding,
    pub(crate) identity: RecoveryFileIdentityV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryInspectionArtifactsV1 {
    pub(crate) trusted_prefix: ArtifactBinding,
    pub(crate) interrupted_capture: RecoverySourceInspectionV1,
    pub(crate) tainted_generation: RecoverySourceInspectionV1,
    pub(crate) frozen_live_source: RecoverySourceInspectionV1,
}

impl RecoveryInspectionArtifactsV1 {
    pub(crate) fn manifest_artifacts(&self) -> RecoveryArtifacts {
        RecoveryArtifacts {
            trusted_prefix: self.trusted_prefix.clone(),
            interrupted_capture: self.interrupted_capture.binding.clone(),
            tainted_generation: self.tainted_generation.binding.clone(),
            frozen_live_source: self.frozen_live_source.binding.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryInspectionSubjectV1 {
    pub(crate) parent_generation: String,
    pub(crate) incident_at_unix_ms: u64,
    pub(crate) trusted_record_count: u64,
    pub(crate) trusted_projection_inventory: TrustedProjectionInventory,
    pub(crate) discarded_range: ByteRange,
    pub(crate) ambiguous_tail_range: ByteRange,
    pub(crate) ambiguous_tail_sha256: Sha256Digest,
    pub(crate) artifacts: RecoveryInspectionArtifactsV1,
    pub(crate) trusted_state_blake3: String,
    pub(crate) frozen_claims: Vec<FrozenClaimSubject>,
    pub(crate) post_prefix_inventory_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveryInspectionV1 {
    pub(crate) kind: String,
    pub(crate) schema_version: u32,
    pub(crate) inspection_id: String,
    pub(crate) subject: RecoveryInspectionSubjectV1,
}

impl RecoveryInspectionV1 {
    pub(crate) fn from_subject(subject: RecoveryInspectionSubjectV1) -> Result<Self, CoordError> {
        let digest = bullet_wire::hash_canonical(INSPECTION_DOMAIN, &subject).map_err(wire)?;
        let value = Self {
            kind: INSPECTION_KIND.to_owned(),
            schema_version: RECOVERY_INSPECTION_SCHEMA_VERSION,
            inspection_id: format!("rci_{}", digest.to_hex()),
            subject,
        };
        value.validate()?;
        Ok(value)
    }

    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        if self.kind != INSPECTION_KIND || self.schema_version != RECOVERY_INSPECTION_SCHEMA_VERSION
        {
            return Err(invalid(
                "recovery inspection kind or schema version is unsupported",
            ));
        }
        let expected = Self::from_subject_unchecked(&self.subject)?;
        if self.inspection_id != expected {
            return Err(invalid(
                "recovery inspection ID does not bind its exact subject",
            ));
        }
        Ok(())
    }

    pub(crate) fn sealed_sha256(&self) -> Result<Sha256Digest, CoordError> {
        self.validate()?;
        let mut bytes = bullet_wire::canonical_json(self).map_err(wire)?;
        bytes.push(b'\n');
        Ok(Sha256Digest::for_bytes(&bytes))
    }

    fn from_subject_unchecked(subject: &RecoveryInspectionSubjectV1) -> Result<String, CoordError> {
        let digest = bullet_wire::hash_canonical(INSPECTION_DOMAIN, subject).map_err(wire)?;
        Ok(format!("rci_{}", digest.to_hex()))
    }
}

pub(super) fn validate_prefixed(
    value: &str,
    prefix: &str,
    hex_length: usize,
    label: &str,
) -> Result<(), CoordError> {
    let Some(hex) = value.strip_prefix(prefix) else {
        return Err(invalid(format!("{label} has the wrong domain")));
    };
    if hex.len() != hex_length
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid(format!(
            "{label} must contain {hex_length} lowercase hex digits"
        )));
    }
    Ok(())
}

pub(super) fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_MANIFEST_PRODUCTION", reason)
}

fn wire(error: impl std::fmt::Display) -> CoordError {
    invalid(format!("cannot bind recovery inspection: {error}"))
}
