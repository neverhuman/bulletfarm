use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::Options;
use crate::coord::recovery_manifest::{
    RecoveryInspectionCommand, RecoveryProvenanceCommand,
    authoring::{self, ObservedRecoveryAuthorizationDraftInput},
    bootstrap_build::seal_bootstrap_build_observation,
};
use crate::coord::{
    CoordError, CoordStore, RecoveryAuthorizationSignatureV1, RecoveryAuthorizationV1,
    RecoveryBootstrapProvenanceV1, RecoveryCommand, RecoveryInspectionV1, RecoveryProductionPlanV1,
    RecoveryProofRequestV1, RecoveryReceiptAdoptionRequestV1, RecoveryReviewApprovalV1,
    RecoveryReviewRequestV1,
};

const RAW_ED25519_SIGNATURE_BYTES: u64 = 64;

pub(in crate::cli) fn build_observe(options: &Options) -> Result<String, CoordError> {
    require_platform()?;
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "provenance",
        "source-archive",
        "builder-contract",
        "toolchain-contract",
        "command-contract",
        "cache-manifest",
        "cache-archive",
        "executable-run-1",
        "executable-run-2",
        "output",
    ])?;
    let provenance = options.one("provenance")?;
    let source_archive = options.one("source-archive")?;
    let builder_contract = options.one("builder-contract")?;
    let toolchain_contract = options.one("toolchain-contract")?;
    let command_contract = options.one("command-contract")?;
    let cache_manifest = options.one("cache-manifest")?;
    let cache_archive = options.one("cache-archive")?;
    let executable_run_1 = options.one("executable-run-1")?;
    let executable_run_2 = options.one("executable-run-2")?;
    let output = options.one("output")?;
    let output = normalized_absolute(&output)?;
    let observation_id = seal_bootstrap_build_observation([
        normalized_absolute(&provenance)?,
        normalized_absolute(&source_archive)?,
        normalized_absolute(&builder_contract)?,
        normalized_absolute(&toolchain_contract)?,
        normalized_absolute(&command_contract)?,
        normalized_absolute(&cache_manifest)?,
        normalized_absolute(&cache_archive)?,
        normalized_absolute(&executable_run_1)?,
        normalized_absolute(&executable_run_2)?,
        output.clone(),
    ])?;
    render_written(
        "bullet.coord.recovery-bootstrap-build-observation.v1",
        &output,
        &observation_id,
    )
}

pub(super) fn inspect(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    require_platform()?;
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "interrupted-capture",
        "tainted-generation",
        "frozen-live-source",
        "output",
    ])?;
    let command = inspection_command(options)?;
    let output = normalized_absolute(&options.one("output")?)?;
    let inspection = crate::coord::recovery_manifest::inspect(store.root(), &command)?;
    crate::coord::sealed::write(&output, &inspection)?;
    render_written(
        "bullet.coord.recovery-inspection.v1",
        &output,
        &inspection.inspection_id,
    )
}

pub(super) fn provenance(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    require_platform()?;
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "bootstrap-commit",
        "cargo-bin",
        "rustc-bin",
        "source-archive-output",
        "output",
    ])?;
    let bootstrap_commit_oid = options.one("bootstrap-commit")?;
    crate::coord::validate_commit_oid(&bootstrap_commit_oid)?;
    let cargo_bin = normalized_absolute(&options.one("cargo-bin")?)?;
    let rustc_bin = normalized_absolute(&options.one("rustc-bin")?)?;
    let output = normalized_absolute(&options.one("output")?)?;
    let provenance = crate::coord::recovery_manifest::produce_provenance(
        store.root(),
        &RecoveryProvenanceCommand {
            bootstrap_commit_oid,
            cargo_bin,
            rustc_bin,
            source_archive_output: normalized_absolute(&options.one("source-archive-output")?)?,
            output: output.clone(),
        },
    )?;
    render_written(
        "bullet.coord.recovery-bootstrap-provenance.v1",
        &output,
        &provenance.bootstrap_commit_oid,
    )
}

