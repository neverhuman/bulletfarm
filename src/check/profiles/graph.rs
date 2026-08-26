use crate::coord::CoordError;

const ALL_LEGACY_GATES: &[&str] = &[
    "release.backup-restore",
    "release.checksums",
    "release.fault-suite",
    "release.forge.github-app",
    "release.forge.jeryu",
    "release.installable-lock",
    "release.installer-twice",
    "release.jankurai-90",
    "release.manifest-non-circular",
    "release.package-matrix",
    "release.platform-containment",
    "release.provenance",
    "release.provider.antigravity",
    "release.provider.claude",
    "release.provider.codex",
    "release.provider.cursor",
    "release.receipt-contracts",
    "release.rust-msrv-1-95",
    "release.rust-pinned-1-97-1",
    "release.scan.dependency",
    "release.scan.license",
    "release.scan.secret",
    "release.scan.workflow",
    "release.sbom",
    "release.signatures",
    "release.transaction-demo",
];

const LINUX_PREVIEW_GATES: &[&str] = &[
    "release.backup-restore",
    "release.checksums",
    "release.fault-suite",
    "release.forge.jeryu",
    "release.installable-lock",
    "release.installer-twice",
    "release.jankurai-90",
    "release.manifest-non-circular",
    "release.platform-containment",
    "release.provenance",
    "release.provider.claude",
    "release.receipt-contracts",
    "release.rust-msrv-1-95",
    "release.rust-pinned-1-97-1",
    "release.scan.dependency",
    "release.scan.license",
    "release.scan.secret",
    "release.scan.workflow",
    "release.sbom",
    "release.signatures",
    "release.transaction-demo",
];

const SELF_HOSTED_DIRECT: &[&str] = &[
    "release.backup-restore",
    "release.fault-suite",
    "release.installable-lock",
    "release.installer-twice",
    "release.jankurai-90",
    "release.manifest-non-circular",
    "release.rust-msrv-1-95",
    "release.rust-pinned-1-97-1",
    "release.scan.dependency",
    "release.scan.license",
    "release.scan.secret",
    "release.scan.workflow",
    "release.transaction-demo",
];

const PLATFORM_GATES: &[&str] = &[
    "release.checksums",
    "release.platform-containment",
    "release.provenance",
    "release.receipt-contracts",
    "release.sbom",
    "release.signatures",
];

