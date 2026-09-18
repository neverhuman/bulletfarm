use std::collections::BTreeSet;

use serde::{Serialize, de::DeserializeOwned};

use crate::{
    WireError, decode_canonical,
    v1alpha1::{
        GateReceiptV1, ReleaseEvidenceSubjectV1, ReleaseFamilySubjectV1, ReleaseRegistryEntryV1,
        ReleaseRegistryManifestV1, ReleaseRegistryObjectKindV1, ReleaseReplayBindingV1,
        ReleaseReplayStateV1, ReleaseRepositorySubjectV1, ReleaseSignerKeyV1,
        ReleaseSignerPolicyV1, TrustedTimeObservationV1,
    },
};

use super::fields::*;

pub trait ReleaseWireRecord: DeserializeOwned + Serialize {
    fn validate_release(&self) -> Result<(), WireError>;
}

pub fn decode_release_record<T: ReleaseWireRecord>(bytes: &[u8]) -> Result<T, WireError> {
    let record: T = decode_canonical(bytes)?;
    record.validate_release()?;
    Ok(record)
}

impl ReleaseWireRecord for GateReceiptV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        typed_id(&self.gate_receipt_id, "grc")?;
        gate_id(&self.gate_id)?;
        if self.gate_version == 0 || self.gate_version > MAX_SAFE_INTEGER {
            return Err(invalid("gate version is outside the admitted range"));
        }
        sorted_unique(&self.profile_ids, profile_id, "profile IDs")?;
        raw_digest(&self.evidence_nonce, "evidence nonce")?;
        for (label, digest) in [
            ("request", &self.request_digest),
            ("gate spec", &self.gate_spec_digest),
            ("profile graph", &self.profile_graph_digest),
            ("gate policy", &self.gate_policy_digest),
        ] {
            tagged_digest(digest, label)?;
        }
        self.family_subject.validate_release()?;
        if self.evidence_subjects.is_empty() || self.evidence_subjects.len() > 256 {
            return Err(invalid("evidence subjects require 1..=256 entries"));
        }
        let mut evidence_kinds = BTreeSet::new();
        let mut previous = None;
        for subject in &self.evidence_subjects {
            subject.validate_release()?;
            let identity = (
                evidence_kind(subject.subject_kind),
                subject.subject_id.as_str(),
            );
            if previous.is_some_and(|prior| prior >= identity) {
                return Err(invalid(
                    "evidence subjects must be byte-sorted with unique identities",
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
                "receipt omits a required policy/schema/toolchain/environment subject",
            ));
        }
        key_id(&self.attestor_key_id)?;
        ordered_times(
            self.started_at_unix_ms,
            self.completed_at_unix_ms,
            self.expires_at_unix_ms,
            "receipt",
        )
    }
}

impl ReleaseWireRecord for ReleaseFamilySubjectV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        family(&self.family)?;
        tagged_digest(&self.family_lock_digest, "family lock")?;
        tagged_digest(&self.schema_bundle_digest, "schema bundle")?;
        if self.repositories.len() != REQUIRED_REPOSITORIES.len() {
            return Err(invalid("family subject requires exactly four repositories"));
        }
        for (subject, expected) in self.repositories.iter().zip(REQUIRED_REPOSITORIES) {
            subject.validate_release()?;
            if repository_name(subject.repository) != expected {
                return Err(invalid(
                    "family repositories must be canonical, byte-sorted, and complete",
                ));
            }
        }
        Ok(())
    }
}

impl ReleaseWireRecord for ReleaseRepositorySubjectV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        release_tag(&self.tag)?;
        git_oid(&self.commit_oid)?;
        git_oid(&self.tree_oid)?;
        signing_identity(&self.release_signing_identity)?;
        tagged_digest(&self.source_subject_digest, "source subject")
    }
}

impl ReleaseWireRecord for ReleaseEvidenceSubjectV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        typed_id(&self.subject_id, "cnt")?;
        native_subject_id(&self.native_subject_id, self.subject_kind)?;
        tagged_digest(&self.subject_digest, "evidence subject")
    }
}

