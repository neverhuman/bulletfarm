use std::collections::BTreeMap;

use super::corrupt;
use crate::coord::{
    CoordError,
    model::{ClaimState, ClaimSummary, FrozenClaimSubject, RecoveryBaselineBody},
};

pub(super) fn apply_recovery_baseline(
    body: &RecoveryBaselineBody,
    claims: &mut BTreeMap<String, ClaimSummary>,
) -> Result<(), CoordError> {
    if !is_tagged_blake3(&body.manifest_blake3)
        || !is_tagged_blake3(&body.trusted_state_blake3)
        || body.incident_at_unix_ms == 0
        || body.recovered_at_unix_ms <= body.incident_at_unix_ms
    {
        return Err(corrupt("recovery baseline policy fields are invalid"));
    }
    let mut trusted_state = claims.clone();
    for claim in trusted_state.values_mut() {
        claim.refresh_state(body.incident_at_unix_ms);
    }
    let trusted_state_digest =
        bullet_wire::hash_canonical("bullet-family.coord.trusted-state.v2", &trusted_state)
            .map_err(|error| corrupt(error.to_string()))?;
    if body.trusted_state_blake3 != format!("blake3:{}", trusted_state_digest.to_hex()) {
        return Err(corrupt(
            "recovery baseline trusted-state digest differs from trusted-prefix replay",
        ));
    }
    let mut expected = trusted_state
        .into_values()
        .filter(|claim| claim.state == ClaimState::Active)
        .map(|claim| {
            let digest = bullet_wire::hash_canonical("bullet-family.coord.frozen-claim.v2", &claim)
                .map_err(|error| corrupt(error.to_string()))?;
            Ok(FrozenClaimSubject {
                claim_id: claim.claim_id,
                claim_blake3: format!("blake3:{}", digest.to_hex()),
            })
        })
        .collect::<Result<Vec<_>, CoordError>>()?;
    expected.sort_by(|left, right| left.claim_id.cmp(&right.claim_id));
    if body.frozen_claims != expected {
        return Err(corrupt(
            "recovery baseline frozen claims differ from trusted-prefix replay",
        ));
    }
    for frozen in &body.frozen_claims {
        let claim = claims
            .get_mut(&frozen.claim_id)
            .ok_or_else(|| corrupt("recovery baseline freezes an unknown claim"))?;
        claim.state = ClaimState::FrozenRecovery;
    }
    Ok(())
}

pub(super) fn is_tagged_blake3(value: &str) -> bool {
    value.strip_prefix("blake3:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

pub(super) fn is_generation_id(value: &str) -> bool {
    value.strip_prefix("gen_").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}
