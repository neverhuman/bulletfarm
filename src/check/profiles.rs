//! Named diagnostic gate slices. Canonical V1 release authority is unprofiled.

use std::{fs, path::Path};

use super::model::{CheckModelError, GateClass, GateResult};
use crate::coord::CoordError;

const LINUX_PREVIEW: &[&str] = &[
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

const PLATFORM: &[&str] = &[
    "release.checksums",
    "release.package-matrix",
    "release.platform-containment",
    "release.provenance",
    "release.receipt-contracts",
    "release.sbom",
    "release.signatures",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ReleaseProfile {
    LinuxPreview,
    ProviderCodex,
    ProviderCursor,
    ProviderAntigravity,
    GithubAdapterV1,
    PlatformLinuxAarch64,
    PlatformMacosX86_64,
    PlatformMacosAarch64,
    PlatformWindowsX86_64,
    TeamV1,
}

impl ReleaseProfile {
    pub(super) const NAMES: &[&str] = &[
        "linux-preview",
        "provider-codex",
        "provider-cursor",
        "provider-antigravity",
        "github-adapter-v1",
        "platform-linux-aarch64",
        "platform-macos-x86_64",
        "platform-macos-aarch64",
        "platform-windows-x86_64",
        "team-v1",
    ];

    pub(super) fn parse(value: &str) -> Result<Self, CoordError> {
        match value {
            "linux-preview" => Ok(Self::LinuxPreview),
            "provider-codex" => Ok(Self::ProviderCodex),
            "provider-cursor" => Ok(Self::ProviderCursor),
            "provider-antigravity" => Ok(Self::ProviderAntigravity),
            "github-adapter-v1" => Ok(Self::GithubAdapterV1),
            "platform-linux-aarch64" => Ok(Self::PlatformLinuxAarch64),
            "platform-macos-x86_64" => Ok(Self::PlatformMacosX86_64),
            "platform-macos-aarch64" => Ok(Self::PlatformMacosAarch64),
            "platform-windows-x86_64" => Ok(Self::PlatformWindowsX86_64),
            "team-v1" => Ok(Self::TeamV1),
            _ => Err(CoordError::new(
                "UNKNOWN_RELEASE_PROFILE",
                format!(
                    "unknown release profile {value:?}; expected one of {}",
                    Self::NAMES.join(", ")
                ),
            )),
        }
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::LinuxPreview => "linux-preview",
            Self::ProviderCodex => "provider-codex",
            Self::ProviderCursor => "provider-cursor",
            Self::ProviderAntigravity => "provider-antigravity",
            Self::GithubAdapterV1 => "github-adapter-v1",
            Self::PlatformLinuxAarch64 => "platform-linux-aarch64",
            Self::PlatformMacosX86_64 => "platform-macos-x86_64",
            Self::PlatformMacosAarch64 => "platform-macos-aarch64",
            Self::PlatformWindowsX86_64 => "platform-windows-x86_64",
            Self::TeamV1 => "team-v1",
        }
    }

    const fn gate_ids(self) -> &'static [&'static str] {
        match self {
            Self::LinuxPreview => LINUX_PREVIEW,
            Self::ProviderCodex => &["release.provider.codex", "release.receipt-contracts"],
            Self::ProviderCursor => &["release.provider.cursor", "release.receipt-contracts"],
            Self::ProviderAntigravity => {
                &["release.provider.antigravity", "release.receipt-contracts"]
            }
            Self::GithubAdapterV1 => &["release.forge.github-app", "release.receipt-contracts"],
            Self::PlatformLinuxAarch64
            | Self::PlatformMacosX86_64
            | Self::PlatformMacosAarch64
            | Self::PlatformWindowsX86_64 => PLATFORM,
            Self::TeamV1 => &["release.receipt-contracts"],
        }
    }
}

