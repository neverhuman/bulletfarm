//! Release inventory. One gate admits external semantic evidence; 27 stay static.

use std::path::Path;

use super::{
    model::{CheckModelError, CheckReport, CheckTier, GateClass, GateResult},
    profiles::{self, ReleaseProfile},
    release_evidence::{self, Evaluation},
};

#[cfg(test)]
pub(super) fn report_release() -> Result<CheckReport, CheckModelError> {
    CheckReport::new(CheckTier::Release, release_gates()?)
}

pub(super) fn report_release_with_evidence(hub: &Path) -> Result<CheckReport, CheckModelError> {
    CheckReport::new(CheckTier::Release, evaluated_release_gates(hub)?)
}

#[cfg(test)]
pub(super) fn report_release_profile(
    profile: ReleaseProfile,
    receipts: &Path,
) -> Result<CheckReport, CheckModelError> {
    report_release_profile_for_hub(Path::new("."), profile, receipts)
}

pub(super) fn report_release_profile_for_hub(
    hub: &Path,
    profile: ReleaseProfile,
    receipts: &Path,
) -> Result<CheckReport, CheckModelError> {
    // Profiled reports admit evidence only from the selected registry. The
    // legacy fixed MSRV descriptor is deliberately not consulted here.
    let gates = profiles::select(hub, profile, release_gates()?, receipts)?;
    CheckReport::for_profile(profile.as_str(), gates)
}

fn evaluated_release_gates(hub: &Path) -> Result<Vec<GateResult>, CheckModelError> {
    let mut gates = release_gates()?;
    let index = gates
        .iter()
        .position(|gate| gate.id() == "release.rust-msrv-1-95")
        .ok_or_else(|| {
            CheckModelError::new(
                "MSRV_GATE_MISSING",
                "release.rust-msrv-1-95 is absent from the release catalog",
            )
        })?;
    gates[index] = match release_evidence::evaluate(hub) {
        Evaluation::Absent => gates[index].clone(),
        Evaluation::Rejected(detail) => GateResult::fail(
            "release.rust-msrv-1-95",
            GateClass::Release,
            format!("externally supplied Rust 1.95 evidence was rejected: {detail}"),
            "preserve the evidence, repair the operator admission/policy/receipt/time subjects, and obtain a new independent attestation; never weaken semantic checks",
        )?,
        Evaluation::Verified { detail, subjects } => {
            GateResult::pass("release.rust-msrv-1-95", GateClass::Release, detail)?
                .with_subjects(subjects)?
        }
    };
    Ok(gates)
}

pub(super) fn required_blockers() -> Result<Vec<GateResult>, CheckModelError> {
    [
        (
            "required.installable-lock",
            GateClass::Release,
            "the checked-in diagnostic schema-2 lock cannot authorize installation",
            "publish signed immutable member tags and generate the authenticated schema-3 family lock",
        ),
        (
            "required.jankurai-ratchet",
            GateClass::Release,
            "the family-wide pinned Jankurai required receipt is absent",
            "run the pinned family audit with zero skips and register exact-subject score/cap/hard-finding results",
        ),
        (
            "required.packaged-browser-e2e",
            GateClass::Transaction,
            "no authenticated browser test against an embedded Portal and packaged farmd is registered",
            "run Playwright against the packaged real farmd with command/session/CSRF reconciliation",
        ),
        (
            "required.pinned-scans",
            GateClass::Release,
            "pinned secret, dependency, license, and workflow scan receipts are absent",
            "admit exact scanner versions and run every required scan against these exact subjects",
        ),
        (
            "required.recovery-faults",
            GateClass::Transaction,
            "backup/restore and crash-boundary receipts are not registered",
            "run the exact-subject SQLite/CAS/journal/generation fault and verified restore suites",
        ),
        (
            "required.transaction-proof",
            GateClass::Transaction,
            "the deterministic demo remains synthetic component evidence, not a five-plane transaction proof",
            "replace synthetic success with the signed exact Candidate, independent Evidence, reconciled effect, and truthful projection transaction",
        ),
    ]
    .into_iter()
    .map(|(id, class, detail, repair)| blocked(id, class, detail, repair))
    .collect()
}

