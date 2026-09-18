use std::collections::BTreeSet;

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::*;
use crate::coord::{validate_field, validate_path, validate_repo_name};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const MAX_CLAIMS: usize = 64;
const MAX_LEAVES: usize = 256;
const MAX_RAW_OBJECT_BYTES: usize = 512 * 1024;
const ADOPTION_DOMAIN: &str = "bullet-family.coord.recovery-receipt-adoption.v1";
const REQUEST_DOMAIN: &str = "bullet-family.coord.recovery-receipt-adoption-request.v1";
const PROOF_DOMAIN: &str = "bullet-family.coord.recovery-proof-subject.v1";
const REVIEW_DOMAIN: &str = "bullet-family.coord.recovery-review-subject.v1";

#[derive(Serialize)]
struct AdoptionIdentity<'a> {
    generation_id: &'a str,
    repo: &'a str,
    commit_oid: &'a str,
    claims: Vec<AdoptionClaimIdentity<'a>>,
}

#[derive(Serialize)]
struct AdoptionClaimIdentity<'a> {
    claim_id: &'a str,
    committed_paths: &'a [String],
}

impl RecoveryReceiptAdoptionRequestV1 {
    pub fn validate(&self) -> Result<(), CoordError> {
        if self.schema_version != RECOVERY_ADOPTION_REQUEST_SCHEMA_VERSION {
            return Err(invalid("request schema version is unsupported"));
        }
        self.request_id.validate()?;
        self.expected_watermark.validate()?;
        self.subject.validate()?;
        self.validate_evidence_refs()?;
        let canonical = bullet_wire::canonical_json(self).map_err(wire)?;
        if canonical.len() > bullet_wire::MAX_CANONICAL_DOCUMENT_BYTES {
            return Err(invalid("canonical adoption request exceeds its wire bound"));
        }
        Ok(())
    }

    fn validate_evidence_refs(&self) -> Result<(), CoordError> {
        let watermark = &self.expected_watermark;
        let mut previous_sequence = None;
        let mut request_ids = BTreeSet::new();
        for proof in &self.subject.proof_observations {
            proof.record.validate_against(watermark)?;
            if previous_sequence.is_some_and(|sequence| sequence >= proof.record.sequence) {
                return Err(invalid(
                    "proof receipt references must have distinct increasing sequences",
                ));
            }
            if !request_ids.insert(proof.record.request_id.as_str()) {
                return Err(invalid(
                    "proof receipt references must have distinct request IDs",
                ));
            }
            previous_sequence = Some(proof.record.sequence);
        }
        let review = &self.subject.review_observation.record;
        review.validate_against(watermark)?;
        if previous_sequence.is_some_and(|sequence| sequence >= review.sequence)
            || !request_ids.insert(review.request_id.as_str())
        {
            return Err(invalid(
                "review receipt must follow every proof and use a distinct request ID",
            ));
        }
        Ok(())
    }

    pub fn request_subject_blake3(&self) -> Result<String, CoordError> {
        self.validate()?;
        digest(REQUEST_DOMAIN, self)
    }

    pub fn adoption_id(&self) -> Result<String, CoordError> {
        self.validate()?;
        let subject = &self.subject;
        let identity = AdoptionIdentity {
            generation_id: &self.expected_watermark.generation_id,
            repo: &subject.repo,
            commit_oid: &subject.git_expectation.commit_oid,
            claims: subject
                .claims
                .iter()
                .map(|claim| AdoptionClaimIdentity {
                    claim_id: &claim.claim_id,
                    committed_paths: &claim.committed_paths,
                })
                .collect(),
        };
        Ok(format!(
            "rad_{}",
            bullet_wire::hash_canonical(ADOPTION_DOMAIN, &identity)
                .map_err(wire)?
                .to_hex()
        ))
    }
}

impl RecoveryAdoptionWatermarkV1 {
    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        tagged(&self.generation_id, "gen_")?;
        tagged(&self.manifest_blake3, "blake3:")?;
        bare_blake3(&self.head_envelope_blake3)?;
        bare_blake3(&self.last_record_blake3)?;
        self.last_request_id.validate()?;
        bare_blake3(&self.last_request_blake3)?;
        safe(self.last_sequence, "last sequence")?;
        safe(self.next_sequence, "next sequence")?;
        safe(self.byte_length, "byte length")?;
        if self.last_sequence == 0
            || self.byte_length == 0
            || self.next_sequence != self.last_sequence.saturating_add(1)
        {
            return Err(invalid("expected watermark is internally inconsistent"));
        }
        Ok(())
    }
}

