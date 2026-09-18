use std::path::Path;

use super::*;

#[path = "linux/source.rs"]
mod source;
use source::{ParentRole, StableSource, read_stable};

const MAX_ARTIFACT_BYTES: u64 = 64 * 1024 * 1024;
const INVENTORY_DOMAIN: &str = "bullet.coord.post-prefix-inventory.v2";

pub(super) fn inspect(
    family_root: &Path,
    command: &RecoveryInspectionCommand,
) -> Result<RecoveryInspectionV1, CoordError> {
    for (path, label) in [
        (&command.interrupted_capture, "interrupted capture"),
        (&command.tainted_generation, "tainted generation"),
        (&command.frozen_live_source, "frozen live source"),
    ] {
        require_normalized_absolute(path, label)?;
    }
    let expected_live = family_root.join(".bullet-family/coord/events.jsonl");
    if command.frozen_live_source != expected_live {
        return Err(invalid(
            "frozen live source must be the selected family's exact legacy coordinator source",
        ));
    }

    let interrupted = read_stable(
        &command.interrupted_capture,
        "interrupted capture",
        ParentRole::Sealed,
    )?;
    let tainted = read_stable(
        &command.tainted_generation,
        "tainted generation",
        ParentRole::Sealed,
    )?;
    let frozen = read_stable(
        &command.frozen_live_source,
        "frozen live source",
        ParentRole::FrozenLegacy,
    )?;
    if interrupted.bytes.len() >= tainted.bytes.len()
        || tainted.bytes.len() >= frozen.bytes.len()
        || !tainted.bytes.starts_with(&interrupted.bytes)
    {
        return Err(invalid(
            "recovery source lengths or interrupted-to-tainted lineage are invalid",
        ));
    }
    let common = common_prefix3(&interrupted.bytes, &tainted.bytes, &frozen.bytes);
    let trusted_end = interrupted.bytes[..common]
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map(|index| index + 1)
        .ok_or_else(|| invalid("recovery sources have no common LF-committed record"))?;
    let trusted = &interrupted.bytes[..trusted_end];
    let records = super::super::store::legacy::read_record_bytes(trusted)?;
    let incident_at_unix_ms = records
        .iter()
        .map(super::super::store::subject::record_time)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .max()
        .ok_or_else(|| invalid("trusted prefix contains no records"))?;
    let summaries = super::super::state::summaries(&records, incident_at_unix_ms)?;
    let projection =
        super::super::generation::recovery::projection::inventory(&records, &summaries)?;
    let trusted_digest =
        bullet_wire::hash_canonical("bullet-family.coord.trusted-state.v2", &summaries)
            .map_err(wire)?;
    let frozen_claims = summaries
        .values()
        .map(|summary| {
            let digest =
                bullet_wire::hash_canonical("bullet-family.coord.frozen-claim.v2", summary)
                    .map_err(wire)?;
            Ok(FrozenClaimSubject {
                claim_id: summary.claim_id.clone(),
                claim_blake3: format!("blake3:{}", digest.to_hex()),
            })
        })
        .collect::<Result<Vec<_>, CoordError>>()?;

    let artifacts = RecoveryInspectionArtifactsV1 {
        trusted_prefix: binding(
            super::super::generation::manifest::TRUSTED_PREFIX_PATH,
            trusted,
        )?,
        interrupted_capture: source_binding(
            super::super::generation::manifest::INTERRUPTED_CAPTURE_PATH,
            &interrupted,
        )?,
        tainted_generation: source_binding(
            super::super::generation::manifest::TAINTED_GENERATION_PATH,
            &tainted,
        )?,
        frozen_live_source: source_binding(
            super::super::generation::manifest::FROZEN_LIVE_SOURCE_PATH,
            &frozen,
        )?,
    };
    let manifest_artifacts = artifacts.manifest_artifacts();
    let start = trusted_end as u64;
    let discarded_range = ByteRange {
        start_inclusive: start,
        end_exclusive: frozen.bytes.len() as u64,
    };
    let ambiguous_tail_range = ByteRange {
        start_inclusive: start,
        end_exclusive: interrupted.bytes.len() as u64,
    };
    let ambiguous_tail_sha256 = Sha256Digest::for_bytes(&interrupted.bytes[trusted_end..]);
    let post_prefix_inventory_blake3 = post_prefix_inventory(PostPrefixInput {
        interrupted: &interrupted.bytes,
        tainted: &tainted.bytes,
        frozen: &frozen.bytes,
        artifacts: &manifest_artifacts,
        discarded_range,
        ambiguous_tail_range,
        ambiguous_tail_sha256: &ambiguous_tail_sha256,
        start: trusted_end,
    })?;
    RecoveryInspectionV1::from_subject(RecoveryInspectionSubjectV1 {
        parent_generation: "legacy-v1".to_owned(),
        incident_at_unix_ms,
        trusted_record_count: records.len() as u64,
        trusted_projection_inventory: projection,
        discarded_range,
        ambiguous_tail_range,
        ambiguous_tail_sha256,
        artifacts,
        trusted_state_blake3: format!("blake3:{}", trusted_digest.to_hex()),
        frozen_claims,
        post_prefix_inventory_blake3,
    })
}

