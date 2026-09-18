use std::collections::{BTreeMap, BTreeSet};

use sha2::{Digest, Sha256};

use super::{ForensicSources, mismatch};
use crate::coord::{
    ClaimState, CoordError,
    generation::manifest::{ArtifactBinding, RecoveryManifestBody},
    model::{
        ClaimSummary, ForensicRecordRefV1, GroupReceipt, LEGACY_SCHEMA_VERSION, Record,
        RecoveryAdoptionClaimV1, RecoveryForensicArtifactKindV1, RecoveryForensicRecordKindV1,
        RecoveryReceiptAdoptionRequestV1,
    },
    validate_field,
};

pub(super) struct ForensicOutcome {
    pub(super) quarantined_orchestrator: String,
}

#[path = "forensic/derive.rs"]
mod derive;
pub(super) use derive::derive_next;

struct Artifacts<'a> {
    trusted: Artifact<'a>,
    frozen: Artifact<'a>,
}

struct Artifact<'a> {
    bytes: &'a [u8],
    binding: &'a ArtifactBinding,
}

impl Artifacts<'_> {
    fn record(&self, reference: &ForensicRecordRefV1) -> Result<Record, CoordError> {
        match reference.artifact_kind {
            RecoveryForensicArtifactKindV1::TrustedPrefix => self.trusted.record(reference),
            RecoveryForensicArtifactKindV1::FrozenLiveSource => self.frozen.record(reference),
        }
    }
}

pub(super) fn verify(
    request: &RecoveryReceiptAdoptionRequestV1,
    manifest: &RecoveryManifestBody,
    claims: &BTreeMap<String, ClaimSummary>,
    sources: ForensicSources<'_>,
) -> Result<ForensicOutcome, CoordError> {
    let artifacts = Artifacts {
        trusted: Artifact::new(sources.trusted_prefix, &manifest.artifacts.trusted_prefix)?,
        frozen: Artifact::new(
            sources.frozen_live_source,
            &manifest.artifacts.frozen_live_source,
        )?,
    };
    reject_duplicate_or_overlapping_refs(request)?;
    verify_causal_order(request)?;
    verify_parent_receipt(request, claims, &artifacts)?;
    for requested in &request.subject.claims {
        verify_claim(requested, claims, &artifacts, manifest.incident_at_unix_ms)?;
    }
    verify_group(request, &artifacts, manifest.incident_at_unix_ms)
}

fn verify_causal_order(request: &RecoveryReceiptAdoptionRequestV1) -> Result<(), CoordError> {
    let group = &request.subject.group_receipt_observation;
    for claim in &request.subject.claims {
        let trusted = &claim.trusted_claim_record;
        let handoff = &claim.handoff_observation;
        if trusted.record_index >= handoff.record_index
            || trusted.byte_end > handoff.byte_start
            || handoff.record_index >= group.record_index
            || handoff.byte_end > group.byte_start
        {
            return Err(mismatch(
                "forensic claim, handoff, and group receipt are not in causal order",
            ));
        }
    }
    Ok(())
}

impl<'a> Artifact<'a> {
    fn new(bytes: &'a [u8], binding: &'a ArtifactBinding) -> Result<Self, CoordError> {
        if bytes.len() as u64 != binding.byte_length
            || binding.sha256.as_str() != sha256(bytes)
            || !binding.ends_with_lf
            || bytes.last() != Some(&b'\n')
        {
            return Err(mismatch(
                "forensic artifact differs from recovery manifest binding",
            ));
        }
        if let Some(expected) = binding.record_count
            && bytes.iter().filter(|byte| **byte == b'\n').count() as u64 != expected
        {
            return Err(mismatch(
                "forensic artifact record count differs from its binding",
            ));
        }
        Ok(Self { bytes, binding })
    }