impl RecoveryReceiptAdoptionSubjectV1 {
    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        validate_base_fields(
            &self.repo,
            &self.git_expectation,
            &self.claims,
            &self.group_receipt_observation,
        )?;
        bounded(
            &self.proof_observations,
            MAX_RECOVERY_PROOF_RECEIPTS,
            "proof observations",
        )?;
        sorted_unique(&self.proof_observations, "proof observations")?;
        for proof in &self.proof_observations {
            proof.validate()?;
        }
        self.review_observation.validate()?;
        Ok(())
    }

    pub(crate) fn proof_subject_blake3(&self) -> Result<String, CoordError> {
        digest(PROOF_DOMAIN, &self.proof_observations)
    }

    pub(crate) fn review_subject_blake3(&self) -> Result<String, CoordError> {
        digest(REVIEW_DOMAIN, &self.review_observation)
    }
}

pub(in crate::coord::model) fn validate_base_fields(
    repo: &str,
    git_expectation: &RecoveryGitExpectationV1,
    claims: &[RecoveryAdoptionClaimV1],
    group_receipt_observation: &ForensicRecordRefV1,
) -> Result<(), CoordError> {
    validate_repo_name(repo).map_err(as_invalid)?;
    git_expectation.validate()?;
    bounded(claims, MAX_CLAIMS, "claims")?;
    if claims.len() < 2 {
        return Err(invalid(
            "grouped recovery adoption requires at least two claims",
        ));
    }
    sorted_unique_by(claims, |claim| claim.claim_id.as_str(), "claims")?;
    group_receipt_observation.validate()?;
    if group_receipt_observation.artifact_kind != RecoveryForensicArtifactKindV1::FrozenLiveSource
        || group_receipt_observation.expected_record_kind
            != RecoveryForensicRecordKindV1::CommitReceiptGroup
    {
        return Err(invalid(
            "group receipt observation has the wrong forensic kind",
        ));
    }
    let mut partitions = Vec::new();
    for claim in claims {
        claim.validate()?;
        partitions.extend(claim.committed_paths.iter().cloned());
    }
    let mut sorted = partitions.clone();
    sorted.sort();
    if partitions.len() != sorted.len()
        || sorted.windows(2).any(|pair| pair[0] == pair[1])
        || sorted
            != git_expectation
                .leaf_transitions
                .iter()
                .map(|leaf| leaf.path.clone())
                .collect::<Vec<_>>()
    {
        return Err(invalid(
            "claim partitions must be disjoint and exactly cover Git leaves",
        ));
    }
    Ok(())
}

impl RecoveryAdoptionClaimV1 {
    fn validate(&self) -> Result<(), CoordError> {
        tagged(&self.claim_id, "clm_")?;
        tagged(&self.frozen_claim_blake3, "blake3:")?;
        self.trusted_claim_record.validate()?;
        self.handoff_observation.validate()?;
        if self.trusted_claim_record.artifact_kind != RecoveryForensicArtifactKindV1::TrustedPrefix
            || self.trusted_claim_record.expected_record_kind != RecoveryForensicRecordKindV1::Claim
            || self.handoff_observation.artifact_kind
                != RecoveryForensicArtifactKindV1::FrozenLiveSource
            || self.handoff_observation.expected_record_kind
                != RecoveryForensicRecordKindV1::Handoff
        {
            return Err(invalid("claim forensic references have the wrong kinds"));
        }
        bounded(&self.committed_paths, MAX_LEAVES, "committed paths")?;
        if !strictly_sorted(&self.committed_paths) {
            return Err(invalid("committed paths must be sorted and unique"));
        }
        for path in &self.committed_paths {
            validate_path(path).map_err(as_invalid)?;
        }
        Ok(())
    }
}

impl ForensicRecordRefV1 {
    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        tagged(&self.artifact_sha256, "sha256:")?;
        tagged(&self.record_sha256, "sha256:")?;
        safe(self.record_index, "record index")?;
        safe(self.byte_start, "record byte start")?;
        safe(self.byte_end, "record byte end")?;
        if self.record_index == 0 || self.byte_end <= self.byte_start {
            return Err(invalid(
                "forensic byte range must be nonempty and end-exclusive",
            ));
        }
        Ok(())
    }
}

