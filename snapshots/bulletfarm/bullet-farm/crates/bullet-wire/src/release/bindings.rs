use std::collections::BTreeSet;

use crate::{
    WireError, hash_canonical,
    v1alpha1::{
        GateReceiptV1, ReleaseGateSpecV1, ReleaseGateVerificationRequestV1, ReleaseProfileGraphV1,
    },
};

use super::{
    RELEASE_GATE_SPEC_DIGEST_DOMAIN, RELEASE_PROFILE_GRAPH_DIGEST_DOMAIN,
    RELEASE_VERIFICATION_REQUEST_DIGEST_DOMAIN,
    fields::{evidence_kind, invalid},
    validate::ReleaseWireRecord,
};

pub fn validate_release_bindings(
    profile_graph: &ReleaseProfileGraphV1,
    gate_spec: &ReleaseGateSpecV1,
    request: &ReleaseGateVerificationRequestV1,
    receipt: &GateReceiptV1,
) -> Result<(), WireError> {
    profile_graph.validate_release()?;
    gate_spec.validate_release()?;
    request.validate_release()?;
    receipt.validate_release()?;

    for profile_id in &gate_spec.profile_ids {
        let profile = profile_graph
            .profiles
            .iter()
            .find(|profile| &profile.profile_id == profile_id)
            .ok_or_else(|| invalid("gate spec names a profile absent from its graph"))?;
        if profile.gate_ids.binary_search(&gate_spec.gate_id).is_err() {
            return Err(invalid("profile graph does not assign the specified gate"));
        }
    }

    let profile_graph_digest = tagged_hash(RELEASE_PROFILE_GRAPH_DIGEST_DOMAIN, profile_graph)?;
    let gate_spec_digest = tagged_hash(RELEASE_GATE_SPEC_DIGEST_DOMAIN, gate_spec)?;
    let request_digest = tagged_hash(RELEASE_VERIFICATION_REQUEST_DIGEST_DOMAIN, request)?;

    if request.gate_id != gate_spec.gate_id
        || request.gate_version != gate_spec.gate_version
        || request.receipt_kind != gate_spec.receipt_kind
        || request.profile_ids != gate_spec.profile_ids
        || request.gate_policy_digest != gate_spec.gate_policy_digest
        || request.profile_graph_digest != profile_graph_digest
        || request.gate_spec_digest != gate_spec_digest
    {
        return Err(invalid(
            "verification request does not bind its graph and gate spec",
        ));
    }

    let request_evidence = request
        .evidence_subjects
        .iter()
        .map(|subject| evidence_kind(subject.subject_kind))
        .collect::<BTreeSet<_>>();
    if !gate_spec
        .required_evidence_kinds
        .iter()
        .all(|kind| request_evidence.contains(evidence_kind(*kind)))
    {
        return Err(invalid("verification request omits gate-required evidence"));
    }

    if receipt.gate_id != request.gate_id
        || receipt.gate_version != request.gate_version
        || receipt.receipt_kind != request.receipt_kind
        || receipt.profile_ids != request.profile_ids
        || receipt.evidence_nonce != request.evidence_nonce
        || receipt.request_digest != request_digest
        || receipt.gate_spec_digest != request.gate_spec_digest
        || receipt.profile_graph_digest != request.profile_graph_digest
        || receipt.gate_policy_digest != request.gate_policy_digest
        || receipt.family_subject != request.family_subject
        || receipt.evidence_subjects != request.evidence_subjects
    {
        return Err(invalid(
            "gate receipt does not bind its verification request",
        ));
    }
    if receipt.started_at_unix_ms < request.requested_at_unix_ms
        || receipt.completed_at_unix_ms >= request.expires_at_unix_ms
    {
        return Err(invalid(
            "gate receipt execution lies outside request validity",
        ));
    }
    Ok(())
}

fn tagged_hash<T: serde::Serialize>(domain: &str, value: &T) -> Result<String, WireError> {
    Ok(format!(
        "blake3:{}",
        hash_canonical(domain, value)?.to_hex()
    ))
}
