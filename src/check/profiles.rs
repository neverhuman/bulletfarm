//! Explicit, composable release profiles. Every profile remains fail-closed.

#[path = "profiles/graph.rs"]
mod graph;

use std::{collections::BTreeSet, path::Path};

use bullet_wire::v1alpha1::ReleaseReceiptKindV1;

use super::{
    model::{CheckModelError, GateClass, GateResult},
    semantic_registry::{self, Evaluation, RequestedProfile},
};

pub(super) use graph::ReleaseProfile;

pub(super) fn select(
    profile: ReleaseProfile,
    gates: Vec<GateResult>,
    registry: &Path,
) -> Result<Vec<GateResult>, CheckModelError> {
    let closure = dependency_closure(profile);
    let requested = closure
        .iter()
        .flat_map(|item| item.catalog_gate_ids().iter().copied())
        .collect::<BTreeSet<_>>();
    let mut found = BTreeSet::new();
    let mut selected = Vec::new();
    for gate in gates {
        if requested.contains(gate.id()) {
            found.insert(gate.id().to_owned());
            selected.push(gate);
        }
    }
    let missing = requested
        .iter()
        .filter(|id| !found.contains(**id))
        .copied()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(CheckModelError::new(
            "PROFILE_GATE_MISSING",
            format!(
                "{} references catalog gates that are absent: {}",
                profile.as_str(),
                missing.join(", ")
            ),
        ));
    }

    if profile != ReleaseProfile::LegacyV1_26 {
        let requested_profiles = closure
            .iter()
            .map(|item| requested_profile(*item))
            .collect::<Result<Vec<_>, _>>()?;
        replace_receipt_registry_gate(
            &mut selected,
            semantic_registry::evaluate(registry, profile.as_str(), &requested_profiles),
        )?;
    }
    for item in closure {
        if item.has_condition_gate() {
            selected.push(profile_condition_gate(item)?);
        }
    }
    if profile == ReleaseProfile::LinuxPreview {
        replace_linux_preview_details(&mut selected)?;
        selected.extend(linux_preview_specific_gates()?);
    }
    Ok(selected)
}

fn requested_profile(profile: ReleaseProfile) -> Result<RequestedProfile, CheckModelError> {
    Ok(RequestedProfile::new(
        profile.as_str(),
        profile
            .dependencies()
            .iter()
            .map(|dependency| dependency.as_str())
            .collect(),
        requested_gate_bindings(profile)?,
    ))
}

fn requested_gate_bindings(
    profile: ReleaseProfile,
) -> Result<Vec<(&'static str, ReleaseReceiptKindV1)>, CheckModelError> {
    let mut gates = profile
        .catalog_gate_ids()
        .iter()
        .copied()
        .filter(|gate| *gate != "release.receipt-contracts")
        .map(|gate| expected_receipt_kind(gate).map(|kind| (gate, kind)))
        .collect::<Result<Vec<_>, _>>()?;
    if profile.has_condition_gate() {
        gates.push((
            profile.condition_gate_id(),
            ReleaseReceiptKindV1::ProfileClosure,
        ));
    }
    Ok(gates)
}

fn expected_receipt_kind(gate: &str) -> Result<ReleaseReceiptKindV1, CheckModelError> {
    let kind = match gate {
        "release.backup-restore" | "release.installer-twice" => ReleaseReceiptKindV1::Operations,
        "release.fault-suite" | "release.transaction-demo" => ReleaseReceiptKindV1::Transaction,
        "release.forge.github-app" | "release.forge.jeryu" => ReleaseReceiptKindV1::Forge,
        "release.platform-containment" => ReleaseReceiptKindV1::Containment,
        "release.provider.antigravity"
        | "release.provider.claude"
        | "release.provider.codex"
        | "release.provider.cursor" => ReleaseReceiptKindV1::Provider,
        "release.rust-msrv-1-95" | "release.rust-pinned-1-97-1" => {
            ReleaseReceiptKindV1::RustToolchain
        }
        "release.jankurai-90"
        | "release.scan.dependency"
        | "release.scan.license"
        | "release.scan.secret"
        | "release.scan.workflow" => ReleaseReceiptKindV1::Scanner,
        "release.checksums"
        | "release.installable-lock"
        | "release.manifest-non-circular"
        | "release.package-linux-x86_64"
        | "release.package-matrix"
        | "release.provenance"
        | "release.sbom"
        | "release.signatures" => ReleaseReceiptKindV1::Artifact,
        "release.systemd-v1" => ReleaseReceiptKindV1::Operations,
        _ => {
            return Err(CheckModelError::new(
                "PROFILE_RECEIPT_KIND_MISSING",
                format!("release gate {gate} has no immutable receipt-kind binding"),
            ));
        }
    };
    Ok(kind)
}

