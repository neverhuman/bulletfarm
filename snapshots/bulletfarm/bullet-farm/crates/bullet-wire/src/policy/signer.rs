use serde::{Deserialize, Serialize};

use crate::AuthorityAudience;

pub const DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE: &str = "dogfood-run-attestation-signing";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyPurposeV1 {
    AuthoritySigning,
    DogfoodLaunchSigning,
    DogfoodRunAttestationSigning,
    ProviderEnrollmentSigning,
    ReleaseSigning,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum KeyAlgorithmV1 {
    #[serde(rename = "paseto-v4.public")]
    PasetoV4Public,
    #[serde(rename = "ssh-ed25519")]
    SshEd25519,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IssuerKeyV1 {
    pub schema_version: String,
    pub issuer: String,
    pub key_id: String,
    pub key_purpose: KeyPurposeV1,
    pub algorithm: KeyAlgorithmV1,
    pub public_key: String,
    pub audiences: Vec<AuthorityAudience>,
    pub activates_at_unix_ms: u64,
    pub expires_at_unix_ms: u64,
    pub revoked_at_unix_ms: Option<u64>,
    pub retain_until_unix_ms: u64,
}