pub(super) fn authorization_draft(options: &Options) -> Result<String, CoordError> {
    require_platform()?;
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "inspection",
        "bootstrap-provenance",
        "decision",
        "recovery-operator",
        "recovery-operator-uid",
        "reviewer-principal",
        "reviewer-fingerprint",
        "policy-namespace",
        "validity-window-ms",
        "output",
    ])?;
    let inspection = crate::coord::sealed::read::<RecoveryInspectionV1>(&normalized_absolute(
        &options.one("inspection")?,
    )?)?;
    let provenance = crate::coord::sealed::read::<RecoveryBootstrapProvenanceV1>(
        &normalized_absolute(&options.one("bootstrap-provenance")?)?,
    )?;
    let decision = match options.one("decision")?.as_str() {
        "APPROVE" => bullet_wire::decode_canonical(b"\"APPROVE\"")
            .map_err(|error| invalid(format!("cannot decode APPROVE decision: {error}")))?,
        _ => return Err(invalid("recovery authorization decision must be APPROVE")),
    };
    let authorization = authoring::draft_observed(
        &inspection,
        &provenance,
        ObservedRecoveryAuthorizationDraftInput {
            decision,
            recovery_operator: options.one("recovery-operator")?,
            recovery_operator_uid: required_u32(options, "recovery-operator-uid")?,
            reviewer_principal: options.one("reviewer-principal")?,
            reviewer_fingerprint: options.one("reviewer-fingerprint")?,
            policy_namespace: options.one("policy-namespace")?,
            validity_window_ms: required_u64(options, "validity-window-ms")?,
        },
    )?;
    let output = normalized_absolute(&options.one("output")?)?;
    authoring::require_observed_current(&authorization)?;
    crate::coord::sealed::write(&output, &authorization)?;
    render_written(
        "bullet.coord.recovery-authorization.v1",
        &output,
        &authorization.inspection_id,
    )
}

pub(super) fn authorization_message(options: &Options) -> Result<String, CoordError> {
    require_platform()?;
    options.reject_flags()?;
    options.reject_unknown_values(&["authorization", "output"])?;
    let authorization = crate::coord::sealed::read::<RecoveryAuthorizationV1>(
        &normalized_absolute(&options.one("authorization")?)?,
    )?;
    let message = authoring::signing_message(&authorization)?;
    let output = normalized_absolute(&options.one("output")?)?;
    let bound = u64::try_from(message.len())
        .map_err(|_| invalid("signing message length cannot be represented"))?;
    crate::coord::sealed::write_raw(&output, &message, bound)?;
    render_written(
        "bullet.coord.recovery-authorization-signing-message.v1",
        &output,
        &authorization.inspection_id,
    )
}

pub(super) fn authorization_signature_import(options: &Options) -> Result<String, CoordError> {
    require_platform()?;
    options.reject_flags()?;
    options.reject_unknown_values(&["authorization", "signature", "output"])?;
    let authorization = crate::coord::sealed::read::<RecoveryAuthorizationV1>(
        &normalized_absolute(&options.one("authorization")?)?,
    )?;
    let raw_signature = crate::coord::sealed::read_raw(
        &normalized_absolute(&options.one("signature")?)?,
        RAW_ED25519_SIGNATURE_BYTES,
    )?;
    let signature = authoring::import_signature(&authorization, &raw_signature)?;
    let output = normalized_absolute(&options.one("output")?)?;
    crate::coord::sealed::write(&output, &signature)?;
    render_written(
        "bullet.coord.recovery-authorization-signature.v1",
        &output,
        &signature.authorization_sha256,
    )
}

fn required_u32(options: &Options, name: &str) -> Result<u32, CoordError> {
    let number = required_u64(options, name)?;
    u32::try_from(number)
        .map_err(|_| CoordError::new("INVALID_OPTION", format!("--{name} exceeds u32")))
}

fn required_u64(options: &Options, name: &str) -> Result<u64, CoordError> {
    let value = options.one(name)?;
    super::parse_ascii_u64(&value)
        .ok_or_else(|| CoordError::new("INVALID_OPTION", format!("--{name} has an invalid value")))
}

