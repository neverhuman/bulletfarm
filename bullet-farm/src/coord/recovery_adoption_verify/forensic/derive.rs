use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use super::{Artifact, ForensicSources, kind, mismatch};
use crate::coord::{
    ClaimState, CoordError,
    generation::manifest::RecoveryManifestBody,
    model::{
        ClaimSummary, ForensicRecordRefV1, GroupReceipt, LEGACY_SCHEMA_VERSION, Record,
        RecoveryAdoptionClaimV1, RecoveryForensicArtifactKindV1,
    },
    validate_field,
};

pub(crate) struct ForensicCandidate {
    pub(crate) repo: String,
    pub(crate) commit_oid: String,
    pub(crate) quarantined_orchestrator: String,
    pub(crate) claims: Vec<RecoveryAdoptionClaimV1>,
    pub(crate) group_receipt_observation: ForensicRecordRefV1,
    pub(crate) parent_receipts: BTreeMap<String, ForensicRecordRefV1>,
}

struct ReferencedRecord {
    record: Record,
    reference: ForensicRecordRefV1,
}

type ObservedHandoffs = BTreeMap<String, Vec<(ForensicRecordRefV1, Vec<String>)>>;

pub(crate) fn derive_next(
    manifest: &RecoveryManifestBody,
    claims: &BTreeMap<String, ClaimSummary>,
    sources: ForensicSources<'_>,
) -> Result<ForensicCandidate, CoordError> {
    let trusted_artifact =
        Artifact::new(sources.trusted_prefix, &manifest.artifacts.trusted_prefix)?;
    let frozen_artifact = Artifact::new(
        sources.frozen_live_source,
        &manifest.artifacts.frozen_live_source,
    )?;
    let trusted = scan(
        sources.trusted_prefix,
        &trusted_artifact,
        RecoveryForensicArtifactKindV1::TrustedPrefix,
    )?;
    let frozen = scan(
        sources.frozen_live_source,
        &frozen_artifact,
        RecoveryForensicArtifactKindV1::FrozenLiveSource,
    )?;
    let trusted_claims = trusted_claim_refs(&trusted, claims, manifest.incident_at_unix_ms)?;
    let handoffs = frozen_handoffs(&frozen, claims, manifest.incident_at_unix_ms)?;
    let frozen_digests = manifest
        .frozen_claims
        .iter()
        .map(|claim| (claim.claim_id.as_str(), claim.claim_blake3.as_str()))
        .collect::<BTreeMap<_, _>>();

    let mut candidates = Vec::new();
    for item in &frozen {
        let Record::CommitReceiptGroup {
            at_unix_ms,
            orchestrator,
            commit_oid,
            receipts,
            ..
        } = &item.record
        else {
            continue;
        };
        if *at_unix_ms <= manifest.incident_at_unix_ms || receipts.len() < 2 {
            continue;
        }
        validate_field("quarantined group orchestrator", orchestrator)
            .map_err(|error| mismatch(error.to_string()))?;
        let mut sorted_receipts = receipts.clone();
        sorted_receipts.sort_by(|left, right| left.claim_id.cmp(&right.claim_id));
        if &sorted_receipts != receipts
            || receipts
                .windows(2)
                .any(|pair| pair[0].claim_id == pair[1].claim_id)
        {
            return Err(mismatch(
                "quarantined group receipts are not sorted and unique",
            ));
        }
        let Some(repo) = candidate_repo(receipts, claims) else {
            continue;
        };
        let mut requested = Vec::with_capacity(receipts.len());
        let mut complete = true;
        for receipt in receipts {
            let Some(claim) = claims.get(&receipt.claim_id) else {
                complete = false;
                break;
            };
            let Some(trusted_claim_record) = trusted_claims.get(&receipt.claim_id) else {
                complete = false;
                break;
            };
            let Some((handoff, changed_paths)) =
                handoffs.get(&receipt.claim_id).and_then(|observed| {
                    observed.iter().rev().find(|(reference, _)| {
                        reference.record_index < item.reference.record_index
                            && reference.byte_end <= item.reference.byte_start
                    })
                })
            else {
                complete = false;
                break;
            };
            let Some(frozen_claim_blake3) = frozen_digests.get(receipt.claim_id.as_str()) else {
                complete = false;
                break;
            };
            if claim.state != ClaimState::FrozenRecovery
                || claim.recovery_adoption.is_some()
                || changed_paths != &receipt.committed_paths
            {
                complete = false;
                break;
            }
            requested.push(RecoveryAdoptionClaimV1 {
                claim_id: receipt.claim_id.clone(),
                frozen_claim_blake3: (*frozen_claim_blake3).to_owned(),
                trusted_claim_record: trusted_claim_record.clone(),
                committed_paths: receipt.committed_paths.clone(),
                handoff_observation: handoff.clone(),
            });
        }
        if complete {
            candidates.push(ForensicCandidate {
                parent_receipts: trusted_parent_receipts(&trusted, claims, &repo)?,
                repo,
                commit_oid: commit_oid.clone(),
                quarantined_orchestrator: orchestrator.clone(),
                claims: requested,
                group_receipt_observation: item.reference.clone(),
            });
        }
    }
    candidates.sort_by(|left, right| {
        (
            &left.repo,
            &left.commit_oid,
            left.group_receipt_observation.record_index,
        )
            .cmp(&(
                &right.repo,
                &right.commit_oid,
                right.group_receipt_observation.record_index,
            ))
    });
    candidates.into_iter().next().ok_or_else(|| {
        CoordError::new(
            "RECOVERY_PRODUCTION_EMPTY",
            "no exact unadopted grouped recovery receipt is derivable",
        )
    })
}

