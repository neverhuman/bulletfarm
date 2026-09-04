use std::collections::{BTreeMap, BTreeSet};

use bullet_wire::{
    v1alpha1::{GateReceiptV1, ReleaseEvidenceKindV1},
    validate_release_bundle_manifest_v2_binding,
};

use super::{kinds::validate_receipt_kind, *};

pub(super) fn validate_registry(
    manifest: &ReleaseRegistryManifestV1,
    objects: &[LoadedObject<'_>],
    selected_profile: &str,
    requested_profiles: &[RequestedProfile],
) -> Result<usize, Reject> {
    let graph_object = object(
        objects,
        ReleaseRegistryObjectKindV1::ProfileGraph,
        &manifest.profile_graph_digest,
    )?;
    let graph = graph_object.decoded.graph()?;
    validate_requested_graph(graph, selected_profile, requested_profiles)?;
    let policy_object = object(
        objects,
        ReleaseRegistryObjectKindV1::SignerPolicy,
        &manifest.signer_policy_digest,
    )?;
    let policy = policy_object.decoded.signer_policy()?;
    validate_policy_binding(manifest, graph, policy)?;

    let requested = requested_profiles
        .iter()
        .map(|profile| profile.id)
        .collect::<BTreeSet<_>>();
    let mut referenced = BTreeSet::from([
        graph_object.subject.object_path.as_str(),
        policy_object.subject.object_path.as_str(),
    ]);
    let mut gate_profiles = BTreeSet::new();
    let mut selected_gate_profiles = BTreeMap::new();
    let mut selected_bindings = 0;
    for entry in &manifest.entries {
        for profile in &entry.profile_ids {
            if !gate_profiles.insert((entry.gate_id.as_str(), profile.as_str())) {
                return Err(reject("registry repeats a gate/profile binding"));
            }
        }

        let receipt_object = exact_object(
            objects,
            ReleaseRegistryObjectKindV1::GateReceipt,
            &entry.receipt_digest,
            &entry.receipt_path,
        )?;
        let receipt = receipt_object.decoded.receipt()?;
        exact_object(
            objects,
            ReleaseRegistryObjectKindV1::GateReceiptSignature,
            &entry.receipt_signature_digest,
            &entry.receipt_signature_path,
        )?;
        let time_object = exact_object(
            objects,
            ReleaseRegistryObjectKindV1::TrustedTimeObservation,
            &entry.trusted_time_digest,
            &entry.trusted_time_path,
        )?;
        let trusted_time = time_object.decoded.trusted_time()?;
        exact_object(
            objects,
            ReleaseRegistryObjectKindV1::TrustedTimeSignature,
            &entry.trusted_time_signature_digest,
            &entry.trusted_time_signature_path,
        )?;
        for path in [
            entry.receipt_path.as_str(),
            entry.receipt_signature_path.as_str(),
            entry.trusted_time_path.as_str(),
            entry.trusted_time_signature_path.as_str(),
        ] {
            referenced.insert(path);
        }

        let spec_object = object(
            objects,
            ReleaseRegistryObjectKindV1::GateSpec,
            &receipt.gate_spec_digest,
        )?;
        let request_object = object(
            objects,
            ReleaseRegistryObjectKindV1::VerificationRequest,
            &receipt.request_digest,
        )?;
        let request = request_object.decoded.request()?;
        referenced.insert(spec_object.subject.object_path.as_str());
        referenced.insert(request_object.subject.object_path.as_str());
        validate_release_bindings(graph, spec_object.decoded.gate_spec()?, request, receipt)
            .map_err(|error| reject(format!("release binding is invalid: {error}")))?;
        validate_receipt_kind(receipt)?;
        validate_artifact_binding(receipt, objects, &mut referenced)?;
        validate_entry(
            entry,
            request,
            receipt,
            trusted_time,
            manifest,
            graph,
            policy,
        )?;

        let selected_profiles = entry
            .profile_ids
            .iter()
            .filter(|profile| requested.contains(profile.as_str()))
            .collect::<Vec<_>>();
        let selected = selected_profiles.len();
        if selected != 0 && selected != entry.profile_ids.len() {
            return Err(reject(
                "one receipt crosses the requested profile dependency boundary",
            ));
        }
        for profile in selected_profiles {
            selected_gate_profiles.insert(
                (entry.gate_id.as_str(), profile.as_str()),
                receipt.receipt_kind,
            );
        }
        selected_bindings += selected;
    }
    let expected_gate_profiles = requested_profiles
        .iter()
        .flat_map(|profile| {
            profile
                .gates
                .iter()
                .map(move |gate| ((gate.id, profile.id), gate.receipt_kind))
        })
        .collect::<BTreeMap<_, _>>();
    if selected_gate_profiles != expected_gate_profiles {
        return Err(reject(
            "registry gate/profile coverage differs from the selected profile closure",
        ));
    }
    if referenced.len() != objects.len()
        || objects
            .iter()
            .any(|object| !referenced.contains(object.subject.object_path.as_str()))
    {
        return Err(reject("registry contains an unreferenced object"));
    }
    Ok(selected_bindings)
}

fn validate_requested_graph(
    graph: &ReleaseProfileGraphV1,
    selected: &str,
    expected: &[RequestedProfile],
) -> Result<(), Reject> {
    if graph.family != "bullet-farm" {
        return Err(reject("profile graph names a different family"));
    }
    let nodes = graph
        .profiles
        .iter()
        .map(|node| (node.profile_id.as_str(), node))
        .collect::<BTreeMap<_, _>>();
    let expected = expected
        .iter()
        .map(|profile| {
            (
                profile.id,
                (
                    profile
                        .dependencies
                        .iter()
                        .copied()
                        .collect::<BTreeSet<_>>(),
                    profile
                        .gates
                        .iter()
                        .map(|gate| gate.id)
                        .collect::<BTreeSet<_>>(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for (profile, (dependencies, gate_ids)) in &expected {
        let node = nodes
            .get(profile)
            .ok_or_else(|| reject(format!("profile graph omits {profile}")))?;
        if node
            .dependency_profile_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            != *dependencies
        {
            return Err(reject(format!(
                "profile graph changes the declared dependencies of {profile}"
            )));
        }
        if node
            .gate_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            != *gate_ids
        {
            return Err(reject(format!(
                "profile graph changes the declared gates of {profile}"
            )));
        }
    }
    let mut actual = BTreeSet::new();
    collect_closure(selected, &nodes, &mut actual)?;
    if actual != expected.keys().copied().collect() {
        return Err(reject(
            "profile graph dependency closure differs from the requested profile",
        ));
    }
    Ok(())
}

fn collect_closure<'a>(
    profile: &'a str,
    nodes: &BTreeMap<&'a str, &'a bullet_wire::v1alpha1::ReleaseProfileNodeV1>,
    closure: &mut BTreeSet<&'a str>,
) -> Result<(), Reject> {
    if !closure.insert(profile) {
        return Ok(());
    }
    let node = nodes
        .get(profile)
        .ok_or_else(|| reject(format!("profile graph omits {profile}")))?;
    for dependency in &node.dependency_profile_ids {
        collect_closure(dependency, nodes, closure)?;
    }
    Ok(())
}

fn validate_entry(
    entry: &bullet_wire::v1alpha1::ReleaseRegistryEntryV1,
    request: &ReleaseGateVerificationRequestV1,
    receipt: &GateReceiptV1,
    time: &TrustedTimeObservationV1,
    manifest: &ReleaseRegistryManifestV1,
    graph: &ReleaseProfileGraphV1,
    policy: &ReleaseSignerPolicyV1,
) -> Result<(), Reject> {
    if entry.gate_id != receipt.gate_id
        || entry.profile_ids != receipt.profile_ids
        || entry.gate_receipt_id != receipt.gate_receipt_id
        || receipt.family_subject.family_lock_digest != manifest.family_lock_digest
    {
        return Err(reject("registry entry does not bind its exact receipt"));
    }
    if receipt.family_subject.family != graph.family
        || time.family != receipt.family_subject.family
        || time.gate_receipt_id != receipt.gate_receipt_id
        || time.receipt_digest != entry.receipt_digest
        || time.evidence_nonce != receipt.evidence_nonce
        || time.signer_policy_digest != manifest.signer_policy_digest
    {
        return Err(reject(
            "trusted-time observation does not bind its receipt and policy subject",
        ));
    }
    if time.trusted_time_key_id != policy.trusted_time_key_id {
        return Err(reject(
            "trusted-time observation names a key outside the signer policy role binding",
        ));
    }
    let attestor = policy_key(
        policy,
        &receipt.attestor_key_id,
        ReleaseSignerRoleV1::GateAttestor,
    )?;
    let time_key = policy_key(
        policy,
        &time.trusted_time_key_id,
        ReleaseSignerRoleV1::TrustedTime,
    )?;
    if receipt.started_at_unix_ms < attestor.activates_at_unix_ms
        || receipt.completed_at_unix_ms >= attestor.expires_at_unix_ms
        || attestor
            .revoked_at_unix_ms
            .is_some_and(|revoked| receipt.completed_at_unix_ms >= revoked)
    {
        return Err(reject(
            "gate receipt execution lies outside its attestor key lifecycle",
        ));
    }
    if request.requested_at_unix_ms < policy.activates_at_unix_ms
        || request.expires_at_unix_ms > policy.expires_at_unix_ms
        || receipt.started_at_unix_ms < policy.activates_at_unix_ms
        || receipt.expires_at_unix_ms > policy.expires_at_unix_ms
        || time.observed_at_unix_ms < policy.activates_at_unix_ms
        || time.valid_until_unix_ms > policy.expires_at_unix_ms
        || receipt.expires_at_unix_ms > request.expires_at_unix_ms
        || time.observed_at_unix_ms < receipt.completed_at_unix_ms
        || time.observed_at_unix_ms >= receipt.expires_at_unix_ms
        || time.observed_at_unix_ms < time_key.activates_at_unix_ms
        || time.valid_until_unix_ms > receipt.expires_at_unix_ms
        || time.valid_until_unix_ms > request.expires_at_unix_ms
        || time.valid_until_unix_ms > time_key.expires_at_unix_ms
        || time_key.revoked_at_unix_ms.is_some_and(|revoked| {
            time.observed_at_unix_ms >= revoked || time.valid_until_unix_ms > revoked
        })
        || manifest.created_at_unix_ms < time.observed_at_unix_ms
        || manifest.expires_at_unix_ms > time.valid_until_unix_ms
    {
        return Err(reject(
            "release request, receipt, trusted-time, and registry windows are incoherent",
        ));
    }
    Ok(())
}

fn validate_artifact_binding<'a>(
    receipt: &GateReceiptV1,
    objects: &[LoadedObject<'a>],
    referenced: &mut BTreeSet<&'a str>,
) -> Result<(), Reject> {
    if !receipt
        .evidence_subjects
        .iter()
        .any(|subject| subject.subject_kind == ReleaseEvidenceKindV1::Artifact)
    {
        return Ok(());
    }
    let mut manifests = objects.iter().filter(|object| {
        object.subject.object_kind == ReleaseRegistryObjectKindV1::ReleaseBundleManifestV2
    });
    let manifest = manifests
        .next()
        .ok_or_else(|| reject("release bundle manifest v2 requires exactly one registry object"))?;
    if manifests.next().is_some() {
        return Err(reject(
            "release bundle manifest v2 requires exactly one registry object",
        ));
    }
    validate_release_bundle_manifest_v2_binding(
        &receipt.evidence_subjects,
        std::slice::from_ref(manifest.subject),
        &manifest.bytes,
    )
    .map_err(|error| {
        reject(format!(
            "release bundle manifest binding is invalid: {error}"
        ))
    })?;
    referenced.insert(manifest.subject.object_path.as_str());
    Ok(())
}

fn validate_policy_binding(
    manifest: &ReleaseRegistryManifestV1,
    graph: &ReleaseProfileGraphV1,
    policy: &ReleaseSignerPolicyV1,
) -> Result<(), Reject> {
    if graph.family != policy.family
        || manifest.registry_signer_key_id != policy.registry_signer_key_id
        || manifest.created_at_unix_ms < policy.activates_at_unix_ms
        || manifest.expires_at_unix_ms > policy.expires_at_unix_ms
    {
        return Err(reject(
            "registry graph, manifest, and signer policy subjects are inconsistent",
        ));
    }
    let registry_key = policy_key(
        policy,
        &manifest.registry_signer_key_id,
        ReleaseSignerRoleV1::RegistryCurator,
    )?;
    if manifest.created_at_unix_ms < registry_key.activates_at_unix_ms
        || manifest.expires_at_unix_ms > registry_key.expires_at_unix_ms
        || registry_key.revoked_at_unix_ms.is_some_and(|revoked| {
            manifest.created_at_unix_ms >= revoked || manifest.expires_at_unix_ms > revoked
        })
    {
        return Err(reject(
            "registry manifest lies outside its curator key lifecycle",
        ));
    }
    Ok(())
}

fn policy_key<'a>(
    policy: &'a ReleaseSignerPolicyV1,
    key_id: &str,
    role: ReleaseSignerRoleV1,
) -> Result<&'a bullet_wire::v1alpha1::ReleaseSignerKeyV1, Reject> {
    policy
        .signer_keys
        .iter()
        .find(|key| key.key_id == key_id && key.role == role)
        .ok_or_else(|| reject("release record names a signer key with the wrong policy role"))
}
