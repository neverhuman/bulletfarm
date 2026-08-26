//! Structural release-registry admission. This module never establishes trust.

#[cfg(target_os = "linux")]
#[path = "semantic_registry/admission.rs"]
mod admission;
mod kinds;
#[cfg(not(target_os = "linux"))]
#[path = "semantic_registry/unsupported.rs"]
mod unsupported;
#[path = "semantic_registry/validation.rs"]
mod validation;

use std::{collections::BTreeSet, path::Path};

use bullet_wire::{
    RELEASE_GATE_RECEIPT_DIGEST_DOMAIN, RELEASE_GATE_SPEC_DIGEST_DOMAIN,
    RELEASE_PROFILE_GRAPH_DIGEST_DOMAIN, RELEASE_REGISTRY_OBJECT_DIGEST_DOMAIN,
    RELEASE_SIGNER_POLICY_DIGEST_DOMAIN, RELEASE_TRUSTED_TIME_DIGEST_DOMAIN,
    RELEASE_VERIFICATION_REQUEST_DIGEST_DOMAIN, decode_release_record, hash_canonical,
    hash_framed_bytes, release_bundle_manifest_v2_digest,
    v1alpha1::{
        GateReceiptV1, ReleaseGateSpecV1, ReleaseGateVerificationRequestV1, ReleaseProfileGraphV1,
        ReleaseReceiptKindV1, ReleaseRegistryManifestV1, ReleaseRegistryObjectKindV1,
        ReleaseRegistryObjectV1, ReleaseSignerPolicyV1, ReleaseSignerRoleV1,
        TrustedTimeObservationV1,
    },
    validate_release_bindings,
};

#[cfg(target_os = "linux")]
use admission::RegistryRoot;

const MANIFEST_PATH: &str = "registry-manifest.json";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Evaluation {
    Absent,
    Rejected(String),
    StructurallyValidButUntrusted { selected_bindings: usize },
}

#[derive(Clone, Debug)]
pub(super) struct RequestedProfile {
    id: &'static str,
    dependencies: Vec<&'static str>,
    gates: Vec<RequestedGate>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RequestedGate {
    id: &'static str,
    receipt_kind: ReleaseReceiptKindV1,
}

impl RequestedProfile {
    pub(super) fn new(
        id: &'static str,
        dependencies: Vec<&'static str>,
        gates: Vec<(&'static str, ReleaseReceiptKindV1)>,
    ) -> Self {
        Self {
            id,
            dependencies,
            gates: gates
                .into_iter()
                .map(|(id, receipt_kind)| RequestedGate { id, receipt_kind })
                .collect(),
        }
    }
}

pub(super) fn evaluate(
    registry: &Path,
    selected_profile: &str,
    requested_profiles: &[RequestedProfile],
) -> Evaluation {
    #[cfg(target_os = "linux")]
    {
        match evaluate_unix(registry, selected_profile, requested_profiles) {
            Ok(None) => Evaluation::Absent,
            Ok(Some(selected_bindings)) => {
                Evaluation::StructurallyValidButUntrusted { selected_bindings }
            }
            Err(error) => Evaluation::Rejected(error.detail),
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (selected_profile, requested_profiles);
        unsupported::evaluate(registry, MANIFEST_PATH)
    }
}

#[derive(Debug)]
pub(super) struct Reject {
    detail: String,
}

pub(super) fn reject(detail: impl Into<String>) -> Reject {
    Reject {
        detail: detail.into(),
    }
}

#[cfg(target_os = "linux")]
fn evaluate_unix(
    path: &Path,
    selected_profile: &str,
    requested_profiles: &[RequestedProfile],
) -> Result<Option<usize>, Reject> {
    let Some(root) = RegistryRoot::open(path)? else {
        return Ok(None);
    };
    let mut remaining_bytes = admission::MAX_REGISTRY_TOTAL_BYTES;
    let Some(manifest_input) = root.read_optional(MANIFEST_PATH, &mut remaining_bytes)? else {
        return Ok(None);
    };
    let manifest: ReleaseRegistryManifestV1 = decode(&manifest_input.bytes, "registry manifest")?;
    let mut identities = BTreeSet::from([manifest_input.identity_key()]);
    let mut loaded = Vec::with_capacity(manifest.objects.len());
    let mut unique_contents = BTreeSet::new();
    for object in &manifest.objects {
        let input = root.read_required(&object.object_path, &mut remaining_bytes)?;
        if !identities.insert(input.identity_key()) {
            return Err(reject("registry paths resolve to the same admitted object"));
        }
        let decoded = DecodedObject::decode(object.object_kind, &input.bytes)?;
        let actual = decoded.digest(&input.bytes)?;
        if actual != object.object_digest {
            return Err(reject(format!(
                "{} digest differs from its manifest binding",
                object.object_path
            )));
        }
        if !unique_contents.insert((kind_name(object.object_kind), actual)) {
            return Err(reject(
                "registry object kind and digest bindings must be unique",
            ));
        }
        loaded.push(LoadedObject {
            subject: object,
            decoded,
            bytes: input.bytes,
        });
    }
    root.ensure_identity()?;
    validation::validate_registry(&manifest, &loaded, selected_profile, requested_profiles)
        .map(Some)
}

fn object<'a>(
    objects: &'a [LoadedObject<'a>],
    kind: ReleaseRegistryObjectKindV1,
    digest: &str,
) -> Result<&'a LoadedObject<'a>, Reject> {
    let mut matches = objects.iter().filter(|object| {
        object.subject.object_kind == kind && object.subject.object_digest == digest
    });
    let found = matches
        .next()
        .ok_or_else(|| reject(format!("registry omits {}/{}", kind_name(kind), digest)))?;
    if matches.next().is_some() {
        return Err(reject("registry object reference is ambiguous"));
    }
    Ok(found)
}

fn exact_object<'a>(
    objects: &'a [LoadedObject<'a>],
    kind: ReleaseRegistryObjectKindV1,
    digest: &str,
    path: &str,
) -> Result<&'a LoadedObject<'a>, Reject> {
    let found = object(objects, kind, digest)?;
    if found.subject.object_path != path {
        return Err(reject(
            "registry object path differs from its entry binding",
        ));
    }
    Ok(found)
}

