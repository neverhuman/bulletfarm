#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "COMPONENT_ONLY recovered W0 facts await an accepted signed review contract"
    )
)]

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{
    COORD_SCHEMA_VERSION, ClaimState, ClaimSummary, CoordError, CoordStore,
    RecoveryProductionWatermarkV1, Status, StatusOrigin,
    git::wave0::{Wave0MechanicalObservation, observe_wave0_mechanical},
    model::{Wave0CleanStateV1, Wave0MemberRoleV1, Wave0MemberV1},
};

const SCHEMA_VERSION: u32 = 1;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const FACTS_DOMAIN: &str = "bullet-family.coord.recovered-wave0-facts.v1";
const CLAIM_PROJECTION_DOMAIN: &str = "bullet-family.coord.recovered-wave0-claim-projection.v1";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum RecoveredWave0FactsKindV1 {
    RecoveredWave0FactsV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveredWave0OriginV1 {
    pub(crate) incident_at_unix_ms: u64,
    pub(crate) recovered_at_unix_ms: u64,
    pub(crate) trusted_records: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveredWave0ClaimProjectionV1 {
    pub(crate) claim_projection_blake3: String,
    pub(crate) total: u64,
    pub(crate) active: u64,
    pub(crate) expired: u64,
    pub(crate) handed_off_unreceipted: u64,
    pub(crate) handed_off_receipted: u64,
    pub(crate) frozen_recovery: u64,
    pub(crate) recovered_receipted: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecoveredWave0FactsV1 {
    pub(crate) kind: RecoveredWave0FactsKindV1,
    pub(crate) schema_version: u32,
    pub(crate) facts_blake3: String,
    pub(crate) coord_schema_version: u32,
    pub(crate) origin: RecoveredWave0OriginV1,
    pub(crate) watermark: RecoveryProductionWatermarkV1,
    pub(crate) claim_projection: RecoveredWave0ClaimProjectionV1,
    pub(crate) members: Vec<Wave0MemberV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NormalizedStatus {
    coord_schema_version: u32,
    origin: RecoveredWave0OriginV1,
    watermark: RecoveryProductionWatermarkV1,
    claim_projection: RecoveredWave0ClaimProjectionV1,
}

#[derive(Serialize)]
struct FactsIdentity<'a> {
    kind: RecoveredWave0FactsKindV1,
    schema_version: u32,
    coord_schema_version: u32,
    origin: &'a RecoveredWave0OriginV1,
    watermark: &'a RecoveryProductionWatermarkV1,
    claim_projection: &'a RecoveredWave0ClaimProjectionV1,
    members: &'a [Wave0MemberV1],
}

#[cfg_attr(
    test,
    allow(
        dead_code,
        reason = "production observer cannot run against the intentionally frozen live coordinator"
    )
)]
pub(crate) fn observe_recovered_wave0(
    family_root: &Path,
) -> Result<RecoveredWave0FactsV1, CoordError> {
    super::recovery_manifest::require_normalized_absolute(family_root, "recovered W0 family root")?;
    observe_recovered_wave0_with(
        family_root,
        || CoordStore::new(family_root.to_path_buf()).status(),
        || observe_wave0_mechanical(family_root),
    )
}

fn observe_recovered_wave0_with(
    family_root: &Path,
    mut observe_status: impl FnMut() -> Result<Status, CoordError>,
    observe_members: impl FnOnce() -> Result<Wave0MechanicalObservation, CoordError>,
) -> Result<RecoveredWave0FactsV1, CoordError> {
    let first_status = observe_status()?;
    let first = normalize_status(family_root, &first_status)?;
    let mechanical = observe_members()?;
    let second_status = observe_status()?;
    let second = normalize_status(family_root, &second_status)?;
    if second_status.observed_at_unix_ms < first_status.observed_at_unix_ms || first != second {
        return Err(changed(
            "coordinator status changed across recovered W0 observation",
        ));
    }
    RecoveredWave0FactsV1::from_observed(first, mechanical.members.to_vec())
}

impl RecoveredWave0FactsV1 {
    fn from_observed(
        status: NormalizedStatus,
        members: Vec<Wave0MemberV1>,
    ) -> Result<Self, CoordError> {
        let mut value = Self {
            kind: RecoveredWave0FactsKindV1::RecoveredWave0FactsV1,
            schema_version: SCHEMA_VERSION,
            facts_blake3: String::new(),
            coord_schema_version: status.coord_schema_version,
            origin: status.origin,
            watermark: status.watermark,
            claim_projection: status.claim_projection,
            members,
        };
        value.facts_blake3 = value.expected_digest()?;
        value.validate()?;
        Ok(value)
    }

    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        if self.kind != RecoveredWave0FactsKindV1::RecoveredWave0FactsV1
            || self.schema_version != SCHEMA_VERSION
            || self.coord_schema_version != COORD_SCHEMA_VERSION
        {
            return Err(invalid(
                "recovered W0 kind or schema version is unsupported",
            ));
        }
        self.origin.validate()?;
        validate_watermark(&self.watermark)?;
        self.claim_projection.validate()?;
        validate_members(&self.members)?;
        if self.facts_blake3 != self.expected_digest()? {
            return Err(invalid(
                "recovered W0 facts digest differs from its exact canonical facts",
            ));
        }
        if bullet_wire::canonical_json(self).map_err(wire)?.len()
            > bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES
        {
            return Err(invalid(
                "recovered W0 facts exceed the canonical wire bound",
            ));
        }
        Ok(())
    }

    fn expected_digest(&self) -> Result<String, CoordError> {
        Ok(format!(
            "blake3:{}",
            bullet_wire::hash_canonical(
                FACTS_DOMAIN,
                &FactsIdentity {
                    kind: self.kind,
                    schema_version: self.schema_version,
                    coord_schema_version: self.coord_schema_version,
                    origin: &self.origin,
                    watermark: &self.watermark,
                    claim_projection: &self.claim_projection,
                    members: &self.members,
                },
            )
            .map_err(wire)?
            .to_hex()
        ))
    }
}

