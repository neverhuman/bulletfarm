use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::{Blake3Digest, LaunchProvider, WireError, digest::validate_lower_hex};

macro_rules! digest_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            pub const PREFIX: &'static str = $prefix;

            pub fn from_digest(digest: Blake3Digest) -> Self {
                Self(format!("{}{}", Self::PREFIX, digest.to_hex()))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }

            pub(crate) fn parse_checked(raw: &str) -> Result<Self, WireError> {
                let hex = raw.strip_prefix(Self::PREFIX).ok_or_else(|| {
                    WireError::new(
                        "INVALID_ID",
                        format!("{} must start with {}", stringify!($name), Self::PREFIX),
                    )
                })?;
                validate_lower_hex(hex, 64, "INVALID_ID")?;
                Ok(Self(raw.to_owned()))
            }
        }

        impl FromStr for $name {
            type Err = WireError;

            fn from_str(raw: &str) -> Result<Self, Self::Err> {
                Self::parse_checked(raw)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_tuple(stringify!($name))
                    .field(&self.0)
                    .finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let raw = String::deserialize(deserializer)?;
                Self::parse_checked(&raw).map_err(de::Error::custom)
            }
        }
    };
}

digest_id!(OrganizationId, "org_");
digest_id!(RepositoryId, "rep_");
digest_id!(MissionId, "mis_");
digest_id!(AcceptanceContractId, "acc_");
digest_id!(PlanRevisionId, "pln_");
digest_id!(GraphRevisionId, "grf_");
digest_id!(WorkPackageId, "wpk_");
digest_id!(SelectionGroupId, "sel_");
digest_id!(VariantId, "var_");
digest_id!(AttemptId, "atm_");
digest_id!(RunnerId, "run_");
digest_id!(WorkspaceId, "wsp_");
digest_id!(PrincipalId, "pri_");
digest_id!(ProviderProfileId, "prf_");
digest_id!(MutationId, "mut_");
digest_id!(MutationReservationId, "rsv_");
digest_id!(ScopeGrantId, "sgr_");
digest_id!(SourceDescriptorId, "src_");
digest_id!(ChangeId, "chg_");
digest_id!(CheckpointId, "ckp_");
digest_id!(ContentId, "cnt_");
digest_id!(CandidateId, "can_");
digest_id!(CandidateProofRoot, "cpr_");
digest_id!(IntegrationProofRoot, "ipr_");
digest_id!(GateId, "gat_");
digest_id!(EvidenceId, "evd_");
digest_id!(EffectIntentId, "efi_");
digest_id!(EffectReceiptId, "efr_");
digest_id!(EventId, "evt_");
digest_id!(CommandId, "cmd_");
digest_id!(RpcRequestId, "rpc_");
digest_id!(DogfoodIntentId, "dfi_");
digest_id!(DogfoodGrantId, "dfg_");
digest_id!(ProviderEnrollmentId, "pen_");
digest_id!(RuntimePassportId, "rtp_");
digest_id!(ProviderCredentialProjectionId, "pcp_");
digest_id!(CredentialProjectionProfileId, "cpp_");
digest_id!(RepositoryContextSnapshotId, "rcs_");
digest_id!(DogfoodBudgetReservationId, "dbr_");
digest_id!(DogfoodRunId, "dfr_");

/// Exact structured protocol admitted for each provider's first dogfood path.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DogfoodProviderProtocolV1 {
    ClaudeStreamJson,
    CodexAppServerJsonl,
    CursorAcp,
    AntigravityHeadlessStructured,
}

impl DogfoodProviderProtocolV1 {
    #[must_use]
    pub const fn required_for(provider: LaunchProvider) -> Self {
        match provider {
            LaunchProvider::Claude => Self::ClaudeStreamJson,
            LaunchProvider::Codex => Self::CodexAppServerJsonl,
            LaunchProvider::Cursor => Self::CursorAcp,
            LaunchProvider::Agy => Self::AntigravityHeadlessStructured,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClaudeStreamJson => "claude_stream_json",
            Self::CodexAppServerJsonl => "codex_app_server_jsonl",
            Self::CursorAcp => "cursor_acp",
            Self::AntigravityHeadlessStructured => "antigravity_headless_structured",
        }
    }
}

pub(crate) fn is_bounded_wire_label(value: &str, max_bytes: usize) -> bool {
    !value.is_empty()
        && value.len() <= max_bytes
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'/' | b'-')
        })
}

pub(crate) fn require_exact_wire(
    name: &str,
    value: &str,
    expected: &str,
    code: &'static str,
) -> Result<(), WireError> {
    if value != expected {
        return Err(WireError::new(code, format!("{name} must be {expected}")));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GitOid {
    Sha1(String),
    Sha256(String),
}

impl GitOid {
    pub(crate) fn parse_checked(raw: &str) -> Result<Self, WireError> {
        let (algorithm, hex) = raw.split_once(':').ok_or_else(|| {
            WireError::new(
                "INVALID_GIT_OID",
                "Git OID must be tagged sha1:<hex> or sha256:<hex>",
            )
        })?;
        match algorithm {
            "sha1" => {
                validate_lower_hex(hex, 40, "INVALID_GIT_OID")?;
                Ok(Self::Sha1(hex.to_owned()))
            }
            "sha256" => {
                validate_lower_hex(hex, 64, "INVALID_GIT_OID")?;
                Ok(Self::Sha256(hex.to_owned()))
            }
            _ => Err(WireError::new(
                "INVALID_GIT_OID",
                format!("unsupported Git object algorithm {algorithm}"),
            )),
        }
    }

    pub fn algorithm(&self) -> &'static str {
        match self {
            Self::Sha1(_) => "sha1",
            Self::Sha256(_) => "sha256",
        }
    }

    pub fn hex(&self) -> &str {
        match self {
            Self::Sha1(hex) | Self::Sha256(hex) => hex,
        }
    }
}

impl FromStr for GitOid {
    type Err = WireError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse_checked(raw)
    }
}

impl fmt::Display for GitOid {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.algorithm(), self.hex())
    }
}

impl Serialize for GitOid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for GitOid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse_checked(&raw).map_err(de::Error::custom)
    }
}
