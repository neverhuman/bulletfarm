use std::collections::{BTreeMap, BTreeSet};

use crate::{
    WireError,
    v1alpha1::{
        ReleaseGateSpecV1, ReleaseGateVerificationRequestV1, ReleaseProfileGraphV1,
        ReleaseProfileNodeV1, ReleaseRegistryObjectV1,
    },
};

use super::{fields::*, validate::ReleaseWireRecord};

impl ReleaseWireRecord for ReleaseProfileNodeV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        profile_id(&self.profile_id)?;
        sorted_unique_optional(
            &self.dependency_profile_ids,
            profile_id,
            "profile dependencies",
        )?;
        sorted_unique(&self.gate_ids, gate_id, "profile gate IDs")
    }
}

impl ReleaseWireRecord for ReleaseProfileGraphV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        typed_id(&self.profile_graph_id, "rpg")?;
        family(&self.family)?;
        positive(self.generation, "profile graph generation")?;
        if self.profiles.is_empty() || self.profiles.len() > 64 {
            return Err(invalid("profile graph requires 1..=64 profiles"));
        }

        let mut previous = None;
        let mut profiles = BTreeMap::new();
        for profile in &self.profiles {
            profile.validate_release()?;
            if previous.is_some_and(|prior: &str| prior >= profile.profile_id.as_str()) {
                return Err(invalid(
                    "profile graph nodes must be byte-sorted and unique",
                ));
            }
            profiles.insert(profile.profile_id.as_str(), profile);
            previous = Some(&profile.profile_id);
        }
        for profile in &self.profiles {
            if profile
                .dependency_profile_ids
                .iter()
                .any(|dependency| !profiles.contains_key(dependency.as_str()))
            {
                return Err(invalid("profile graph names an unknown dependency"));
            }
        }

        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        for profile in &self.profiles {
            visit_profile(
                profile.profile_id.as_str(),
                &profiles,
                &mut visiting,
                &mut visited,
            )?;
        }
        Ok(())
    }
}

fn visit_profile<'a>(
    profile_id: &'a str,
    profiles: &BTreeMap<&'a str, &'a ReleaseProfileNodeV1>,
    visiting: &mut BTreeSet<&'a str>,
    visited: &mut BTreeSet<&'a str>,
) -> Result<(), WireError> {
    if visited.contains(profile_id) {
        return Ok(());
    }
    if !visiting.insert(profile_id) {
        return Err(invalid("profile dependency graph contains a cycle"));
    }
    for dependency in &profiles[profile_id].dependency_profile_ids {
        visit_profile(dependency, profiles, visiting, visited)?;
    }
    visiting.remove(profile_id);
    visited.insert(profile_id);
    Ok(())
}

impl ReleaseWireRecord for ReleaseGateSpecV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        typed_id(&self.gate_spec_id, "gsp")?;
        gate_id(&self.gate_id)?;
        positive(self.gate_version, "gate version")?;
        sorted_unique(&self.profile_ids, profile_id, "gate spec profile IDs")?;
        if self.required_evidence_kinds.is_empty() || self.required_evidence_kinds.len() > 64 {
            return Err(invalid("gate spec requires 1..=64 evidence kinds"));
        }
        let mut previous = None;
        let mut kinds = BTreeSet::new();
        for kind in &self.required_evidence_kinds {
            let name = evidence_kind(*kind);
            if previous.is_some_and(|prior: &str| prior >= name) {
                return Err(invalid(
                    "required evidence kinds must be byte-sorted and unique",
                ));
            }
            kinds.insert(name);
            previous = Some(name);
        }
        if !REQUIRED_COMMON_EVIDENCE
            .iter()
            .all(|kind| kinds.contains(kind))
        {
            return Err(invalid(
                "gate spec omits a required policy/schema/toolchain/environment kind",
            ));
        }
        tagged_digest(&self.gate_policy_digest, "gate policy")
    }
}

impl ReleaseWireRecord for ReleaseGateVerificationRequestV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        typed_id(&self.verification_request_id, "rvr")?;
        gate_id(&self.gate_id)?;
        positive(self.gate_version, "gate version")?;
        sorted_unique(&self.profile_ids, profile_id, "request profile IDs")?;
        raw_digest(&self.evidence_nonce, "evidence nonce")?;
        tagged_digest(&self.gate_spec_digest, "gate spec")?;
        tagged_digest(&self.profile_graph_digest, "profile graph")?;
        tagged_digest(&self.gate_policy_digest, "gate policy")?;
        self.family_subject.validate_release()?;
        if self.evidence_subjects.is_empty() || self.evidence_subjects.len() > 256 {
            return Err(invalid("request evidence requires 1..=256 subjects"));
        }
        let mut previous = None;
        let mut evidence_kinds = BTreeSet::new();
        for subject in &self.evidence_subjects {
            subject.validate_release()?;
            let identity = (
                evidence_kind(subject.subject_kind),
                subject.subject_id.as_str(),
            );
            if previous.is_some_and(|prior| prior >= identity) {
                return Err(invalid(
                    "request evidence must be byte-sorted with unique identities",
                ));
            }
            evidence_kinds.insert(identity.0);
            previous = Some(identity);
        }
        if !REQUIRED_COMMON_EVIDENCE
            .iter()
            .all(|kind| evidence_kinds.contains(kind))
        {
            return Err(invalid(
                "request omits a required policy/schema/toolchain/environment subject",
            ));
        }
        ordered_pair(
            self.requested_at_unix_ms,
            self.expires_at_unix_ms,
            "verification request validity",
        )
    }
}

impl ReleaseWireRecord for ReleaseRegistryObjectV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        typed_id(&self.object_id, "rob")?;
        tagged_digest(&self.object_digest, "registry object")?;
        relative_path(&self.object_path)
    }
}