pub(super) fn select(
    profile: ReleaseProfile,
    gates: Vec<GateResult>,
    registry: &Path,
) -> Result<Vec<GateResult>, CheckModelError> {
    let ids = profile.gate_ids();
    let mut selected = gates
        .into_iter()
        .filter(|gate| ids.contains(&gate.id()))
        .collect::<Vec<_>>();
    if selected.len() != ids.len() {
        return Err(CheckModelError::new(
            "PROFILE_GATE_MISSING",
            format!(
                "{} references a gate absent from the catalog",
                profile.as_str()
            ),
        ));
    }
    replace_receipt_registry_gate(&mut selected, registry)?;
    match profile {
        ReleaseProfile::LinuxPreview => {
            replace_linux_preview_details(&mut selected)?;
            selected.extend(linux_preview_specific_gates()?);
        }
        ReleaseProfile::TeamV1 => selected.push(blocked(
            "release.team-v1",
            GateClass::Release,
            "distributed PostgreSQL/workload-mTLS team mode has no exact partition, failover, freeze, restore, or admission receipt",
            "certify team-v1 independently after canonical V1 GA; do not treat a Linux preview as release authority",
        )?),
        _ => {}
    }
    Ok(selected)
}

fn replace_linux_preview_details(gates: &mut [GateResult]) -> Result<(), CheckModelError> {
    for (id, detail, repair) in [
        (
            "release.platform-containment",
            "Ubuntu 24.04 x86_64 S1 rootless-crun and policy-required S2 Firecracker containment receipts are absent",
            "certify the Linux namespace/cgroup/seccomp/network boundary and the pinned Firecracker guest path; S2-required work must refuse until then",
        ),
        (
            "release.provenance",
            "signed build provenance for the exact Ubuntu 24.04 x86_64 package is absent",
            "produce and verify provenance bound to the exact hub tag, schema-3 lock, toolchains, Portal bundle, and Linux archive",
        ),
        (
            "release.sbom",
            "CycloneDX and SPDX bills of materials for the exact Ubuntu 24.04 x86_64 package are absent",
            "generate and semantically validate both SBOM formats against the exact Linux archive",
        ),
        (
            "release.signatures",
            "verified Ed25519 receipt and Sigstore artifact signatures for the exact Ubuntu 24.04 x86_64 package are absent",
            "sign and verify the Linux archive, checksums, SBOMs, provenance, and final non-circular manifest with admitted release keys",
        ),
    ] {
        let index = gates
            .iter()
            .position(|gate| gate.id() == id)
            .ok_or_else(|| {
                CheckModelError::new(
                    "PROFILE_GATE_MISSING",
                    format!("linux-preview is missing {id}"),
                )
            })?;
        gates[index] = blocked(id, GateClass::Release, detail, repair)?;
    }
    Ok(())
}

fn replace_receipt_registry_gate(
    gates: &mut [GateResult],
    registry: &Path,
) -> Result<(), CheckModelError> {
    let index = gates
        .iter()
        .position(|gate| gate.id() == "release.receipt-contracts")
        .ok_or_else(|| {
            CheckModelError::new(
                "PROFILE_RECEIPT_GATE_MISSING",
                "release profile omits the receipt-contracts gate",
            )
        })?;
    gates[index] = match fs::symlink_metadata(registry) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => blocked(
            "release.receipt-contracts",
            GateClass::Release,
            "the selected receipt registry is absent; zero profile gates were cleared from it",
            "provision an absolute, non-symlink registry only after its kind-specific semantic verifiers and external trust policy exist",
        )?,
        Err(error) => GateResult::fail(
            "release.receipt-contracts",
            GateClass::Release,
            format!("the selected receipt registry could not be inspected: {error}"),
            "preserve the registry and repair its operating-system admission before retrying",
        )?,
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
            GateResult::fail(
                "release.receipt-contracts",
                GateClass::Release,
                "the selected receipt registry is not a real non-symlink directory",
                "supply an absolute non-symlink directory; never redirect release evidence through a link or regular file",
            )?
        }
        Ok(_) => blocked(
            "release.receipt-contracts",
            GateClass::Release,
            "the selected registry is present, but no generic receipt can clear a gate without a kind-specific semantic verifier and admitted trust/time roots",
            "add the gate-specific semantic verifier and externally admitted signer/trusted-time policy, then register exact current-family evidence",
        )?,
    };
    Ok(())
}