struct LoadedObject<'a> {
    subject: &'a ReleaseRegistryObjectV1,
    decoded: DecodedObject,
    bytes: Vec<u8>,
}

enum DecodedObject {
    Receipt(GateReceiptV1),
    GateSpec(ReleaseGateSpecV1),
    Graph(ReleaseProfileGraphV1),
    SignerPolicy(ReleaseSignerPolicyV1),
    TrustedTime(TrustedTimeObservationV1),
    Request(ReleaseGateVerificationRequestV1),
    Signature(ReleaseRegistryObjectKindV1),
    ReleaseBundleManifestV2,
}

impl DecodedObject {
    fn decode(kind: ReleaseRegistryObjectKindV1, bytes: &[u8]) -> Result<Self, Reject> {
        Ok(match kind {
            ReleaseRegistryObjectKindV1::GateReceipt => {
                Self::Receipt(decode(bytes, "gate receipt")?)
            }
            ReleaseRegistryObjectKindV1::GateReceiptSignature => {
                Self::Signature(ReleaseRegistryObjectKindV1::GateReceiptSignature)
            }
            ReleaseRegistryObjectKindV1::TrustedTimeSignature => {
                Self::Signature(ReleaseRegistryObjectKindV1::TrustedTimeSignature)
            }
            ReleaseRegistryObjectKindV1::GateSpec => Self::GateSpec(decode(bytes, "gate spec")?),
            ReleaseRegistryObjectKindV1::ProfileGraph => {
                Self::Graph(decode(bytes, "profile graph")?)
            }
            ReleaseRegistryObjectKindV1::SignerPolicy => {
                Self::SignerPolicy(decode(bytes, "signer policy")?)
            }
            ReleaseRegistryObjectKindV1::TrustedTimeObservation => {
                Self::TrustedTime(decode(bytes, "trusted time")?)
            }
            ReleaseRegistryObjectKindV1::VerificationRequest => {
                Self::Request(decode(bytes, "verification request")?)
            }
            ReleaseRegistryObjectKindV1::ReleaseBundleManifestV2 => Self::ReleaseBundleManifestV2,
        })
    }