    fn record(&self, reference: &ForensicRecordRefV1) -> Result<Record, CoordError> {
        if reference.artifact_sha256 != self.binding.sha256.as_str() {
            return Err(mismatch("forensic reference names another artifact digest"));
        }
        let start = usize::try_from(reference.byte_start)
            .map_err(|_| mismatch("forensic byte start does not fit this host"))?;
        let end = usize::try_from(reference.byte_end)
            .map_err(|_| mismatch("forensic byte end does not fit this host"))?;
        let line = self
            .bytes
            .get(start..end)
            .ok_or_else(|| mismatch("forensic byte range is outside its artifact"))?;
        if line.last() != Some(&b'\n')
            || (start > 0 && self.bytes[start - 1] != b'\n')
            || line[..line.len() - 1].contains(&b'\n')
            || self.bytes[..start]
                .iter()
                .filter(|byte| **byte == b'\n')
                .count() as u64
                + 1
                != reference.record_index
            || sha256(line) != reference.record_sha256
        {
            return Err(mismatch(
                "forensic record range, index, line boundary, or digest differs",
            ));
        }
        let record =
            bullet_wire::decode_canonical::<Record>(&line[..line.len() - 1]).map_err(|error| {
                mismatch(format!(
                    "forensic record is not strict canonical JSON: {error}"
                ))
            })?;
        if record.schema_version() != LEGACY_SCHEMA_VERSION
            || kind(&record) != Some(reference.expected_record_kind)
        {
            return Err(mismatch(
                "forensic record schema or kind differs from its reference",
            ));
        }
        Ok(record)
    }
}

fn verify_parent_receipt(
    request: &RecoveryReceiptAdoptionRequestV1,
    claims: &BTreeMap<String, ClaimSummary>,
    artifacts: &Artifacts<'_>,
) -> Result<(), CoordError> {
    let record = artifacts.record(&request.subject.git_expectation.parent_receipt_observation)?;
    let Record::CommitReceipt {
        at_unix_ms,
        claim_id,
        orchestrator,
        commit_oid,
        committed_paths,
        ..
    } = record
    else {
        return Err(mismatch("trusted parent receipt is not a commit receipt"));
    };
    let parent = untag(&request.subject.git_expectation.parent_oid, "sha1:")?;
    let claim = claims
        .get(&claim_id)
        .ok_or_else(|| mismatch("trusted parent receipt references a missing claim"))?;
    validate_field("trusted parent orchestrator", &orchestrator)
        .map_err(|error| mismatch(error.to_string()))?;
    if commit_oid != parent
        || claim.repo != request.subject.repo
        || claim.state != ClaimState::HandedOff
        || claim.commit_oid.as_deref() != Some(commit_oid.as_str())
        || claim.commit_orchestrator.as_deref() != Some(orchestrator.as_str())
        || claim.commit_recorded_at_unix_ms != Some(at_unix_ms)
        || claim.changed_paths != committed_paths
    {
        return Err(mismatch(
            "trusted parent receipt is not the final trusted repository receipt",
        ));
    }
    Ok(())
}

fn verify_claim(
    requested: &RecoveryAdoptionClaimV1,
    claims: &BTreeMap<String, ClaimSummary>,
    artifacts: &Artifacts<'_>,
    incident_at: u64,
) -> Result<(), CoordError> {
    let trusted = artifacts.record(&requested.trusted_claim_record)?;
    let Record::Claim {
        at_unix_ms,
        claim_id,
        agent,
        lane,
        repo,
        paths,
        ..
    } = trusted
    else {
        return Err(mismatch("trusted claim reference is not a claim record"));
    };
    let claim = claims
        .get(&requested.claim_id)
        .ok_or_else(|| mismatch("trusted claim is absent from current projection"))?;
    if claim_id != requested.claim_id
        || at_unix_ms > incident_at
        || agent != claim.agent
        || lane != claim.lane
        || repo != claim.repo
        || paths != claim.paths
    {
        return Err(mismatch(
            "trusted claim record differs from frozen claim authority",
        ));
    }

    let handoff = artifacts.record(&requested.handoff_observation)?;
    let Record::Handoff {
        at_unix_ms,
        claim_id,
        agent,
        proof_command,
        proof_exit_code,
        changed_paths,
        commit_oid,
        ..
    } = handoff
    else {
        return Err(mismatch("handoff observation is not a handoff record"));
    };
    validate_field("quarantined handoff proof command", &proof_command)
        .map_err(|error| mismatch(error.to_string()))?;
    if at_unix_ms <= incident_at
        || claim_id != requested.claim_id
        || agent != claim.agent
        || proof_exit_code != 0
        || changed_paths != requested.committed_paths
        || commit_oid.is_some()
    {
        return Err(mismatch(
            "quarantined handoff does not bind the expected clean claim partition",
        ));
    }
    Ok(())
}