impl RecoveryGitExpectationV1 {
    fn validate(&self) -> Result<(), CoordError> {
        tagged(&self.commit_oid, "sha1:")?;
        tagged(&self.parent_oid, "sha1:")?;
        tagged(&self.parent_tree_oid, "sha1:")?;
        self.parent_receipt_observation.validate()?;
        if self.parent_receipt_observation.artifact_kind
            != RecoveryForensicArtifactKindV1::TrustedPrefix
            || self.parent_receipt_observation.expected_record_kind
                != RecoveryForensicRecordKindV1::CommitReceipt
        {
            return Err(invalid(
                "parent receipt observation has the wrong forensic kind",
            ));
        }
        tagged(&self.result_tree_oid, "sha1:")?;
        bytes(&self.raw_commit_bytes, "raw commit")?;
        sha256(
            &self.raw_commit_bytes,
            &self.raw_commit_sha256,
            "raw commit",
        )?;
        tagged(&self.raw_tree_sha256, "sha256:")?;
        bounded(&self.leaf_transitions, MAX_LEAVES, "Git leaf transitions")?;
        sorted_unique_by(
            &self.leaf_transitions,
            |leaf| leaf.path.as_str(),
            "Git leaf transitions",
        )?;
        for leaf in &self.leaf_transitions {
            leaf.validate()?;
        }
        Ok(())
    }
}

impl RecoveryGitLeafTransitionV1 {
    fn validate(&self) -> Result<(), CoordError> {
        validate_path(&self.path).map_err(as_invalid)?;
        if !matches!(self.old_mode.as_str(), "100644" | "100755")
            || !matches!(self.new_mode.as_str(), "100644" | "100755")
        {
            return Err(invalid("Git leaf mode is not a regular-file mode"));
        }
        tagged(&self.old_blob_oid, "sha1:")?;
        tagged(&self.new_blob_oid, "sha1:")?;
        bytes(&self.old_bytes, "prior blob")?;
        bytes(&self.new_bytes, "new blob")?;
        sha256(&self.old_bytes, &self.old_sha256, "prior blob")?;
        sha256(&self.new_bytes, &self.new_sha256, "new blob")?;
        if self.old_mode == self.new_mode && self.old_blob_oid == self.new_blob_oid {
            return Err(invalid(
                "modified Git leaf has no object or mode transition",
            ));
        }
        Ok(())
    }
}

impl RecoveryReceiptAdoptionRecordV1 {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::coord) fn verified(
        request: RecoveryReceiptAdoptionRequestV1,
        recovery_operator: String,
        recovery_policy_sha256: String,
        operator_decision_sha256: String,
        replay_contract_version: u32,
        replay_contract_sha256: String,
        verified_orchestrator: String,
        verified_reviewer: String,
    ) -> Result<Self, CoordError> {
        request.validate()?;
        let value = Self {
            adoption_id: request.adoption_id()?,
            request_subject_blake3: request.request_subject_blake3()?,
            proof_subject_blake3: request.subject.proof_subject_blake3()?,
            review_subject_blake3: request.subject.review_subject_blake3()?,
            request,
            recovery_operator,
            recovery_policy_sha256,
            operator_decision_sha256,
            replay_contract_version,
            replay_contract_sha256,
            authority_class: RecoveryAdoptionAuthorityClassV1::LocalOsAuthority,
            verified_orchestrator,
            verified_reviewer,
        };
        value.validate()?;
        Ok(value)
    }

    pub(in crate::coord) fn adoption_id(&self) -> &str {
        &self.adoption_id
    }

    pub(in crate::coord) fn request_subject_blake3(&self) -> &str {
        &self.request_subject_blake3
    }

    pub(in crate::coord) fn request(&self) -> &RecoveryReceiptAdoptionRequestV1 {
        &self.request
    }

    pub(in crate::coord) fn verified_orchestrator(&self) -> &str {
        &self.verified_orchestrator
    }

    pub(in crate::coord) fn verified_reviewer(&self) -> &str {
        &self.verified_reviewer
    }

    pub(in crate::coord) fn proof_subject_blake3(&self) -> &str {
        &self.proof_subject_blake3
    }

    pub(in crate::coord) fn review_subject_blake3(&self) -> &str {
        &self.review_subject_blake3
    }

    pub(crate) fn validate(&self) -> Result<(), CoordError> {
        self.request.validate()?;
        tagged(&self.adoption_id, "rad_")?;
        tagged(&self.request_subject_blake3, "blake3:")?;
        tagged(&self.recovery_policy_sha256, "sha256:")?;
        tagged(&self.operator_decision_sha256, "sha256:")?;
        tagged(&self.replay_contract_sha256, "sha256:")?;
        tagged(&self.proof_subject_blake3, "blake3:")?;
        tagged(&self.review_subject_blake3, "blake3:")?;
        validate_field("recovery_operator", &self.recovery_operator).map_err(as_invalid)?;
        validate_field("verified_orchestrator", &self.verified_orchestrator).map_err(as_invalid)?;
        validate_field("verified_reviewer", &self.verified_reviewer).map_err(as_invalid)?;
        if self.replay_contract_version == 0
            || self.verified_orchestrator == self.verified_reviewer
            || self.adoption_id != self.request.adoption_id()?
            || self.request_subject_blake3 != self.request.request_subject_blake3()?
            || self.proof_subject_blake3 != self.request.subject.proof_subject_blake3()?
            || self.review_subject_blake3 != self.request.subject.review_subject_blake3()?
        {
            return Err(invalid(
                "verified recovery adoption provenance is inconsistent",
            ));
        }
        Ok(())
    }
}