pub(super) fn manifest(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    require_platform()?;
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "inspection",
        "authorization",
        "authorization-signature",
        "bootstrap-provenance",
        "interrupted-capture",
        "tainted-generation",
        "frozen-live-source",
        "output",
    ])?;
    let command = inspection_command(options)?;
    let inspection = crate::coord::sealed::read::<RecoveryInspectionV1>(&normalized_absolute(
        &options.one("inspection")?,
    )?)?;
    let authorization = crate::coord::sealed::read::<RecoveryAuthorizationV1>(
        &normalized_absolute(&options.one("authorization")?)?,
    )?;
    let signature = crate::coord::sealed::read::<RecoveryAuthorizationSignatureV1>(
        &normalized_absolute(&options.one("authorization-signature")?)?,
    )?;
    let provenance = crate::coord::sealed::read::<RecoveryBootstrapProvenanceV1>(
        &normalized_absolute(&options.one("bootstrap-provenance")?)?,
    )?;
    let output = normalized_absolute(&options.one("output")?)?;
    let manifest = crate::coord::recovery_manifest::manifest(
        store.root(),
        &command,
        &inspection,
        &authorization,
        &signature,
        &provenance,
    )?;
    crate::coord::sealed::write(&output, &manifest)?;
    render_written(
        "bullet.coord.generation-manifest.v2",
        &output,
        manifest.generation_id().as_str(),
    )
}

fn inspection_command(options: &Options) -> Result<RecoveryInspectionCommand, CoordError> {
    Ok(RecoveryInspectionCommand {
        interrupted_capture: normalized_absolute(&options.one("interrupted-capture")?)?,
        tainted_generation: normalized_absolute(&options.one("tainted-generation")?)?,
        frozen_live_source: normalized_absolute(&options.one("frozen-live-source")?)?,
    })
}

pub(super) fn recover_rollover(
    store: &CoordStore,
    options: &Options,
) -> Result<String, CoordError> {
    options.reject_flags()?;
    options.reject_unknown_values(&[
        "manifest",
        "inspection",
        "authorization",
        "authorization-signature",
        "bootstrap-provenance",
        "interrupted-capture",
        "tainted-generation",
        "frozen-live-source",
    ])?;
    let command = RecoveryCommand::new(
        normalized_absolute(&options.one("manifest")?)?,
        normalized_absolute(&options.one("inspection")?)?,
        normalized_absolute(&options.one("authorization")?)?,
        normalized_absolute(&options.one("authorization-signature")?)?,
        normalized_absolute(&options.one("bootstrap-provenance")?)?,
        normalized_absolute(&options.one("interrupted-capture")?)?,
        normalized_absolute(&options.one("tainted-generation")?)?,
        normalized_absolute(&options.one("frozen-live-source")?)?,
    );
    let execution = store.recover_rollover(&command)?;
    serde_json::to_string_pretty(&execution).map_err(CoordError::json)
}

pub(super) fn plan(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    require_platform()?;
    options.reject_flags()?;
    options.reject_unknown_values(&["output"])?;
    let output = normalized_absolute(&options.one("output")?)?;
    let plan = store.derive_recovery_plan()?;
    crate::coord::sealed::write(&output, &plan)?;
    render_written("recovery_production_plan_v1", &output, &plan.plan_id)
}

pub(super) fn proof(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    require_platform()?;
    options.reject_flags()?;
    options.reject_unknown_values(&["plan"])?;
    let plan = crate::coord::sealed::read::<RecoveryProductionPlanV1>(&normalized_absolute(
        &options.one("plan")?,
    )?)?;
    let applied = store.record_recovery_proof(&RecoveryProofRequestV1::for_plan(plan)?)?;
    serde_json::to_string_pretty(&applied).map_err(CoordError::json)
}

pub(super) fn review(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    require_platform()?;
    options.reject_flags()?;
    options.reject_unknown_values(&["plan", "approval"])?;
    let request = review_request(options)?;
    let applied = store.record_recovery_review(&request)?;
    serde_json::to_string_pretty(&applied).map_err(CoordError::json)
}

pub(super) fn request(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    require_platform()?;
    options.reject_flags()?;
    options.reject_unknown_values(&["plan", "approval", "output"])?;
    let output = normalized_absolute(&options.one("output")?)?;
    let review = review_request(options)?;
    let request = store.build_recovery_adoption_request(&review)?;
    crate::coord::sealed::write(&output, &request)?;
    render_written(
        "recovery_receipt_adoption_request_v1",
        &output,
        request.request_id.as_str(),
    )
}

