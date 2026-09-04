//! Provider observations that close enrollment digests without carrying host or authority facts.

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    Blake3Digest, CredentialProjectionProfileId, DogfoodProviderProtocolV1, LaunchProvider,
    PrincipalId, ProviderEnrollmentClaimsV2, ProviderProfileId, ProviderRuntimePassportV1,
    RuntimePassportError, RuntimePassportId, WireError, decode_canonical, hash_canonical,
    ids::{is_bounded_wire_label, require_exact_wire},
};

use super::DOGFOOD_SCHEMA_VERSION;

pub const PROVIDER_PROBE_OBSERVATION_DIGEST_DOMAIN: &str =
    "dogfood.provider-probe-observation.v1alpha1";
pub const PROVIDER_ENDPOINT_OBSERVATION_DIGEST_DOMAIN: &str =
    "dogfood.provider-endpoint-observation.v1alpha1";
pub const PROVIDER_VERSION_OBSERVATION_DIGEST_DOMAIN: &str =
    "dogfood.provider-version-observation.v1alpha1";
pub const PROVIDER_PROFILE_OBSERVATION_DIGEST_DOMAIN: &str =
    "dogfood.provider-profile-observation.v1alpha1";
pub const MAX_PROVIDER_OBSERVATION_BYTES: usize = 8 * 1024;
pub const MAX_PROVIDER_OBSERVATION_STALENESS_MS: u64 = 300_000;

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const MAX_VERSION_BYTES: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderObservationSubjectV1 {
    pub provider: LaunchProvider,
    pub protocol: DogfoodProviderProtocolV1,
    pub runtime_passport_id: RuntimePassportId,
    pub provider_profile_id: ProviderProfileId,
    pub service_identity_id: PrincipalId,
    pub credential_projection_profile_id: CredentialProjectionProfileId,
    pub policy_snapshot_digest: Blake3Digest,
    pub policy_generation: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderProbeObservationV1 {
    pub schema_version: String,
    pub subject: ProviderObservationSubjectV1,
    pub probe_grant_digest: Blake3Digest,
    pub containment_receipt_digest: Blake3Digest,
    pub protocol_transcript_digest: Blake3Digest,
    pub observed_at_unix_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderEndpointObservationV1 {
    pub schema_version: String,
    pub subject: ProviderObservationSubjectV1,
    pub probe_observation_digest: Blake3Digest,
    pub entrypoint_blake3: Blake3Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderVersionObservationV1 {
    pub schema_version: String,
    pub subject: ProviderObservationSubjectV1,
    pub probe_observation_digest: Blake3Digest,
    pub runtime_version: String,
    pub native_version_artifact_digest: Blake3Digest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderProfileObservationV1 {
    pub schema_version: String,
    pub subject: ProviderObservationSubjectV1,
    pub probe_observation_digest: Blake3Digest,
    pub effective_identity_artifact_digest: Blake3Digest,
}

impl ProviderObservationSubjectV1 {
    fn validate(&self, code: &'static str) -> Result<(), WireError> {
        let expected = DogfoodProviderProtocolV1::required_for(self.provider);
        if self.protocol != expected {
            return Err(invalid(
                code,
                format!(
                    "provider {} requires {}, got {}",
                    self.provider.as_str(),
                    expected.as_str(),
                    self.protocol.as_str()
                ),
            ));
        }
        if self.policy_generation == 0 || self.policy_generation > MAX_SAFE_INTEGER {
            return Err(invalid(
                code,
                "policy_generation must be a positive safe integer",
            ));
        }
        Ok(())
    }
}

impl ProviderProbeObservationV1 {
    pub fn validate(&self) -> Result<(), WireError> {
        let code = "PROVIDER_PROBE_OBSERVATION_INVALID";
        require_schema(&self.schema_version, code)?;
        self.subject.validate(code)?;
        if self.observed_at_unix_ms == 0 || self.observed_at_unix_ms > MAX_SAFE_INTEGER {
            return Err(invalid(
                code,
                "observed_at_unix_ms must be a positive safe integer",
            ));
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate()?;
        hash_canonical(PROVIDER_PROBE_OBSERVATION_DIGEST_DOMAIN, self)
    }
}

impl ProviderEndpointObservationV1 {
    pub fn validate(&self) -> Result<(), WireError> {
        let code = "PROVIDER_ENDPOINT_OBSERVATION_INVALID";
        require_schema(&self.schema_version, code)?;
        self.subject.validate(code)
    }

    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate()?;
        hash_canonical(PROVIDER_ENDPOINT_OBSERVATION_DIGEST_DOMAIN, self)
    }
}

impl ProviderVersionObservationV1 {
    pub fn validate(&self) -> Result<(), WireError> {
        let code = "PROVIDER_VERSION_OBSERVATION_INVALID";
        require_schema(&self.schema_version, code)?;
        self.subject.validate(code)?;
        if !is_bounded_wire_label(&self.runtime_version, MAX_VERSION_BYTES) {
            return Err(invalid(
                code,
                "runtime_version must be bounded identifier text",
            ));
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate()?;
        hash_canonical(PROVIDER_VERSION_OBSERVATION_DIGEST_DOMAIN, self)
    }
}

impl ProviderProfileObservationV1 {
    pub fn validate(&self) -> Result<(), WireError> {
        let code = "PROVIDER_PROFILE_OBSERVATION_INVALID";
        require_schema(&self.schema_version, code)?;
        self.subject.validate(code)
    }

    pub fn digest(&self) -> Result<Blake3Digest, WireError> {
        self.validate()?;
        hash_canonical(PROVIDER_PROFILE_OBSERVATION_DIGEST_DOMAIN, self)
    }
}

pub fn decode_provider_probe_observation(
    bytes: &[u8],
) -> Result<ProviderProbeObservationV1, WireError> {
    decode_observation(
        bytes,
        "PROVIDER_PROBE_OBSERVATION_INVALID",
        |value: &ProviderProbeObservationV1| value.validate(),
    )
}

pub fn decode_provider_endpoint_observation(
    bytes: &[u8],
) -> Result<ProviderEndpointObservationV1, WireError> {
    decode_observation(
        bytes,
        "PROVIDER_ENDPOINT_OBSERVATION_INVALID",
        |value: &ProviderEndpointObservationV1| value.validate(),
    )
}

pub fn decode_provider_version_observation(
    bytes: &[u8],
) -> Result<ProviderVersionObservationV1, WireError> {
    decode_observation(
        bytes,
        "PROVIDER_VERSION_OBSERVATION_INVALID",
        |value: &ProviderVersionObservationV1| value.validate(),
    )
}

pub fn decode_provider_profile_observation(
    bytes: &[u8],
) -> Result<ProviderProfileObservationV1, WireError> {
    decode_observation(
        bytes,
        "PROVIDER_PROFILE_OBSERVATION_INVALID",
        |value: &ProviderProfileObservationV1| value.validate(),
    )
}

pub fn verify_provider_observations(
    enrollment: &ProviderEnrollmentClaimsV2,
    passport: &ProviderRuntimePassportV1,
    probe: &ProviderProbeObservationV1,
    endpoint: &ProviderEndpointObservationV1,
    version: &ProviderVersionObservationV1,
    profile: &ProviderProfileObservationV1,
) -> Result<(), WireError> {
    enrollment.validate()?;
    passport.validate().map_err(runtime_error)?;
    probe.validate()?;
    endpoint.validate()?;
    version.validate()?;
    profile.validate()?;

    let subject = &probe.subject;
    let passport_id = passport.passport_id().map_err(runtime_error)?;
    if endpoint.subject != *subject
        || version.subject != *subject
        || profile.subject != *subject
        || subject.provider != enrollment.provider
        || subject.protocol != enrollment.protocol
        || subject.runtime_passport_id != enrollment.runtime_passport_id
        || subject.runtime_passport_id != passport_id
        || subject.provider_profile_id != enrollment.provider_profile_id
        || subject.service_identity_id != enrollment.service_identity_id
        || subject.credential_projection_profile_id != enrollment.credential_projection_profile_id
        || subject.policy_snapshot_digest != enrollment.policy_snapshot_digest
        || subject.policy_generation != enrollment.policy_generation
    {
        return Err(invalid(
            "PROVIDER_OBSERVATION_SUBJECT_MISMATCH",
            "provider observations do not bind the exact enrollment and runtime subject",
        ));
    }

    if probe.observed_at_unix_ms > enrollment.activates_at_unix_ms
        || enrollment.activates_at_unix_ms - probe.observed_at_unix_ms
            > MAX_PROVIDER_OBSERVATION_STALENESS_MS
    {
        return Err(invalid(
            "PROVIDER_OBSERVATION_TIME_MISMATCH",
            "provider probe is after activation or more than five minutes stale",
        ));
    }

    let probe_digest = probe.digest()?;
    if endpoint.probe_observation_digest != probe_digest
        || version.probe_observation_digest != probe_digest
        || profile.probe_observation_digest != probe_digest
    {
        return Err(invalid(
            "PROVIDER_PROBE_OBSERVATION_MISMATCH",
            "provider observations do not bind the same probe",
        ));
    }

    let entrypoint = passport
        .files
        .iter()
        .find(|file| file.path == passport.entrypoint)
        .ok_or_else(|| {
            invalid(
                "PROVIDER_ENDPOINT_OBSERVATION_MISMATCH",
                "runtime passport has no entrypoint manifest member",
            )
        })?;
    if endpoint.entrypoint_blake3.to_hex() != entrypoint.blake3
        || endpoint.digest()? != enrollment.endpoint_observation_digest
    {
        return Err(invalid(
            "PROVIDER_ENDPOINT_OBSERVATION_MISMATCH",
            "endpoint observation does not bind the passport entrypoint and enrollment",
        ));
    }
    if version.runtime_version.as_bytes() != passport.version.as_bytes()
        || version.runtime_version.as_bytes() != enrollment.runtime_version.as_bytes()
        || version.digest()? != enrollment.version_observation_digest
    {
        return Err(invalid(
            "PROVIDER_VERSION_OBSERVATION_MISMATCH",
            "version observation does not bind the runtime version and enrollment",
        ));
    }
    if profile.digest()? != enrollment.profile_observation_digest {
        return Err(invalid(
            "PROVIDER_PROFILE_OBSERVATION_MISMATCH",
            "profile observation does not bind the enrollment",
        ));
    }
    Ok(())
}

fn decode_observation<T>(
    bytes: &[u8],
    code: &'static str,
    validate: impl FnOnce(&T) -> Result<(), WireError>,
) -> Result<T, WireError>
where
    T: DeserializeOwned + Serialize,
{
    if bytes.len() > MAX_PROVIDER_OBSERVATION_BYTES {
        return Err(invalid(code, "provider observation exceeds 8 KiB"));
    }
    let value = decode_canonical(bytes).map_err(|error| invalid(code, error.to_string()))?;
    validate(&value)?;
    Ok(value)
}

fn require_schema(actual: &str, code: &'static str) -> Result<(), WireError> {
    require_exact_wire("schema_version", actual, DOGFOOD_SCHEMA_VERSION, code)
}

fn runtime_error(error: RuntimePassportError) -> WireError {
    WireError::new(error.reason_code(), error.to_string())
}

fn invalid(code: &'static str, reason: impl Into<String>) -> WireError {
    WireError::new(code, reason)
}