fn sha256(bytes: &[u8], expected: &str, label: &str) -> Result<(), CoordError> {
    tagged(expected, "sha256:")?;
    if expected != format!("sha256:{:x}", Sha256::digest(bytes)) {
        return Err(invalid(format!("{label} SHA-256 differs from its bytes")));
    }
    Ok(())
}

fn bytes(value: &[u8], label: &str) -> Result<(), CoordError> {
    if value.is_empty() || value.len() > MAX_RAW_OBJECT_BYTES {
        return Err(invalid(format!("{label} exceeds its closed byte bound")));
    }
    Ok(())
}

pub(super) fn tagged(value: &str, prefix: &str) -> Result<(), CoordError> {
    let expected = if prefix == "sha1:" { 40 } else { 64 };
    let valid = value.strip_prefix(prefix).is_some_and(|hex| {
        hex.len() == expected
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    });
    valid.then_some(()).ok_or_else(|| {
        invalid(format!(
            "identity must be {prefix} plus {expected} lowercase hexadecimal digits"
        ))
    })
}

pub(super) fn bare_blake3(value: &str) -> Result<(), CoordError> {
    let valid = value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    valid
        .then_some(())
        .ok_or_else(|| invalid("schema-2 ledger digest must be 64 lowercase hexadecimal digits"))
}

fn safe(value: u64, label: &str) -> Result<(), CoordError> {
    if value > MAX_SAFE_INTEGER {
        Err(invalid(format!("{label} is not a JSON-safe integer")))
    } else {
        Ok(())
    }
}

fn bounded<T>(values: &[T], maximum: usize, label: &str) -> Result<(), CoordError> {
    if values.is_empty() || values.len() > maximum {
        Err(invalid(format!("{label} count is outside 1..={maximum}")))
    } else {
        Ok(())
    }
}

fn sorted_unique(values: &[impl Ord], label: &str) -> Result<(), CoordError> {
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        Err(invalid(format!("{label} must be sorted and unique")))
    } else {
        Ok(())
    }
}

fn sorted_unique_by<T>(
    values: &[T],
    key: impl Fn(&T) -> &str,
    label: &str,
) -> Result<(), CoordError> {
    if values.windows(2).any(|pair| key(&pair[0]) >= key(&pair[1])) {
        Err(invalid(format!("{label} must be sorted and unique")))
    } else {
        Ok(())
    }
}

fn strictly_sorted(values: &[String]) -> bool {
    !values.is_empty() && values.windows(2).all(|pair| pair[0] < pair[1])
}

fn digest(domain: &str, value: &impl Serialize) -> Result<String, CoordError> {
    Ok(format!(
        "blake3:{}",
        bullet_wire::hash_canonical(domain, value)
            .map_err(wire)?
            .to_hex()
    ))
}

fn wire(error: bullet_wire::WireError) -> CoordError {
    invalid(format!("canonical adoption subject failed: {error}"))
}

fn as_invalid(error: CoordError) -> CoordError {
    invalid(error.to_string())
}

pub(super) fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_ADOPTION", reason)
}