impl RecoveredWave0OriginV1 {
    fn validate(&self) -> Result<(), CoordError> {
        for (label, value) in [
            ("incident time", self.incident_at_unix_ms),
            ("recovery time", self.recovered_at_unix_ms),
            ("trusted record count", self.trusted_records),
        ] {
            safe(value, label)?;
        }
        if self.incident_at_unix_ms == 0
            || self.recovered_at_unix_ms < self.incident_at_unix_ms
            || self.trusted_records == 0
        {
            return Err(invalid("recovered W0 origin is internally inconsistent"));
        }
        Ok(())
    }
}

impl RecoveredWave0ClaimProjectionV1 {
    fn validate(&self) -> Result<(), CoordError> {
        tagged(
            &self.claim_projection_blake3,
            "blake3:",
            "claim projection digest",
        )?;
        let counts = [
            self.active,
            self.expired,
            self.handed_off_unreceipted,
            self.handed_off_receipted,
            self.frozen_recovery,
            self.recovered_receipted,
        ];
        safe(self.total, "claim total")?;
        let partition = counts.into_iter().try_fold(0_u64, |total, count| {
            safe(count, "claim state count")?;
            total
                .checked_add(count)
                .ok_or_else(|| invalid("claim state partition overflowed"))
        })?;
        if partition != self.total {
            return Err(invalid("claim state partition differs from its total"));
        }
        if self.active != 0 || self.handed_off_unreceipted != 0 || self.frozen_recovery != 0 {
            return Err(open_claims(self));
        }
        Ok(())
    }
}

fn normalize_status(family_root: &Path, status: &Status) -> Result<NormalizedStatus, CoordError> {
    if status.schema_version != COORD_SCHEMA_VERSION {
        return Err(invalid("coordinator status schema version is unsupported"));
    }
    safe(status.observed_at_unix_ms, "status observation time")?;
    if status.observed_at_unix_ms == 0 {
        return Err(invalid("status observation time must be positive"));
    }
    let origin = match status.origin {
        StatusOrigin::Genesis => {
            return Err(CoordError::new(
                "W0_RECOVERY_REQUIRED",
                "recovered W0 refuses a Genesis coordinator generation",
            ));
        }
        StatusOrigin::Recovered {
            incident_at_unix_ms,
            recovered_at_unix_ms,
            trusted_records,
        } => RecoveredWave0OriginV1 {
            incident_at_unix_ms,
            recovered_at_unix_ms,
            trusted_records,
        },
    };
    origin.validate()?;
    let watermark = RecoveryProductionWatermarkV1 {
        generation_id: status.generation_id.clone(),
        manifest_blake3: status.manifest_blake3.clone(),
        last_sequence: status.as_of_sequence,
        next_sequence: status.next_sequence,
        head_envelope_blake3: status.last_envelope_blake3.clone(),
        last_record_blake3: status.last_record_blake3.clone(),
        last_request_id: status.last_request_id.clone(),
        last_request_blake3: status.last_request_blake3.clone(),
        byte_length: status.byte_length,
    };
    validate_watermark(&watermark)?;
    let expected_source = family_root
        .join(".bullet-family/coord/generations")
        .join(&status.generation_id)
        .join("events.jsonl");
    if status.source != expected_source.to_string_lossy() {
        return Err(changed(
            "coordinator status source is not the exact recovered generation ledger",
        ));
    }
    let claim_projection = project_claims(&status.claims)?;
    claim_projection.validate()?;
    Ok(NormalizedStatus {
        coord_schema_version: status.schema_version,
        origin,
        watermark,
        claim_projection,
    })
}