impl ReleaseWireRecord for ReleaseRegistryEntryV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        gate_id(&self.gate_id)?;
        sorted_unique(&self.profile_ids, profile_id, "registry entry profile IDs")?;
        typed_id(&self.gate_receipt_id, "grc")?;
        for (label, digest) in [
            ("receipt", &self.receipt_digest),
            ("receipt signature", &self.receipt_signature_digest),
            ("trusted time", &self.trusted_time_digest),
            (
                "trusted-time signature",
                &self.trusted_time_signature_digest,
            ),
        ] {
            tagged_digest(digest, label)?;
        }
        for path in [
            &self.receipt_path,
            &self.receipt_signature_path,
            &self.trusted_time_path,
            &self.trusted_time_signature_path,
        ] {
            relative_path(path)?;
        }
        Ok(())
    }
}

impl ReleaseWireRecord for ReleaseRegistryManifestV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        typed_id(&self.registry_id, "rrg")?;
        positive(self.generation, "registry generation")?;
        for (label, digest) in [
            ("previous registry", &self.previous_registry_digest),
            ("signer policy", &self.signer_policy_digest),
            ("profile graph", &self.profile_graph_digest),
            ("family lock", &self.family_lock_digest),
        ] {
            tagged_digest(digest, label)?;
        }
        ordered_pair(
            self.created_at_unix_ms,
            self.expires_at_unix_ms,
            "registry validity",
        )?;
        key_id(&self.registry_signer_key_id)?;
        if self.objects.is_empty() || self.objects.len() > 2_048 {
            return Err(invalid("registry objects require 1..=2048 entries"));
        }
        let mut previous_object = None;
        let mut object_ids = BTreeSet::new();
        let mut object_paths = BTreeSet::new();
        for object in &self.objects {
            object.validate_release()?;
            let identity = (
                registry_object_kind(object.object_kind),
                object.object_id.as_str(),
            );
            if previous_object.is_some_and(|prior| prior >= identity) {
                return Err(invalid(
                    "registry objects must be sorted by kind and object ID",
                ));
            }
            if !object_ids.insert(object.object_id.as_str())
                || !object_paths.insert(object.object_path.as_str())
            {
                return Err(invalid("registry object IDs and paths must be unique"));
            }
            previous_object = Some(identity);
        }
        if self.entries.len() > 512 {
            return Err(invalid("registry exceeds 512 active entries"));
        }
        let mut previous = None;
        let mut gate_profiles = BTreeSet::new();
        for entry in &self.entries {
            entry.validate_release()?;
            let identity = (
                entry.gate_id.as_str(),
                entry.profile_ids.as_slice(),
                entry.gate_receipt_id.as_str(),
            );
            if previous.is_some_and(|prior| prior >= identity) {
                return Err(invalid(
                    "registry entries must be sorted by gate, profiles, and receipt ID",
                ));
            }
            for profile in &entry.profile_ids {
                if !gate_profiles.insert((entry.gate_id.as_str(), profile.as_str())) {
                    return Err(invalid(
                        "registry entries overlap for the same gate and profile",
                    ));
                }
            }
            for (kind, digest, path) in [
                (
                    ReleaseRegistryObjectKindV1::GateReceipt,
                    entry.receipt_digest.as_str(),
                    entry.receipt_path.as_str(),
                ),
                (
                    ReleaseRegistryObjectKindV1::GateReceiptSignature,
                    entry.receipt_signature_digest.as_str(),
                    entry.receipt_signature_path.as_str(),
                ),
                (
                    ReleaseRegistryObjectKindV1::TrustedTimeObservation,
                    entry.trusted_time_digest.as_str(),
                    entry.trusted_time_path.as_str(),
                ),
                (
                    ReleaseRegistryObjectKindV1::TrustedTimeSignature,
                    entry.trusted_time_signature_digest.as_str(),
                    entry.trusted_time_signature_path.as_str(),
                ),
            ] {
                if !self.objects.iter().any(|object| {
                    object.object_kind == kind
                        && object.object_digest == digest
                        && object.object_path == path
                }) {
                    return Err(invalid("registry entry references an unmanifested object"));
                }
            }
            previous = Some(identity);
        }
        Ok(())
    }
}

impl ReleaseWireRecord for ReleaseReplayBindingV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        raw_digest(&self.evidence_nonce, "evidence nonce")?;
        typed_id(&self.gate_receipt_id, "grc")?;
        gate_id(&self.gate_id)?;
        tagged_digest(&self.request_digest, "request")?;
        tagged_digest(&self.receipt_digest, "receipt")
    }
}