fn linux_preview_specific_gates() -> Result<Vec<GateResult>, CheckModelError> {
    [
        (
            "release.evolution-v1",
            "post-V1 evolutionary study and canary evidence is absent, as expected while evolutionary_authority remains disabled for V1",
            "retain this preview-only diagnostic without treating it as a canonical GA gate; schedule the frozen external study and bounded canary only after V1",
        ),
        (
            "release.operations-v1",
            "production health/readiness/metrics, freeze, incident, audit-anchor, backup, restore, rollback, and disaster workflows lack one exact operations receipt",
            "exercise the packaged single-host operations runbook and register its signed exact-subject receipt",
        ),
        (
            "release.package-linux-x86_64",
            "the signed Ubuntu 24.04 x86_64 package with embedded Portal, services, migrations, sandbox assets, and guest image is absent",
            "build and semantically verify the exact tagged x86_64-unknown-linux-gnu package and its supply-chain subjects",
        ),
        (
            "release.systemd-v1",
            "the native systemd install, upgrade, activation, rollback, uninstall, and non-destructive retention receipt is absent",
            "run two clean Ubuntu 24.04 installs plus lifecycle and disaster drills from the signed package bytes",
        ),
    ]
    .into_iter()
    .map(|(id, detail, repair)| blocked(id, GateClass::Release, detail, repair))
    .collect()
}

fn blocked(
    id: &str,
    class: GateClass,
    detail: &str,
    repair: &str,
) -> Result<GateResult, CheckModelError> {
    GateResult::blocked(id, class, detail, repair)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::{model::GateStatus, prerequisites};

    #[test]
    fn linux_preview_is_narrow_and_not_the_v1_gate_set() {
        let registry = std::env::temp_dir().join(format!(
            "bullet-release-profile-missing-{}",
            std::process::id()
        ));
        let gates = select(
            ReleaseProfile::LinuxPreview,
            prerequisites::report_release().unwrap().gates().to_vec(),
            &registry,
        )
        .unwrap();
        let ids = gates.iter().map(GateResult::id).collect::<Vec<_>>();
        for excluded in [
            "release.forge.github-app",
            "release.provider.codex",
            "release.provider.cursor",
            "release.provider.antigravity",
            "release.package-matrix",
        ] {
            assert!(!ids.contains(&excluded), "preview gate leaked: {excluded}");
        }
        for required in [
            "release.forge.jeryu",
            "release.provider.claude",
            "release.package-linux-x86_64",
            "release.systemd-v1",
            "release.operations-v1",
            "release.evolution-v1",
        ] {
            assert!(ids.contains(&required), "profile gate missing: {required}");
        }
    }

    #[test]
    fn certification_profiles_are_independent() {
        let registry = std::env::temp_dir().join(format!(
            "bullet-release-profile-missing-{}",
            std::process::id()
        ));
        for (profile, expected) in [
            (ReleaseProfile::ProviderCodex, "release.provider.codex"),
            (ReleaseProfile::ProviderCursor, "release.provider.cursor"),
            (
                ReleaseProfile::ProviderAntigravity,
                "release.provider.antigravity",
            ),
            (ReleaseProfile::GithubAdapterV1, "release.forge.github-app"),
        ] {
            let gates = select(
                profile,
                prerequisites::report_release().unwrap().gates().to_vec(),
                &registry,
            )
            .unwrap();
            assert_eq!(gates.len(), 2);
            assert!(gates.iter().any(|gate| gate.id() == expected));
            assert!(
                gates
                    .iter()
                    .any(|gate| gate.id() == "release.receipt-contracts")
            );
        }
    }

    #[test]
    fn non_directory_registry_is_a_failure_not_evidence() {
        let registry = std::env::temp_dir().join(format!(
            "bullet-release-profile-file-{}",
            std::process::id()
        ));
        std::fs::write(&registry, b"not a registry").unwrap();
        let gates = select(
            ReleaseProfile::ProviderCodex,
            prerequisites::report_release().unwrap().gates().to_vec(),
            &registry,
        )
        .unwrap();
        let receipt_gate = gates
            .iter()
            .find(|gate| gate.id() == "release.receipt-contracts")
            .unwrap();
        assert_eq!(receipt_gate.status(), GateStatus::Fail);
        std::fs::remove_file(registry).unwrap();
    }
}