fn project_claims(claims: &[ClaimSummary]) -> Result<RecoveredWave0ClaimProjectionV1, CoordError> {
    let mut sorted = claims.to_vec();
    sorted.sort_by(|left, right| left.claim_id.cmp(&right.claim_id));
    if sorted
        .windows(2)
        .any(|pair| pair[0].claim_id == pair[1].claim_id)
    {
        return Err(invalid("claim projection contains duplicate claim IDs"));
    }
    let mut value = RecoveredWave0ClaimProjectionV1 {
        claim_projection_blake3: format!(
            "blake3:{}",
            bullet_wire::hash_canonical(CLAIM_PROJECTION_DOMAIN, &sorted)
                .map_err(wire)?
                .to_hex()
        ),
        total: u64::try_from(sorted.len())
            .map_err(|_| invalid("claim projection does not fit its wire count"))?,
        active: 0,
        expired: 0,
        handed_off_unreceipted: 0,
        handed_off_receipted: 0,
        frozen_recovery: 0,
        recovered_receipted: 0,
    };
    for claim in &sorted {
        let count = match claim.state {
            ClaimState::Active => &mut value.active,
            ClaimState::Expired => &mut value.expired,
            ClaimState::HandedOff
                if claim.commit_oid.is_some()
                    && claim.commit_orchestrator.is_some()
                    && claim.commit_recorded_at_unix_ms.is_some() =>
            {
                &mut value.handed_off_receipted
            }
            ClaimState::HandedOff => &mut value.handed_off_unreceipted,
            ClaimState::FrozenRecovery => &mut value.frozen_recovery,
            ClaimState::RecoveredReceipted => &mut value.recovered_receipted,
        };
        *count = count
            .checked_add(1)
            .ok_or_else(|| invalid("claim state count overflowed"))?;
    }
    value.validate()?;
    Ok(value)
}

fn validate_members(members: &[Wave0MemberV1]) -> Result<(), CoordError> {
    let expected = [
        (Wave0MemberRoleV1::Hub, "root/bullet-farm"),
        (Wave0MemberRoleV1::Kernel, "root/bullet-kernel"),
        (Wave0MemberRoleV1::BulletGit, "root/bullet-git"),
        (Wave0MemberRoleV1::Portal, "root/bullet-portal"),
    ];
    if members.len() != expected.len() {
        return Err(invalid(
            "recovered W0 must contain exactly four family members",
        ));
    }
    for (member, (role, identity)) in members.iter().zip(expected) {
        if member.role != role || member.repository_identity != identity {
            return Err(invalid(
                "recovered W0 member order or repository identity differs",
            ));
        }
        tagged(&member.commit_oid, "sha1:", "member commit OID")?;
        tagged(&member.tree_oid, "sha1:", "member tree OID")?;
        if member.index_state != Wave0CleanStateV1::Clean
            || member.worktree_state != Wave0CleanStateV1::Clean
            || member.untracked_state != Wave0CleanStateV1::Clean
        {
            return Err(invalid("recovered W0 admits only clean family members"));
        }
    }
    Ok(())
}

fn open_claims(value: &RecoveredWave0ClaimProjectionV1) -> CoordError {
    CoordError::new(
        "W0_CLAIMS_OPEN",
        format!(
            "recovered W0 refuses {} active, {} unreceipted handoff, and {} frozen claim(s)",
            value.active, value.handed_off_unreceipted, value.frozen_recovery
        ),
    )
}

fn validate_watermark(value: &RecoveryProductionWatermarkV1) -> Result<(), CoordError> {
    value
        .validate()
        .map_err(|error| invalid(format!("recovered W0 watermark is invalid: {error}")))
}

fn tagged(value: &str, prefix: &str, label: &str) -> Result<(), CoordError> {
    let width = if prefix == "sha1:" { 40 } else { 64 };
    let valid = value.strip_prefix(prefix).is_some_and(|hex| {
        hex.len() == width
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    });
    valid
        .then_some(())
        .ok_or_else(|| invalid(format!("{label} is not a full-width tagged digest")))
}

fn safe(value: u64, label: &str) -> Result<(), CoordError> {
    (value <= MAX_SAFE_INTEGER)
        .then_some(())
        .ok_or_else(|| invalid(format!("{label} exceeds the JSON safe-integer bound")))
}

fn wire(error: bullet_wire::WireError) -> CoordError {
    invalid(format!("recovered W0 canonicalization failed: {error}"))
}

fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("W0_SUBJECT_CHANGED", reason)
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERED_WAVE0_FACTS", reason)
}

#[cfg(test)]
#[path = "recovered_wave0/tests.rs"]
mod tests;