impl ReleaseWireRecord for ReleaseReplayStateV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        typed_id(&self.registry_id, "rrg")?;
        positive(self.generation, "replay generation")?;
        positive(self.restore_epoch, "restore epoch")?;
        tagged_digest(&self.registry_manifest_digest, "registry manifest")?;
        tagged_digest(&self.previous_state_digest, "previous replay state")?;
        positive(self.trusted_time_floor_unix_ms, "trusted-time floor")?;
        key_id(&self.registry_signer_key_id)?;
        if self.bindings.len() > 512 {
            return Err(invalid("replay state exceeds 512 active bindings"));
        }
        let mut previous = None;
        for binding in &self.bindings {
            binding.validate_release()?;
            if previous.is_some_and(|nonce: &str| nonce >= binding.evidence_nonce.as_str()) {
                return Err(invalid(
                    "replay bindings must be byte-sorted with unique nonces",
                ));
            }
            previous = Some(&binding.evidence_nonce);
        }
        Ok(())
    }
}

impl ReleaseWireRecord for ReleaseSignerKeyV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        key_id(&self.key_id)?;
        signing_identity(&self.signing_identity)?;
        public_key(&self.public_key)?;
        ordered_pair(
            self.activates_at_unix_ms,
            self.expires_at_unix_ms,
            "signer validity",
        )?;
        if self.retain_until_unix_ms < self.expires_at_unix_ms {
            return Err(invalid("signer retention ends before key expiry"));
        }
        positive(self.retain_until_unix_ms, "signer retention")?;
        if self.revoked_at_unix_ms.is_some_and(|revoked| {
            revoked < self.activates_at_unix_ms || revoked >= self.expires_at_unix_ms
        }) {
            return Err(invalid("signer revocation lies outside key validity"));
        }
        Ok(())
    }
}

impl ReleaseWireRecord for ReleaseSignerPolicyV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        family(&self.family)?;
        positive(self.policy_generation, "policy generation")?;
        ordered_pair(
            self.activates_at_unix_ms,
            self.expires_at_unix_ms,
            "policy validity",
        )?;
        key_id(&self.registry_signer_key_id)?;
        key_id(&self.trusted_time_key_id)?;
        if !(5..=64).contains(&self.signer_keys.len()) {
            return Err(invalid("signer policy requires 5..=64 role keys"));
        }
        let mut previous = None;
        let mut public_keys = BTreeSet::new();
        let mut roles = BTreeSet::new();
        let mut registry_role = None;
        let mut time_role = None;
        for key in &self.signer_keys {
            key.validate_release()?;
            if previous.is_some_and(|id: &str| id >= key.key_id.as_str()) {
                return Err(invalid("signer keys must be byte-sorted and unique"));
            }
            if !public_keys.insert(key.public_key.as_str()) {
                return Err(invalid(
                    "release signer roles must use distinct Ed25519 keys",
                ));
            }
            let role = signer_role(key.role);
            roles.insert(role);
            if key.key_id == self.registry_signer_key_id {
                registry_role = Some(role);
            }
            if key.key_id == self.trusted_time_key_id {
                time_role = Some(role);
            }
            previous = Some(&key.key_id);
        }
        if !REQUIRED_SIGNER_ROLES
            .iter()
            .all(|role| roles.contains(role))
            || registry_role != Some("registry-curator")
            || time_role != Some("trusted-time")
        {
            return Err(invalid(
                "signer policy omits a role or assigns registry/time keys incorrectly",
            ));
        }
        Ok(())
    }
}

impl ReleaseWireRecord for TrustedTimeObservationV1 {
    fn validate_release(&self) -> Result<(), WireError> {
        schema(&self.schema_version)?;
        family(&self.family)?;
        typed_id(&self.gate_receipt_id, "grc")?;
        tagged_digest(&self.receipt_digest, "receipt")?;
        raw_digest(&self.evidence_nonce, "evidence nonce")?;
        tagged_digest(&self.signer_policy_digest, "signer policy")?;
        positive(self.restore_epoch, "restore epoch")?;
        ordered_pair(
            self.observed_at_unix_ms,
            self.valid_until_unix_ms,
            "trusted-time validity",
        )?;
        key_id(&self.trusted_time_key_id)
    }
}