fn verify_group(
    request: &RecoveryReceiptAdoptionRequestV1,
    artifacts: &Artifacts<'_>,
    incident_at: u64,
) -> Result<ForensicOutcome, CoordError> {
    let record = artifacts.record(&request.subject.group_receipt_observation)?;
    let Record::CommitReceiptGroup {
        at_unix_ms,
        orchestrator,
        commit_oid,
        receipts,
        ..
    } = record
    else {
        return Err(mismatch(
            "group receipt observation is not a grouped commit receipt",
        ));
    };
    let expected = request
        .subject
        .claims
        .iter()
        .map(|claim| GroupReceipt {
            claim_id: claim.claim_id.clone(),
            committed_paths: claim.committed_paths.clone(),
        })
        .collect::<Vec<_>>();
    validate_field("quarantined group orchestrator", &orchestrator)
        .map_err(|error| mismatch(error.to_string()))?;
    if at_unix_ms <= incident_at
        || commit_oid != untag(&request.subject.git_expectation.commit_oid, "sha1:")?
        || receipts != expected
    {
        return Err(mismatch(
            "quarantined grouped receipt differs from the expected commit partition",
        ));
    }
    Ok(ForensicOutcome {
        quarantined_orchestrator: orchestrator,
    })
}

fn reject_duplicate_or_overlapping_refs(
    request: &RecoveryReceiptAdoptionRequestV1,
) -> Result<(), CoordError> {
    let mut refs = vec![
        &request.subject.git_expectation.parent_receipt_observation,
        &request.subject.group_receipt_observation,
    ];
    for claim in &request.subject.claims {
        refs.push(&claim.trusted_claim_record);
        refs.push(&claim.handoff_observation);
    }
    let mut ranges = BTreeSet::new();
    for reference in refs {
        let key = (
            reference.artifact_kind,
            reference.byte_start,
            reference.byte_end,
        );
        if !ranges.insert(key) {
            return Err(mismatch("forensic request repeats a record range"));
        }
    }
    let ranges = ranges.into_iter().collect::<Vec<_>>();
    for pair in ranges.windows(2) {
        if pair[0].0 == pair[1].0 && pair[0].2 > pair[1].1 {
            return Err(mismatch("forensic request contains overlapping ranges"));
        }
    }
    Ok(())
}

fn kind(record: &Record) -> Option<RecoveryForensicRecordKindV1> {
    Some(match record {
        Record::Claim { .. } => RecoveryForensicRecordKindV1::Claim,
        Record::Handoff { .. } => RecoveryForensicRecordKindV1::Handoff,
        Record::CommitReceipt { .. } => RecoveryForensicRecordKindV1::CommitReceipt,
        Record::CommitReceiptGroup { .. } => RecoveryForensicRecordKindV1::CommitReceiptGroup,
        _ => return None,
    })
}

fn untag<'a>(value: &'a str, prefix: &str) -> Result<&'a str, CoordError> {
    value
        .strip_prefix(prefix)
        .ok_or_else(|| mismatch("forensic Git OID is not tagged"))
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
#[path = "forensic/tests.rs"]
mod tests;
