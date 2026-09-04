#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "COMPONENT_ONLY fresh-Genesis contracts await separately reviewed producers"
    )
)]

use serde::{Deserialize, Serialize};

use crate::coord::{CoordError, validate_field};

mod inventory;
pub(crate) use inventory::{
    IncidentDirectoryIdentityV1, IncidentInventoryNodeTypeV1, IncidentInventoryNodeV1,
    IncidentInventorySubjectV1, IncidentInventoryV1,
};

const SCHEMA_VERSION: u32 = 1;
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const MAX_PATH_BYTES: usize = 4_096;
const MAX_COMPONENT_BYTES: usize = 255;
const MAX_SEALED_RECORD_BYTES: u64 = bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES as u64 + 1;
const MAX_CLAIM_LEDGER_BYTES: u64 = 64 * 1024 * 1024;
const MAX_CLAIM_LEDGER_ENTRIES: u64 = 1_000_000;
const WAVE0_FACTS_DOMAIN: &str = "bullet-family.coord.fresh-genesis-wave0-facts.v1";
const WAVE0_SUBJECT_DOMAIN: &str = "bullet-family.coord.fresh-genesis-wave0-subject.v1";
const ADMISSION_REFERENCES_DOMAIN: &str =
    "bullet-family.coord.fresh-genesis-admission-references.v1";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Wave0MemberRoleV1 {
    Hub,
    Kernel,
    BulletGit,
    Portal,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum Wave0CleanStateV1 {
    Clean,
    Dirty,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Wave0SubjectKindV1 {
    Wave0SubjectV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Wave0ReviewKindV1 {
    IndependentReviewV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FreshGenesisRecordKindV1 {
    IncidentInventoryV1,
    Wave0SubjectV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Wave0MemberV1 {
    pub(crate) role: Wave0MemberRoleV1,
    pub(crate) repository_identity: String,
    pub(crate) commit_oid: String,
    pub(crate) tree_oid: String,
    pub(crate) index_state: Wave0CleanStateV1,
    pub(crate) worktree_state: Wave0CleanStateV1,
    pub(crate) untracked_state: Wave0CleanStateV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Wave0ClaimHighWaterV1 {
    pub(crate) claim_ledger_path_hex: String,
    pub(crate) claim_ledger_sha256: String,
    pub(crate) claim_projection_blake3: String,
    pub(crate) byte_length: u64,
    pub(crate) entry_count: u64,
    pub(crate) active_claim_count: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Wave0FactsV1 {
    pub(crate) producer_principal: String,
    pub(crate) claim_high_water: Wave0ClaimHighWaterV1,
    pub(crate) members: Vec<Wave0MemberV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Wave0ReviewBindingV1 {
    kind: Wave0ReviewKindV1,
    pub(crate) reviewer_principal: String,
    pub(crate) reviewed_facts_blake3: String,
    pub(crate) review_record_path_hex: String,
    pub(crate) review_record_sha256: String,
    pub(crate) review_record_byte_length: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Wave0SubjectV1 {
    kind: Wave0SubjectKindV1,
    schema_version: u32,
    pub(crate) subject_id: String,
    pub(crate) facts: Wave0FactsV1,
    pub(crate) review: Wave0ReviewBindingV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FreshGenesisSealedRecordRefV1 {
    pub(crate) record_kind: FreshGenesisRecordKindV1,
    pub(crate) absolute_path_hex: String,
    pub(crate) record_id: String,
    pub(crate) sealed_sha256: String,
    pub(crate) byte_length: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FreshGenesisAdmissionReferencesV1 {
    pub(crate) incident_inventory: FreshGenesisSealedRecordRefV1,
    pub(crate) wave0_subject: FreshGenesisSealedRecordRefV1,
}

impl Wave0SubjectV1 {
    pub(crate) fn from_reviewed(
        facts: Wave0FactsV1,
        reviewer_principal: String,
        review_record_path_hex: String,
        review_record_sha256: String,
        review_record_byte_length: u64,
    ) -> Result<Self, CoordError> {
        facts.validate()?;
        let review = Wave0ReviewBindingV1 {
            kind: Wave0ReviewKindV1::IndependentReviewV1,
            reviewer_principal,
            reviewed_facts_blake3: facts_digest(&facts)?,
            review_record_path_hex,
            review_record_sha256,
            review_record_byte_length,
        };
        review.validate(&facts)?;
        let mut value = Self {
            kind: Wave0SubjectKindV1::Wave0SubjectV1,
            schema_version: SCHEMA_VERSION,
            subject_id: String::new(),
            facts,
            review,
        };
        value.subject_id = value.expected_id()?;
        value.validate()?;
        Ok(value)
    }

    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        if self.kind != Wave0SubjectKindV1::Wave0SubjectV1 || self.schema_version != SCHEMA_VERSION
        {
            return Err(invalid("W0 kind or schema version is unsupported"));
        }
        self.facts.validate()?;
        self.review.validate(&self.facts)?;
        if self.subject_id != self.expected_id()? {
            return Err(invalid(
                "W0 subject ID differs from its exact reviewed facts",
            ));
        }
        canonical_bound(self, "W0 subject")
    }

    fn expected_id(&self) -> Result<String, CoordError> {
        domain_id("w0_", WAVE0_SUBJECT_DOMAIN, &(&self.facts, &self.review))
    }
}

impl Wave0FactsV1 {
    fn validate(&self) -> Result<(), CoordError> {
        validate_principal("W0 producer", &self.producer_principal)?;
        self.claim_high_water.validate()?;
        let expected = [
            (Wave0MemberRoleV1::Hub, "root/bullet-farm"),
            (Wave0MemberRoleV1::Kernel, "root/bullet-kernel"),
            (Wave0MemberRoleV1::BulletGit, "root/bullet-git"),
            (Wave0MemberRoleV1::Portal, "root/bullet-portal"),
        ];
        if self.members.len() != expected.len() {
            return Err(invalid("W0 must contain exactly four family members"));
        }
        for (member, (role, identity)) in self.members.iter().zip(expected) {
            member.validate(role, identity)?;
        }
        Ok(())
    }
}

impl Wave0MemberV1 {
    fn validate(&self, role: Wave0MemberRoleV1, identity: &str) -> Result<(), CoordError> {
        if self.role != role || self.repository_identity != identity {
            return Err(invalid(
                "W0 member order or canonical repository identity differs",
            ));
        }
        validate_tagged_digest(&self.commit_oid, "sha1:", 40, "commit OID")?;
        validate_tagged_digest(&self.tree_oid, "sha1:", 40, "tree OID")?;
        if self.index_state != Wave0CleanStateV1::Clean
            || self.worktree_state != Wave0CleanStateV1::Clean
            || self.untracked_state != Wave0CleanStateV1::Clean
        {
            return Err(invalid(
                "W0 admits only clean index, worktree, and untracked states",
            ));
        }
        Ok(())
    }
}

impl Wave0ClaimHighWaterV1 {
    fn validate(&self) -> Result<(), CoordError> {
        validate_absolute_path_hex(&self.claim_ledger_path_hex, "claim ledger path")?;
        validate_tagged_digest(
            &self.claim_ledger_sha256,
            "sha256:",
            64,
            "claim ledger SHA-256",
        )?;
        validate_tagged_digest(
            &self.claim_projection_blake3,
            "blake3:",
            64,
            "claim projection BLAKE3",
        )?;
        safe(self.byte_length, "claim ledger byte length")?;
        safe(self.entry_count, "claim ledger entry count")?;
        safe(self.active_claim_count, "active claim count")?;
        if self.byte_length > MAX_CLAIM_LEDGER_BYTES
            || self.entry_count > MAX_CLAIM_LEDGER_ENTRIES
            || self.active_claim_count != 0
            || ((self.byte_length == 0) != (self.entry_count == 0))
        {
            return Err(invalid(
                "W0 claim high-water must be internally consistent and have zero active claims",
            ));
        }
        Ok(())
    }
}

impl Wave0ReviewBindingV1 {
    fn validate(&self, facts: &Wave0FactsV1) -> Result<(), CoordError> {
        validate_principal("W0 reviewer", &self.reviewer_principal)?;
        validate_absolute_path_hex(&self.review_record_path_hex, "W0 review record path")?;
        validate_tagged_digest(
            &self.review_record_sha256,
            "sha256:",
            64,
            "W0 review record SHA-256",
        )?;
        safe(
            self.review_record_byte_length,
            "W0 review record byte length",
        )?;
        if self.reviewer_principal == facts.producer_principal
            || self.reviewed_facts_blake3 != facts_digest(facts)?
            || self.review_record_byte_length == 0
            || self.review_record_byte_length > MAX_SEALED_RECORD_BYTES
        {
            return Err(invalid(
                "W0 review must be independent and bind the exact facts digest",
            ));
        }
        Ok(())
    }
}

impl FreshGenesisSealedRecordRefV1 {
    fn validate(&self) -> Result<(), CoordError> {
        validate_absolute_path_hex(&self.absolute_path_hex, "sealed record path")?;
        validate_tagged_digest(&self.sealed_sha256, "sha256:", 64, "sealed record SHA-256")?;
        safe(self.byte_length, "sealed record byte length")?;
        if self.byte_length == 0 || self.byte_length > MAX_SEALED_RECORD_BYTES {
            return Err(invalid(
                "sealed record byte length exceeds its closed framing bound",
            ));
        }
        let prefix = match self.record_kind {
            FreshGenesisRecordKindV1::IncidentInventoryV1 => "fgi_",
            FreshGenesisRecordKindV1::Wave0SubjectV1 => "w0_",
        };
        validate_tagged_digest(&self.record_id, prefix, 64, "sealed record ID")
    }
}

impl FreshGenesisAdmissionReferencesV1 {
    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        self.incident_inventory.validate()?;
        self.wave0_subject.validate()?;
        if self.incident_inventory.record_kind != FreshGenesisRecordKindV1::IncidentInventoryV1
            || self.wave0_subject.record_kind != FreshGenesisRecordKindV1::Wave0SubjectV1
            || self.incident_inventory.absolute_path_hex == self.wave0_subject.absolute_path_hex
        {
            return Err(invalid(
                "fresh-Genesis references require distinct inventory and W0 records",
            ));
        }
        canonical_bound(self, "fresh-Genesis admission references")
    }

    pub(crate) fn subject_blake3(&self) -> Result<String, CoordError> {
        self.validate()?;
        Ok(format!(
            "blake3:{}",
            bullet_wire::hash_canonical(ADMISSION_REFERENCES_DOMAIN, self)
                .map_err(wire)?
                .to_hex()
        ))
    }
}

fn facts_digest(facts: &Wave0FactsV1) -> Result<String, CoordError> {
    Ok(format!(
        "blake3:{}",
        bullet_wire::hash_canonical(WAVE0_FACTS_DOMAIN, facts)
            .map_err(wire)?
            .to_hex()
    ))
}

pub(super) fn domain_id(
    prefix: &str,
    domain: &str,
    value: &impl Serialize,
) -> Result<String, CoordError> {
    Ok(format!(
        "{prefix}{}",
        bullet_wire::hash_canonical(domain, value)
            .map_err(wire)?
            .to_hex()
    ))
}

pub(super) fn validate_tagged_digest(
    value: &str,
    prefix: &str,
    hex_length: usize,
    label: &str,
) -> Result<(), CoordError> {
    let valid = value.strip_prefix(prefix).is_some_and(|hex| {
        hex.len() == hex_length
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    });
    valid
        .then_some(())
        .ok_or_else(|| invalid(format!("{label} has a noncanonical digest identity")))
}

pub(super) fn validate_absolute_path_hex(value: &str, label: &str) -> Result<(), CoordError> {
    let bytes = decode_path_hex(value, label)?;
    if bytes == b"/" || bytes.first() != Some(&b'/') || bytes.last() == Some(&b'/') {
        return Err(invalid(format!("{label} must be a non-root absolute path")));
    }
    validate_segments(&bytes[1..], label)
}

pub(super) fn validate_relative_path_hex(value: &str, label: &str) -> Result<(), CoordError> {
    let bytes = decode_path_hex(value, label)?;
    if bytes.first() == Some(&b'/') || bytes.last() == Some(&b'/') {
        return Err(invalid(format!(
            "{label} must be a normalized relative path"
        )));
    }
    validate_segments(&bytes, label)
}

pub(super) fn validate_destination_name_hex(value: &str) -> Result<(), CoordError> {
    let bytes = decode_path_hex(value, "incident destination name")?;
    if bytes.len() > MAX_COMPONENT_BYTES
        || bytes.contains(&b'/')
        || matches!(bytes.as_slice(), b"." | b"..")
    {
        return Err(invalid(
            "incident destination name must be one normalized path component",
        ));
    }
    Ok(())
}

fn decode_path_hex(value: &str, label: &str) -> Result<Vec<u8>, CoordError> {
    if value.is_empty()
        || !value.len().is_multiple_of(2)
        || value.len() > MAX_PATH_BYTES * 2
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(invalid(format!(
            "{label} must be bounded lowercase hexadecimal path bytes"
        )));
    }
    let mut bytes = Vec::with_capacity(value.len() / 2);
    for pair in value.as_bytes().chunks_exact(2) {
        let text = std::str::from_utf8(pair)
            .map_err(|_| invalid(format!("{label} contains invalid hexadecimal")))?;
        bytes.push(
            u8::from_str_radix(text, 16)
                .map_err(|_| invalid(format!("{label} contains invalid hexadecimal")))?,
        );
    }
    if bytes.contains(&0) {
        return Err(invalid(format!("{label} contains a NUL byte")));
    }
    Ok(bytes)
}

fn validate_segments(bytes: &[u8], label: &str) -> Result<(), CoordError> {
    if bytes.split(|byte| *byte == b'/').any(|segment| {
        segment.is_empty() || segment.len() > MAX_COMPONENT_BYTES || matches!(segment, b"." | b"..")
    }) {
        return Err(invalid(format!(
            "{label} has an empty, dot, or oversized segment"
        )));
    }
    Ok(())
}

pub(super) fn validate_mode(mode: u32, label: &str) -> Result<(), CoordError> {
    if mode > 0o7777 {
        Err(invalid(format!(
            "{label} exceeds the closed permission-mode range"
        )))
    } else {
        Ok(())
    }
}

pub(super) fn safe(value: u64, label: &str) -> Result<(), CoordError> {
    if value > MAX_SAFE_INTEGER {
        Err(invalid(format!("{label} is not a JSON-safe integer")))
    } else {
        Ok(())
    }
}

pub(super) fn canonical_bound(value: &impl Serialize, label: &str) -> Result<(), CoordError> {
    let bytes = bullet_wire::canonical_json(value).map_err(wire)?;
    if bytes.len() > bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES {
        Err(invalid(format!(
            "{label} exceeds the canonical document bound"
        )))
    } else {
        Ok(())
    }
}

fn validate_principal(label: &str, value: &str) -> Result<(), CoordError> {
    validate_field(label, value).map_err(|error| invalid(error.to_string()))
}

pub(super) fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_FRESH_GENESIS_SUBJECT", reason)
}

fn wire(error: impl std::fmt::Display) -> CoordError {
    invalid(format!(
        "cannot bind fresh-Genesis canonical subject: {error}"
    ))
}

#[cfg(test)]
mod tests;