fn binding(path: &str, bytes: &[u8]) -> Result<ArtifactBinding, CoordError> {
    ArtifactBinding::new(
        RelativeArtifactPath::parse(path)?,
        bytes.len() as u64,
        Some(bytes.iter().filter(|byte| **byte == b'\n').count() as u64),
        bytes.last() == Some(&b'\n'),
        Sha256Digest::for_bytes(bytes),
    )
}

fn source_binding(
    path: &str,
    source: &StableSource,
) -> Result<RecoverySourceInspectionV1, CoordError> {
    Ok(RecoverySourceInspectionV1 {
        binding: binding(path, &source.bytes)?,
        identity: source.identity.clone(),
    })
}

fn common_prefix3(left: &[u8], middle: &[u8], right: &[u8]) -> usize {
    left.iter()
        .zip(middle)
        .zip(right)
        .take_while(|((left, middle), right)| left == middle && middle == right)
        .count()
}

struct PostPrefixInput<'a> {
    interrupted: &'a [u8],
    tainted: &'a [u8],
    frozen: &'a [u8],
    artifacts: &'a RecoveryArtifacts,
    discarded_range: ByteRange,
    ambiguous_tail_range: ByteRange,
    ambiguous_tail_sha256: &'a Sha256Digest,
    start: usize,
}

fn post_prefix_inventory(input: PostPrefixInput<'_>) -> Result<String, CoordError> {
    let PostPrefixInput {
        interrupted,
        tainted,
        frozen,
        artifacts,
        discarded_range,
        ambiguous_tail_range,
        ambiguous_tail_sha256,
        start,
    } = input;
    let subject = serde_json::json!({
        "kind": "coord_post_prefix_inventory_v2",
        "schema_version": 2,
        "artifacts": artifacts,
        "trusted_range": { "start_inclusive": 0, "end_exclusive": start },
        "discarded_range": discarded_range,
        "ambiguous_tail_range": ambiguous_tail_range,
        "ambiguous_tail_sha256": ambiguous_tail_sha256,
        "suffixes": {
            "interrupted": observe(&interrupted[start..]),
            "tainted": observe(&tainted[start..]),
            "frozen_live": observe(&frozen[start..]),
        },
        "pairwise_suffix_common_prefix_bytes": {
            "interrupted_tainted": common_suffix(interrupted, tainted, start),
            "interrupted_frozen": common_suffix(interrupted, frozen, start),
            "tainted_frozen": common_suffix(tainted, frozen, start),
        },
        "lineage": "TRUSTED_PREFIX_THEN_AMBIGUOUS_THEN_QUARANTINED",
        "post_prefix_default": "QUARANTINED",
        "implicit_adoptions": 0,
    });
    let canonical = bullet_wire::canonical_json(&subject).map_err(wire)?;
    let digest = bullet_wire::hash_framed_bytes(INVENTORY_DOMAIN, &canonical).map_err(wire)?;
    Ok(format!("blake3:{}", digest.to_hex()))
}

fn observe(bytes: &[u8]) -> serde_json::Value {
    serde_json::json!({
        "byte_length": bytes.len(),
        "sha256": Sha256Digest::for_bytes(bytes),
        "lf_count": bytes.iter().filter(|byte| **byte == b'\n').count(),
        "ends_with_lf": bytes.last() == Some(&b'\n'),
    })
}

fn common_suffix(left: &[u8], right: &[u8], start: usize) -> usize {
    left[start..]
        .iter()
        .zip(&right[start..])
        .take_while(|(left, right)| left == right)
        .count()
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_MANIFEST_PRODUCTION", reason)
}

fn changed(reason: impl Into<String>) -> CoordError {
    CoordError::new("RECOVERY_INSPECTION_CHANGED", reason)
}

fn wire(error: impl std::fmt::Display) -> CoordError {
    invalid(format!("cannot derive recovery inspection: {error}"))
}