const PLATFORM_LINUX_X86_64_GATES: &[&str] = &[
    "release.checksums",
    "release.package-linux-x86_64",
    "release.platform-containment",
    "release.provenance",
    "release.receipt-contracts",
    "release.sbom",
    "release.signatures",
    "release.systemd-v1",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(in crate::check) enum ReleaseProfile {
    SelfHostedV1,
    EvolutionV1,
    ProviderClaude,
    ProviderCodex,
    ProviderCursor,
    ProviderAntigravity,
    JeryuForgeV1,
    GithubAdapterV1,
    GitlabAdapterV1,
    GitlabSelfManagedV1,
    PlatformLinuxX86_64,
    PlatformLinuxAarch64,
    PlatformMacosX86_64,
    PlatformMacosAarch64,
    PlatformWindowsX86_64,
    UniversalV1,
    TeamV1,
    SagaV1,
    LegacyV1_26,
    LinuxPreview,
}

impl ReleaseProfile {
    pub(in crate::check) const NAMES: &[&str] = &[
        "self-hosted-v1",
        "evolution-v1",
        "provider-claude",
        "provider-codex",
        "provider-cursor",
        "provider-antigravity",
        "jeryu-forge-v1",
        "github-adapter-v1",
        "gitlab-adapter-v1",
        "gitlab-self-managed-v1",
        "platform-linux-x86_64",
        "platform-linux-aarch64",
        "platform-macos-x86_64",
        "platform-macos-aarch64",
        "platform-windows-x86_64",
        "universal-v1",
        "team-v1",
        "saga-v1",
        "legacy-v1-26",
        "linux-preview",
    ];

    const ALL: [Self; 20] = [
        Self::SelfHostedV1,
        Self::EvolutionV1,
        Self::ProviderClaude,
        Self::ProviderCodex,
        Self::ProviderCursor,
        Self::ProviderAntigravity,
        Self::JeryuForgeV1,
        Self::GithubAdapterV1,
        Self::GitlabAdapterV1,
        Self::GitlabSelfManagedV1,
        Self::PlatformLinuxX86_64,
        Self::PlatformLinuxAarch64,
        Self::PlatformMacosX86_64,
        Self::PlatformMacosAarch64,
        Self::PlatformWindowsX86_64,
        Self::UniversalV1,
        Self::TeamV1,
        Self::SagaV1,
        Self::LegacyV1_26,
        Self::LinuxPreview,
    ];

    pub(in crate::check) fn parse(value: &str) -> Result<Self, CoordError> {
        Self::NAMES
            .iter()
            .position(|name| *name == value)
            .map(|index| Self::ALL[index])
            .ok_or_else(|| {
                CoordError::new(
                    "UNKNOWN_RELEASE_PROFILE",
                    format!(
                        "unknown release profile {value:?}; expected one of {}",
                        Self::NAMES.join(", ")
                    ),
                )
            })
    }

    pub(in crate::check) const fn as_str(self) -> &'static str {
        Self::NAMES[self as usize]
    }

    pub(super) const fn dependencies(self) -> &'static [Self] {
        match self {
            Self::SelfHostedV1 => &[
                Self::ProviderClaude,
                Self::JeryuForgeV1,
                Self::PlatformLinuxX86_64,
            ],
            Self::EvolutionV1 => &[Self::SelfHostedV1],
            Self::UniversalV1 => &[
                Self::SelfHostedV1,
                Self::ProviderCodex,
                Self::ProviderCursor,
                Self::ProviderAntigravity,
                Self::GithubAdapterV1,
                Self::GitlabAdapterV1,
                Self::GitlabSelfManagedV1,
                Self::PlatformLinuxAarch64,
                Self::PlatformMacosX86_64,
                Self::PlatformMacosAarch64,
                Self::PlatformWindowsX86_64,
            ],
            Self::TeamV1 => &[Self::SelfHostedV1],
            Self::SagaV1 => &[Self::TeamV1],
            _ => &[],
        }
    }

    pub(super) const fn catalog_gate_ids(self) -> &'static [&'static str] {
        match self {
            Self::LegacyV1_26 => ALL_LEGACY_GATES,
            Self::LinuxPreview => LINUX_PREVIEW_GATES,
            Self::SelfHostedV1 => SELF_HOSTED_DIRECT,
            Self::ProviderClaude => &["release.provider.claude", "release.receipt-contracts"],
            Self::ProviderCodex => &["release.provider.codex", "release.receipt-contracts"],
            Self::ProviderCursor => &["release.provider.cursor", "release.receipt-contracts"],
            Self::ProviderAntigravity => {
                &["release.provider.antigravity", "release.receipt-contracts"]
            }
            Self::JeryuForgeV1 => &["release.forge.jeryu", "release.receipt-contracts"],
            Self::GithubAdapterV1 => &["release.forge.github-app", "release.receipt-contracts"],
            Self::GitlabAdapterV1 | Self::GitlabSelfManagedV1 => &["release.receipt-contracts"],
            Self::PlatformLinuxX86_64 => PLATFORM_LINUX_X86_64_GATES,
            Self::PlatformLinuxAarch64
            | Self::PlatformMacosX86_64
            | Self::PlatformMacosAarch64
            | Self::PlatformWindowsX86_64 => PLATFORM_GATES,
            Self::UniversalV1 => &["release.package-matrix"],
            Self::EvolutionV1 | Self::TeamV1 | Self::SagaV1 => &[],
        }
    }

    pub(super) const fn has_condition_gate(self) -> bool {
        !matches!(self, Self::LegacyV1_26 | Self::LinuxPreview)
    }

    pub(super) const fn condition_gate_id(self) -> &'static str {
        match self {
            Self::SelfHostedV1 => "release.profile.self-hosted-v1",
            Self::EvolutionV1 => "release.profile.evolution-v1",
            Self::ProviderClaude => "release.profile.provider-claude",
            Self::ProviderCodex => "release.profile.provider-codex",
            Self::ProviderCursor => "release.profile.provider-cursor",
            Self::ProviderAntigravity => "release.profile.provider-antigravity",
            Self::JeryuForgeV1 => "release.profile.jeryu-forge-v1",
            Self::GithubAdapterV1 => "release.profile.github-adapter-v1",
            Self::GitlabAdapterV1 => "release.profile.gitlab-adapter-v1",
            Self::GitlabSelfManagedV1 => "release.profile.gitlab-self-managed-v1",
            Self::PlatformLinuxX86_64 => "release.profile.platform-linux-x86_64",
            Self::PlatformLinuxAarch64 => "release.profile.platform-linux-aarch64",
            Self::PlatformMacosX86_64 => "release.profile.platform-macos-x86_64",
            Self::PlatformMacosAarch64 => "release.profile.platform-macos-aarch64",
            Self::PlatformWindowsX86_64 => "release.profile.platform-windows-x86_64",
            Self::UniversalV1 => "release.profile.universal-v1",
            Self::TeamV1 => "release.profile.team-v1",
            Self::SagaV1 => "release.profile.saga-v1",
            Self::LegacyV1_26 => "release.profile.legacy-v1-26",
            Self::LinuxPreview => "release.profile.linux-preview",
        }
    }

    pub(in crate::check) const fn required_closure(self) -> &'static str {
        match self {
            Self::SelfHostedV1 => {
                "Ubuntu 24.04 x86_64/systemd, a signed schema-3 family, pinned local Jeryu, Claude service identity, offline and live five-authority transactions, every selected product surface durable or typed OUT_OF_PROFILE, operations, installer, security, and supply-chain closure"
            }
            Self::EvolutionV1 => {
                "self-hosted-v1 plus all fifteen product surfaces durable, frozen T0-T5 recipes, external T0-versus-T3 confirmation, MOME/ASHA, shadow routing, R0/R1 canary, and rollback"
            }
            Self::ProviderClaude => {
                "one exact Claude provider version, profile, service identity, isolation, failure, quota, and patch certification"
            }
            Self::ProviderCodex => {
                "one exact Codex provider version, profile, service identity, isolation, failure, quota, and patch certification"
            }
            Self::ProviderCursor => {
                "one exact Cursor provider version, profile, service identity, isolation, failure, quota, and patch certification"
            }
            Self::ProviderAntigravity => {
                "one exact Antigravity provider version, profile, service identity, isolation, failure, quota, and patch certification"
            }
            Self::JeryuForgeV1 => {
                "pinned local Jeryu capabilities, protected integration, reconciliation, backup/restore, and drift refusal"
            }
            Self::GithubAdapterV1 => {
                "GitHub App delivery, exact-SHA check, protected integration, observation, and reconciliation"
            }
            Self::GitlabAdapterV1 => {
                "GitLab.com REST v4 delivery, exact-SHA status, protected merge-request integration, observation, and reconciliation"
            }
            Self::GitlabSelfManagedV1 => {
                "one exact self-managed GitLab endpoint/version and its independently certified capabilities, credentials, protection, integration, observation, and drift"
            }
            Self::PlatformLinuxX86_64 => {
                "the Linux x86_64 package and its independently certified containment"
            }
            Self::PlatformLinuxAarch64 => {
                "the Linux arm64 package and its independently certified OCI/Firecracker containment"
            }
            Self::PlatformMacosX86_64 => {
                "the macOS x86_64 package and mutation refusal until an exact containment profile passes"
            }
            Self::PlatformMacosAarch64 => {
                "the macOS arm64 package and mutation refusal until an exact containment profile passes"
            }
            Self::PlatformWindowsX86_64 => {
                "the Windows x64 package and mutation refusal until an exact containment profile passes"
            }
            Self::UniversalV1 => {
                "every provider, forge, and five-platform profile without implicitly admitting evolution-v1"
            }
            Self::TeamV1 => {
                "self-hosted-v1 plus PostgreSQL, remote runners, mTLS/SPIFFE identities, replicated projections, object storage, and partition/failover closure"
            }
            Self::SagaV1 => {
                "team-v1 plus cross-repository staged integration, honest partial states, compensation, and forward repair"
            }
            Self::LegacyV1_26 => "the non-authoritative historical 26-gate diagnostic",
            Self::LinuxPreview => "the non-authoritative Linux preview diagnostic",
        }
    }
}