    fn digest(&self, bytes: &[u8]) -> Result<String, Reject> {
        match self {
            Self::Receipt(value) => canonical_digest(RELEASE_GATE_RECEIPT_DIGEST_DOMAIN, value),
            Self::GateSpec(value) => canonical_digest(RELEASE_GATE_SPEC_DIGEST_DOMAIN, value),
            Self::Graph(value) => canonical_digest(RELEASE_PROFILE_GRAPH_DIGEST_DOMAIN, value),
            Self::SignerPolicy(value) => {
                canonical_digest(RELEASE_SIGNER_POLICY_DIGEST_DOMAIN, value)
            }
            Self::TrustedTime(value) => canonical_digest(RELEASE_TRUSTED_TIME_DIGEST_DOMAIN, value),
            Self::Request(value) => {
                canonical_digest(RELEASE_VERIFICATION_REQUEST_DIGEST_DOMAIN, value)
            }
            Self::Signature(kind) => registry_blob_digest(*kind, bytes),
            Self::ReleaseBundleManifestV2 => release_bundle_manifest_v2_digest(bytes)
                .map_err(|error| reject(format!("invalid release bundle manifest: {error}"))),
        }
    }

    fn receipt(&self) -> Result<&GateReceiptV1, Reject> {
        match self {
            Self::Receipt(value) => Ok(value),
            _ => Err(reject("object is not a gate receipt")),
        }
    }
    fn gate_spec(&self) -> Result<&ReleaseGateSpecV1, Reject> {
        match self {
            Self::GateSpec(value) => Ok(value),
            _ => Err(reject("object is not a gate spec")),
        }
    }
    fn graph(&self) -> Result<&ReleaseProfileGraphV1, Reject> {
        match self {
            Self::Graph(value) => Ok(value),
            _ => Err(reject("object is not a profile graph")),
        }
    }
    fn signer_policy(&self) -> Result<&ReleaseSignerPolicyV1, Reject> {
        match self {
            Self::SignerPolicy(value) => Ok(value),
            _ => Err(reject("object is not a signer policy")),
        }
    }
    fn trusted_time(&self) -> Result<&TrustedTimeObservationV1, Reject> {
        match self {
            Self::TrustedTime(value) => Ok(value),
            _ => Err(reject("object is not trusted time")),
        }
    }
    fn request(&self) -> Result<&ReleaseGateVerificationRequestV1, Reject> {
        match self {
            Self::Request(value) => Ok(value),
            _ => Err(reject("object is not a verification request")),
        }
    }
}

fn registry_blob_digest(kind: ReleaseRegistryObjectKindV1, bytes: &[u8]) -> Result<String, Reject> {
    let kind = kind_name(kind).as_bytes();
    let mut framed = Vec::with_capacity(16 + kind.len() + bytes.len());
    framed.extend_from_slice(&(kind.len() as u64).to_le_bytes());
    framed.extend_from_slice(kind);
    framed.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    framed.extend_from_slice(bytes);
    hash_framed_bytes(RELEASE_REGISTRY_OBJECT_DIGEST_DOMAIN, &framed)
        .map(|digest| format!("blake3:{}", digest.to_hex()))
        .map_err(|error| reject(format!("cannot hash detached signature object: {error}")))
}

fn canonical_digest<T: serde::Serialize>(domain: &str, value: &T) -> Result<String, Reject> {
    hash_canonical(domain, value)
        .map(|digest| format!("blake3:{}", digest.to_hex()))
        .map_err(|error| reject(format!("cannot hash release object: {error}")))
}

fn decode<T: bullet_wire::ReleaseWireRecord>(bytes: &[u8], label: &str) -> Result<T, Reject> {
    decode_release_record(bytes).map_err(|error| reject(format!("invalid {label}: {error}")))
}

fn kind_name(kind: ReleaseRegistryObjectKindV1) -> &'static str {
    match kind {
        ReleaseRegistryObjectKindV1::GateReceipt => "gate-receipt",
        ReleaseRegistryObjectKindV1::GateReceiptSignature => "gate-receipt-signature",
        ReleaseRegistryObjectKindV1::GateSpec => "gate-spec",
        ReleaseRegistryObjectKindV1::ProfileGraph => "profile-graph",
        ReleaseRegistryObjectKindV1::SignerPolicy => "signer-policy",
        ReleaseRegistryObjectKindV1::TrustedTimeObservation => "trusted-time-observation",
        ReleaseRegistryObjectKindV1::TrustedTimeSignature => "trusted-time-signature",
        ReleaseRegistryObjectKindV1::VerificationRequest => "verification-request",
        ReleaseRegistryObjectKindV1::ReleaseBundleManifestV2 => "release-bundle-manifest-v2",
    }
}