fn dependency_closure(profile: ReleaseProfile) -> Vec<ReleaseProfile> {
    fn visit(
        profile: ReleaseProfile,
        visited: &mut BTreeSet<ReleaseProfile>,
        ordered: &mut Vec<ReleaseProfile>,
    ) {
        for dependency in profile.dependencies() {
            visit(*dependency, visited, ordered);
        }
        if visited.insert(profile) {
            ordered.push(profile);
        }
    }

    let mut visited = BTreeSet::new();
    let mut ordered = Vec::new();
    visit(profile, &mut visited, &mut ordered);
    ordered
}

fn profile_condition_gate(profile: ReleaseProfile) -> Result<GateResult, CheckModelError> {
    blocked(
        profile.condition_gate_id(),
        GateClass::Release,
        &format!(
            "{} has no current kind-specific semantic receipt proving {}",
            profile.as_str(),
            profile.required_closure()
        ),
        "register exact current-family evidence only after signer lifecycle, dependency closure, schema-3 family, policy/toolchain/environment fingerprints, trusted time, replay state, and the profile-specific semantic verifier all pass",
    )
}

fn replace_receipt_registry_gate(
    gates: &mut [GateResult],
    evaluation: Evaluation,
) -> Result<(), CheckModelError> {
    let index = gates
        .iter()
        .position(|gate| gate.id() == "release.receipt-contracts")
        .ok_or_else(|| {
            CheckModelError::new(
                "PROFILE_RECEIPT_GATE_MISSING",
                "release profile closure omits the receipt-contracts gate",
            )
        })?;
    gates[index] = match evaluation {
        Evaluation::Absent => blocked(
            "release.receipt-contracts",
            GateClass::Release,
            "the selected receipt registry is absent; zero profile gates were cleared from it",
            "provision an absolute, non-symlink registry only after its kind-specific semantic verifiers and external trust policy exist",
        )?,
        Evaluation::Rejected(detail) => GateResult::fail(
            "release.receipt-contracts",
            GateClass::Release,
            format!("the selected receipt registry was structurally rejected: {detail}"),
            "preserve the registry bytes, repair descriptor admission, canonical wire records, digests, profile closure, and exact structural bindings, then retry without weakening the checks",
        )?,
        Evaluation::StructurallyValidButUntrusted { selected_bindings } => blocked(
            "release.receipt-contracts",
            GateClass::Release,
            &format!(
                "the selected registry has {selected_bindings} structurally bound gate/profile bindings, but self-selected signer policy and detached signatures are untrusted and external trust-root, trusted-time, replay/high-water, exact-family, and kind-specific semantic verification are absent"
            ),
            "admit signer policy from the schema-3 family lock, verify role-separated signatures and trusted time, enforce replay/high-water state and exact-family binding, then run each gate's kind-specific semantic verifier; structural JSON alone never clears a gate",
        )?,
    };
    Ok(())
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

fn linux_preview_specific_gates() -> Result<Vec<GateResult>, CheckModelError> {
    [
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

    #[test]
    fn requested_gate_bindings_include_only_declared_profile_conditions() {
        let preview = requested_gate_bindings(ReleaseProfile::LinuxPreview).unwrap();
        assert!(
            !preview
                .iter()
                .any(|(gate, _)| gate.starts_with("release.profile."))
        );

        let provider = requested_gate_bindings(ReleaseProfile::ProviderCodex).unwrap();
        assert!(provider.contains(&(
            "release.profile.provider-codex",
            ReleaseReceiptKindV1::ProfileClosure,
        )));
        assert!(provider.contains(&("release.provider.codex", ReleaseReceiptKindV1::Provider,)));
    }
}