fn candidate_repo(
    receipts: &[GroupReceipt],
    claims: &BTreeMap<String, ClaimSummary>,
) -> Option<String> {
    let mut repo = None;
    for receipt in receipts {
        let claim = claims.get(&receipt.claim_id)?;
        if repo.as_ref().is_some_and(|value| value != &claim.repo) {
            return None;
        }
        repo = Some(claim.repo.clone());
    }
    repo
}

fn frozen_handoffs(
    records: &[ReferencedRecord],
    claims: &BTreeMap<String, ClaimSummary>,
    incident_at: u64,
) -> Result<ObservedHandoffs, CoordError> {
    let mut handoffs = BTreeMap::<String, Vec<_>>::new();
    for item in records {
        match &item.record {
            Record::Handoff {
                at_unix_ms,
                claim_id,
                agent,
                proof_command,
                proof_exit_code,
                changed_paths,
                commit_oid,
                ..
            } if *at_unix_ms > incident_at && *proof_exit_code == 0 && commit_oid.is_none() => {
                let Some(claim) = claims.get(claim_id) else {
                    continue;
                };
                validate_field("quarantined handoff proof command", proof_command)
                    .map_err(|error| mismatch(error.to_string()))?;
                if agent != &claim.agent {
                    continue;
                }
                handoffs
                    .entry(claim_id.clone())
                    .or_default()
                    .push((item.reference.clone(), changed_paths.clone()));
            }
            _ => {}
        }
    }
    Ok(handoffs)
}

fn trusted_claim_refs(
    records: &[ReferencedRecord],
    claims: &BTreeMap<String, ClaimSummary>,
    incident_at: u64,
) -> Result<BTreeMap<String, ForensicRecordRefV1>, CoordError> {
    let mut result = BTreeMap::new();
    for item in records {
        let Record::Claim {
            at_unix_ms,
            claim_id,
            agent,
            lane,
            repo,
            paths,
            ..
        } = &item.record
        else {
            continue;
        };
        let Some(claim) = claims.get(claim_id) else {
            continue;
        };
        if *at_unix_ms <= incident_at
            && agent == &claim.agent
            && lane == &claim.lane
            && repo == &claim.repo
            && paths == &claim.paths
            && result
                .insert(claim_id.clone(), item.reference.clone())
                .is_some()
        {
            return Err(mismatch("trusted prefix repeats a frozen claim record"));
        }
    }
    Ok(result)
}

fn trusted_parent_receipts(
    records: &[ReferencedRecord],
    claims: &BTreeMap<String, ClaimSummary>,
    repo: &str,
) -> Result<BTreeMap<String, ForensicRecordRefV1>, CoordError> {
    let mut result = BTreeMap::new();
    for item in records {
        let Record::CommitReceipt {
            at_unix_ms,
            claim_id,
            orchestrator,
            commit_oid,
            committed_paths,
            ..
        } = &item.record
        else {
            continue;
        };
        let Some(claim) = claims.get(claim_id) else {
            continue;
        };
        if claim.repo == repo
            && claim.state == ClaimState::HandedOff
            && claim.commit_oid.as_deref() == Some(commit_oid)
            && claim.commit_orchestrator.as_deref() == Some(orchestrator)
            && claim.commit_recorded_at_unix_ms == Some(*at_unix_ms)
            && claim.changed_paths == *committed_paths
            && result
                .insert(commit_oid.clone(), item.reference.clone())
                .is_some()
        {
            return Err(mismatch(
                "trusted prefix repeats a parent commit receipt OID",
            ));
        }
    }
    Ok(result)
}

fn scan(
    bytes: &[u8],
    artifact: &Artifact<'_>,
    artifact_kind: RecoveryForensicArtifactKindV1,
) -> Result<Vec<ReferencedRecord>, CoordError> {
    let mut result = Vec::new();
    let mut start = 0usize;
    for (index, line) in bytes.split_inclusive(|byte| *byte == b'\n').enumerate() {
        let end = start + line.len();
        let record =
            bullet_wire::decode_canonical::<Record>(&line[..line.len() - 1]).map_err(|error| {
                mismatch(format!(
                    "forensic record is not strict canonical JSON: {error}"
                ))
            })?;
        let Some(expected_record_kind) = kind(&record) else {
            start = end;
            continue;
        };
        if record.schema_version() != LEGACY_SCHEMA_VERSION {
            return Err(mismatch("forensic producer record schema is unsupported"));
        }
        let reference = ForensicRecordRefV1 {
            artifact_kind,
            artifact_sha256: artifact.binding.sha256.as_str().to_owned(),
            record_index: index as u64 + 1,
            byte_start: start as u64,
            byte_end: end as u64,
            record_sha256: format!("sha256:{:x}", Sha256::digest(line)),
            expected_record_kind,
        };
        artifact.record(&reference)?;
        result.push(ReferencedRecord { record, reference });
        start = end;
    }
    Ok(result)
}