pub(super) fn adopt(store: &CoordStore, options: &Options) -> Result<String, CoordError> {
    require_platform()?;
    options.reject_flags()?;
    options.reject_unknown_values(&["request"])?;
    let request = crate::coord::sealed::read::<RecoveryReceiptAdoptionRequestV1>(
        &normalized_absolute(&options.one("request")?)?,
    )?;
    let applied = store.adopt_recovery_receipts(&request)?;
    serde_json::to_string_pretty(&applied).map_err(CoordError::json)
}

fn review_request(options: &Options) -> Result<RecoveryReviewRequestV1, CoordError> {
    let plan_path = normalized_absolute(&options.one("plan")?)?;
    let approval_path = normalized_absolute(&options.one("approval")?)?;
    let plan = crate::coord::sealed::read::<RecoveryProductionPlanV1>(&plan_path)?;
    let approval = crate::coord::sealed::read::<RecoveryReviewApprovalV1>(&approval_path)?;
    let approval_bytes = bullet_wire::canonical_json(&approval)
        .map_err(|error| invalid(format!("cannot canonicalize review approval: {error}")))?;
    RecoveryReviewRequestV1::from_approval(
        plan,
        approval,
        format!("sha256:{:x}", Sha256::digest(approval_bytes)),
    )
}

fn normalized_absolute(value: &str) -> Result<PathBuf, CoordError> {
    let path = PathBuf::from(value);
    if !crate::coord::recovery_manifest::is_normalized_absolute(&path) {
        return Err(invalid(
            "recovery document path must be normalized absolute lexical bytes",
        ));
    }
    Ok(path)
}

fn render_written(kind: &str, path: &Path, id: &str) -> Result<String, CoordError> {
    serde_json::to_string_pretty(&serde_json::json!({
        "kind": kind,
        "path": path,
        "id": id,
    }))
    .map_err(CoordError::json)
}

#[cfg(target_os = "linux")]
fn require_platform() -> Result<(), CoordError> {
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn require_platform() -> Result<(), CoordError> {
    Err(CoordError::new(
        "COORD_RECOVERY_PLATFORM_UNSUPPORTED",
        "recovery production is unavailable until this platform has an exact native proof",
    ))
}

fn invalid(reason: impl Into<String>) -> CoordError {
    CoordError::new("INVALID_RECOVERY_PRODUCTION", reason)
}

#[cfg(test)]
mod tests {
    use crate::cli::test_recovery_action;

    fn complete_args() -> Vec<String> {
        [
            "provenance",
            "source-archive",
            "builder-contract",
            "toolchain-contract",
            "command-contract",
            "cache-manifest",
            "cache-archive",
            "executable-run-1",
            "executable-run-2",
            "output",
        ]
        .into_iter()
        .enumerate()
        .flat_map(|(index, name)| [format!("--{name}"), format!("/tmp/subject-{index}")])
        .collect()
    }

    #[test]
    fn recovery_build_observe_routes_without_coordinator_state() {
        let error = test_recovery_action("recovery-build-observe", &[]).unwrap_err();
        assert_eq!(error.code(), "MISSING_OPTION");
    }

    #[test]
    fn recovery_build_observe_rejects_unknown_duplicate_and_relative_options_before_io() {
        let unknown = test_recovery_action(
            "recovery-build-observe",
            &["--untrusted".into(), "/tmp/value".into()],
        )
        .unwrap_err();
        assert_eq!(unknown.code(), "UNKNOWN_OPTION");

        let mut duplicate_args = complete_args();
        duplicate_args.extend(["--provenance".into(), "/tmp/other".into()]);
        let duplicate =
            test_recovery_action("recovery-build-observe", &duplicate_args).unwrap_err();
        assert_eq!(duplicate.code(), "DUPLICATE_OPTION");

        let mut relative_args = complete_args();
        relative_args[1] = "relative.json".into();
        let relative = test_recovery_action("recovery-build-observe", &relative_args).unwrap_err();
        assert_eq!(relative.code(), "INVALID_RECOVERY_PRODUCTION");
    }
}