fn release_gates() -> Result<Vec<GateResult>, CheckModelError> {
    [
        (
            "release.backup-restore",
            GateClass::Release,
            "no exact-subject backup and restore receipt is registered",
            "run the tagged backup/restore suite and register its signed release receipt",
        ),
        (
            "release.checksums",
            GateClass::Release,
            "release archive checksums are absent",
            "generate and verify checksums for every exact release archive",
        ),
        (
            "release.fault-suite",
            GateClass::Release,
            "the crash-boundary and recovery fault receipt is absent",
            "run the tagged fault suite and register its exact signed receipt",
        ),
        (
            "release.forge.github-app",
            GateClass::Live,
            "no protected GitHub App integration and reconciliation receipt is registered",
            "configure the test repository and register exact dispatch, read-back, reconciliation, and integration evidence",
        ),
        (
            "release.forge.jeryu",
            GateClass::Live,
            "no protected Jeryu integration and reconciliation receipt is registered",
            "restore operator authentication and register exact protected integration and reconciliation evidence",
        ),
        (
            "release.installable-lock",
            GateClass::Release,
            "the checked-in family lock is not an installable signed schema-3 release lock",
            "publish immutable signed member tags and generate the authenticated schema-3 family lock",
        ),
        (
            "release.installer-twice",
            GateClass::Release,
            "no two-run fresh-HOME installer receipt from tagged hub-only bytes is registered",
            "verify the signed prebuilt installer twice with exact clean member OIDs and no worktrees",
        ),
        (
            "release.jankurai-90",
            GateClass::Release,
            "Jankurai release evidence at score 90 with zero hard findings and caps is absent",
            "resolve all release findings and register the pinned Jankurai >=90 zero-hard/zero-cap receipt",
        ),
        (
            "release.manifest-non-circular",
            GateClass::Release,
            "the final non-circular signed release manifest is absent",
            "generate and sign a manifest that binds the hub tag without embedding its own digest",
        ),
        (
            "release.package-matrix",
            GateClass::Release,
            "the five required platform archives with the embedded Portal are absent",
            "build and smoke Linux x86_64/aarch64, macOS x86_64/arm64, and Windows x64 archives from the exact tagged family with the Portal embedded",
        ),
        (
            "release.package-linux-x86_64",
            GateClass::Release,
            "the signed Ubuntu 24.04 x86_64 package with embedded Portal, services, migrations, sandbox assets, and guest image is absent",
            "build and semantically verify the exact tagged x86_64-unknown-linux-gnu package and its supply-chain subjects",
        ),
        (
            "release.platform-containment",
            GateClass::Release,
            "platform containment or fail-closed refusal receipts are absent",
            "prove Linux production containment and mutation refusal on every unsupported packaged platform",
        ),
        (
            "release.provenance",
            GateClass::Release,
            "signed build provenance for the package matrix is absent",
            "produce and verify provenance bound to the exact hub tag, lock, toolchains, and archives",
        ),
        (
            "release.provider.antigravity",
            GateClass::Live,
            "Antigravity release conformance evidence is absent",
            "register the exact adapter/version/profile isolation, failure, quota, and patch conformance receipt",
        ),
        (
            "release.provider.claude",
            GateClass::Live,
            "Claude release conformance evidence is absent",
            "register the exact adapter/version/profile isolation, failure, quota, and patch conformance receipt",
        ),
        (
            "release.provider.codex",
            GateClass::Live,
            "Codex release conformance evidence is absent",
            "register the exact adapter/version/profile isolation, failure, quota, and patch conformance receipt",
        ),
        (
            "release.provider.cursor",
            GateClass::Live,
            "Cursor release conformance evidence is absent",
            "register the exact adapter/version/profile isolation, failure, quota, and patch conformance receipt",
        ),
        (
            "release.receipt-contracts",
            GateClass::Release,
            "the strict receipt verifier exists, but no independently provisioned allowed-signers policy or actual signed release receipt is registered",
            "provision the external signer policy, trusted-time observation, kind-specific semantic verifier, and exact tagged receipts without treating component verification as gate evidence",
        ),
        (
            "release.rust-msrv-1-95",
            GateClass::Release,
            "an exact release build receipt for Rust 1.95 is absent",
            "build and test the exact tagged family with the admitted Rust 1.95 MSRV toolchain",
        ),
        (
            "release.rust-pinned-1-97-1",
            GateClass::Release,
            "an exact release build receipt for pinned Rust 1.97.1 is absent",
            "build and test the exact tagged family with the admitted pinned Rust 1.97.1 toolchain",
        ),
        (
            "release.scan.dependency",
            GateClass::Release,
            "the pinned dependency scan receipt is absent",
            "run the admitted dependency scanner against exact lockfiles and register its receipt",
        ),
        (
            "release.scan.license",
            GateClass::Release,
            "the pinned license policy scan receipt is absent",
            "run the admitted license scanner against exact artifacts and register its receipt",
        ),
        (
            "release.scan.secret",
            GateClass::Release,
            "the pinned secret scan receipt is absent",
            "run the admitted secret scanner against exact tagged trees and register its receipt",
        ),
        (
            "release.scan.workflow",
            GateClass::Release,
            "the pinned workflow policy scan receipt is absent",
            "run the admitted workflow scanner against exact workflow bytes and register its receipt",
        ),
        (
            "release.sbom",
            GateClass::Release,
            "software bills of materials for the package matrix are absent",
            "generate and validate an SBOM for every exact release archive",
        ),
        (
            "release.signatures",
            GateClass::Release,
            "verified signatures for the package matrix are absent",
            "sign every archive, checksum set, SBOM, provenance statement, and final manifest with admitted release keys",
        ),
        (
            "release.transaction-demo",
            GateClass::Transaction,
            "the exact offline five-plane transaction demo receipt is absent",
            "run the non-synthetic tagged transaction demo and register its independent exact-subject receipt",
        ),
        (
            "release.systemd-v1",
            GateClass::Release,
            "the native systemd install, upgrade, activation, rollback, uninstall, and non-destructive retention receipt is absent",
            "run two clean Ubuntu 24.04 installs plus lifecycle and disaster drills from the signed package bytes",
        ),
    ]
    .into_iter()
    .map(|(id, class, detail, repair)| blocked(id, class, detail, repair))
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
    use crate::check::model::GateStatus;

    #[test]
    fn every_static_gate_is_blocked_with_repair() {
        let required = required_blockers().unwrap();
        let release = report_release().unwrap();
        assert_eq!(required.len(), 6);
        assert_eq!(release.status(), GateStatus::Blocked);
        assert_eq!(release.exit_code(), 3);
        assert!(required.iter().chain(release.gates()).all(|gate| {
            gate.status() == GateStatus::Blocked
                && gate.repair().is_some_and(|repair| !repair.is_empty())
        }));
    }

    #[test]
    fn release_inventory_is_complete_and_cannot_be_cleared_by_input() {
        let report = report_release().unwrap();
        let ids = report
            .gates()
            .iter()
            .map(GateResult::id)
            .collect::<Vec<_>>();
        for required in [
            "release.receipt-contracts",
            "release.rust-msrv-1-95",
            "release.rust-pinned-1-97-1",
            "release.installable-lock",
            "release.transaction-demo",
            "release.jankurai-90",
            "release.package-matrix",
            "release.package-linux-x86_64",
            "release.installer-twice",
            "release.sbom",
            "release.checksums",
            "release.signatures",
            "release.provenance",
            "release.manifest-non-circular",
            "release.backup-restore",
            "release.fault-suite",
            "release.forge.jeryu",
            "release.forge.github-app",
            "release.provider.claude",
            "release.provider.codex",
            "release.provider.cursor",
            "release.provider.antigravity",
            "release.platform-containment",
            "release.scan.dependency",
            "release.scan.license",
            "release.scan.secret",
            "release.scan.workflow",
            "release.systemd-v1",
        ] {
            assert!(ids.contains(&required), "missing {required}");
        }
        assert_eq!(ids.len(), 28);
    }
}
