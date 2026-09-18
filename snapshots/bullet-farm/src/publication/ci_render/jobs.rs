//! Root workflow topology and explicit unavailable runtime profiles.
use super::{Manifest, Result, TEMPLATE, Workflow, require};
use crate::publication::ci_inventory::Job;

pub(super) fn job_id(workflow: &Workflow, id: &str) -> String {
    if workflow.member == "bullet-farm" && workflow.scope == "REQUIRED" && id == "source_scan" {
        "publication_integrity".into()
    } else {
        format!(
            "{}_{}",
            workflow.member.replace('-', "_"),
            id.replace('-', "_")
        )
    }
}

fn unavailable(manifest: &Manifest, workflow: &Workflow, job: &Job) -> String {
    let id = job_id(workflow, job.id);
    let key = format!("{}:{}:{}", workflow.member, workflow.scope, job.id);
    let subject = &manifest.members[workflow.member];
    let profile = if job.id == "required" {
        "MEMBER_AGGREGATOR_ARTIFACTS"
    } else if matches!(job.id, "audit" | "hygiene") {
        "PORTABLE_AUDITOR_SOURCE_ARTIFACT"
    } else if job.runner != "ubuntu-24.04" {
        "NATIVE_RUNNER_TOOL_SUBJECTS"
    } else if matches!(job.id, "source_scan" | "preflight" | "source-admission") {
        "SOURCE_SCAN_TOOL_CLOSURE"
    } else if workflow.member == "bullet-portal" {
        "NODE_NPM_TOOL_SUBJECTS_AND_CAPACITY"
    } else {
        "RUST_CARGO_TOOL_SUBJECTS_AND_CAPACITY"
    };
    let mut out = format!(
        "\n  {id}:\n    name: '{key}'\n    runs-on: {}\n    timeout-minutes: {}\n",
        job.runner, job.timeout_minutes
    );
    if !job.needs.is_empty() {
        let needs = job
            .needs
            .iter()
            .map(|id| job_id(workflow, id))
            .collect::<Vec<_>>();
        out.push_str(&format!("    needs: [{}]\n", needs.join(", ")));
    }
    if job.always {
        out.push_str("    if: ${{ always() }}\n");
    }
    if !job.os.is_empty() {
        out.push_str(&format!(
            "    strategy:\n      fail-fast: false\n      matrix:\n        os: [{}]\n",
            job.os.join(", ")
        ));
    }
    out.push_str(&format!(
        "    env:\n      CI_INVOCATION_KEY: '{key}'\n      CI_MEMBER_COMMIT: '{}'\n      CI_MEMBER_TREE: '{}'\n      CI_NESTED_WORKFLOW: '{}'\n      CI_NESTED_WORKFLOW_SHA256: '{}'\n      CI_REQUIRED_ADMISSION: '{profile}'\n",
        subject.commit, subject.tree, workflow.path, workflow.sha256
    ));
    out.push_str("      CI_AGGREGATE_EVENT_SHA: ${{ github.sha }}\n      CI_ROOT_WORKFLOW_SHA: ${{ github.workflow_sha }}\n      CI_EVENT: ${{ github.event_name }}\n      CI_RUN_ID: ${{ github.run_id }}\n      CI_RUN_ATTEMPT: ${{ github.run_attempt }}\n");
    if !job.os.is_empty() {
        out.push_str("      CI_MATRIX_OS: ${{ matrix.os }}\n");
    }
    out.push_str("    steps:\n      - name: Refuse the unavailable runtime admission\n        shell: bash\n        run: |\n          printf 'PUBLICATION_CI_RUNTIME_ADMISSION_UNAVAILABLE: %s requires %s; execution_evidence=false\\n' \"$CI_INVOCATION_KEY\" \"$CI_REQUIRED_ADMISSION\" >&2\n          exit 1\n");
    out
}

pub(super) fn final_step(ids: &[String]) -> String {
    let mut out = String::from(
        "      - name: Reject incomplete family execution and absent profile artifacts\n        if: ${{ always() }}\n        shell: bash\n        env:\n",
    );
    for (index, id) in ids.iter().enumerate() {
        out.push_str(&format!(
            "          RESULT_{index}: ${{{{ needs.{id}.result }}}}\n"
        ));
    }
    let results = (0..ids.len())
        .map(|i| format!("\"$RESULT_{i}\""))
        .collect::<Vec<_>>();
    out.push_str(&format!("        run: |\n          for result in {}; do\n            if [[ \"$result\" != success ]]; then\n              printf 'PUBLICATION_CI_REQUIRED_RESULT_INVALID: %s\\n' \"$result\" >&2\n              exit 1\n            fi\n          done\n", results.join(" ")));
    out.push_str("          printf 'PUBLICATION_CI_FULL_EXECUTION_ARTIFACT_ADMISSION_UNAVAILABLE; execution_evidence=false\\n' >&2\n          exit 1\n");
    out
}

// Infrastructure steps belong only to v2; the authentic v1 template is untouched.
const TRANSFER: &str = r#"      - id: transfer
        name: Bind the proven verifier and scanner for this attempt
        env:
          BULLET_BOOTSTRAP_COMPLETION_SHA256: ${{ steps.prove.outputs.bootstrap_completion_sha256 }}
          BULLET_HUB_COMPLETION_SHA256: ${{ steps.member.outputs.member_completion_sha256 }}
        run: bash bullet-farm/publication/ci-transfer.sh prepare
      - name: Upload the exact diagnostic tools for the dependent lane
        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02
        with:
          name: publication-tools-${{ github.run_id }}-${{ github.run_attempt }}
          path: ${{ runner.temp }}/bullet-publication-transfer/
          if-no-files-found: error
          retention-days: 14
"#;
const GIT_STEPS: &str = r#"    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683
        with:
          ref: ${{ github.sha }}
          fetch-depth: 0
          persist-credentials: false
      - name: Download this attempt's exact tool artifact
        uses: actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093
        with:
          name: publication-tools-${{ github.run_id }}-${{ github.run_attempt }}
          path: ${{ runner.temp }}/bullet-publication-transfer-download/
      - name: Verify artifact identity before executing the transferred tools
        env:
          BULLET_BOOTSTRAP_RESULT: ${{ needs.publication_integrity.result }}
          BULLET_TRANSFER_SHA256: ${{ needs.publication_integrity.outputs.transfer_sha256 }}
          BULLET_BOOTSTRAP_COMPLETION_SHA256: ${{ needs.publication_integrity.outputs.bootstrap_completion_sha256 }}
          BULLET_HUB_COMPLETION_SHA256: ${{ needs.publication_integrity.outputs.member_completion_sha256 }}
        run: bash bullet-farm/publication/ci-transfer.sh receive
      - name: Reconstruct the exact retained source refs in this fresh job
        shell: bash
        run: |
          timeout --signal=TERM --kill-after=5s 120s "$RUNNER_TEMP/bullet-publication-target/debug/bullet-publish" reconstruct "$GITHUB_WORKSPACE" "$RUNNER_TEMP/bullet-publication-family" >"$RUNNER_TEMP/bullet-publication-target/reconstruct.log" 2>&1
      - id: member
        name: Execute and validate the exact BulletGit source scan
        run: PATH="$RUNNER_TEMP/bullet-tools:/usr/bin:/bin" bash bullet-farm/publication/ci-required.sh git-member-run
      - name: Retain only the successful exact Git diagnostic
        if: ${{ !cancelled() && steps.member.outcome == 'success' }}
        uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02
        with:
          name: publication-git-source-scan-${{ github.run_id }}-${{ github.run_attempt }}
          path: ${{ runner.temp }}/bullet-publication-git-member-report/
          include-hidden-files: true
          if-no-files-found: error
          retention-days: 14
"#;
const GIT_FINAL: &str = r#"      - name: Download this attempt's exact Git source-scan report
        if: ${{ always() }}
        uses: actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093
        with:
          name: publication-git-source-scan-${{ github.run_id }}-${{ github.run_attempt }}
          path: ${{ runner.temp }}/bullet-publication-git-downloaded/publication-git-source-scan-${{ github.run_id }}-${{ github.run_attempt }}/
      - name: Reject absent, stale or unsuccessful Git source-scan completion
        if: ${{ always() }}
        env:
          BULLET_GIT_SOURCE_SCAN_RESULT: ${{ needs.bullet_git_source_scan.result }}
          BULLET_GIT_MEMBER_COMPLETION_SHA256: ${{ needs.bullet_git_source_scan.outputs.git_member_completion_sha256 }}
          BULLET_EXPECTED_VERIFIER_SHA256: ${{ needs.publication_integrity.outputs.verifier_sha256 }}
        run: bash bullet-farm/publication/ci-transfer.sh git-required
"#;

fn git_source_scan(manifest: &Manifest, workflow: &Workflow, job: &Job) -> String {
    let stub = unavailable(manifest, workflow, job);
    let (header, _) = stub.split_once("    steps:\n").unwrap();
    let header = header.replacen("    env:\n", "    needs: [publication_integrity]\n    outputs:\n      git_member_completion_sha256: ${{ steps.member.outputs.git_member_completion_sha256 }}\n    env:\n", 1);
    format!(
        "{}{GIT_STEPS}",
        header.replace(
            "SOURCE_SCAN_TOOL_CLOSURE",
            "COOPERATIVE_DIAGNOSTIC_TRANSFER_ONLY"
        )
    )
}

pub(super) fn required(manifest: &Manifest, catalog: &[Workflow]) -> Result<String> {
    // Exact compiled template admission above makes these transformations closed.
    require(
        TEMPLATE.matches("\n  publication_required:\n").count() == 1,
        "PUBLICATION_CI_TEMPLATE_UNSUPPORTED",
    )?;
    let (producer, final_job) = TEMPLATE.split_once("\n  publication_required:\n").unwrap();
    let mut out = producer
        .replace("name: Publication bootstrap\n", "name: Publication family CI\n")
        .replace("  push:\n    branches: [main]\n", "  push:\n  merge_group:\n    types: [checks_requested]\n")
        .replacen("    steps:\n", "    steps:\n      - name: Admit the implemented bootstrap event context\n        shell: bash\n        run: |\n          case \"$GITHUB_EVENT_NAME:$GITHUB_REF\" in\n            pull_request:refs/pull/*/merge|push:refs/heads/main|workflow_dispatch:refs/heads/*) ;;\n            *) printf 'PUBLICATION_CI_BOOTSTRAP_EVENT_UNSUPPORTED\\n' >&2; exit 1 ;;\n          esac\n", 1);
    out = out.replacen("    outputs:\n", "    outputs:\n      transfer_sha256: ${{ steps.transfer.outputs.transfer_sha256 }}\n      verifier_sha256: ${{ steps.transfer.outputs.verifier_sha256 }}\n", 1);
    out.push_str(TRANSFER);
    let mut ids = Vec::new();
    for workflow in catalog.iter().filter(|w| w.scope == "REQUIRED") {
        for job in &workflow.jobs {
            let id = job_id(workflow, job.id);
            require(!ids.contains(&id), "PUBLICATION_CI_JOB_ID_COLLISION")?;
            if id == "bullet_git_source_scan" {
                out.push_str(&git_source_scan(manifest, workflow, job));
            } else if id != "publication_integrity" {
                out.push_str(&unavailable(manifest, workflow, job));
            }
            ids.push(id);
        }
    }
    out.push_str("\n  publication_required:\n");
    out.push_str(&final_job.replace(
        "    needs: [publication_integrity]\n",
        &format!("    needs: [{}]\n", ids.join(", ")),
    ));
    out.push_str(GIT_FINAL);
    out.push_str(&final_step(&ids));
    Ok(out)
}

pub(super) fn scheduled(manifest: &Manifest, workflow: &Workflow) -> String {
    let cron = if matches!(workflow.member, "bullet-farm" | "bullet-portal") {
        "17 4 * * 1"
    } else {
        "23 4 * * 1"
    };
    let mut out = format!(
        "# Generated from the exact admitted nested workflow; runtime admissions remain open.\nname: '{} scheduled'\non:\n  schedule:\n    - cron: '{cron}'\n  workflow_dispatch:\npermissions:\n  contents: read\nconcurrency:\n  group: publication-{}-${{{{ github.run_id }}}}\n  cancel-in-progress: false\njobs:\n",
        workflow.member, workflow.member
    );
    let ids = workflow
        .jobs
        .iter()
        .map(|job| job_id(workflow, job.id))
        .collect::<Vec<_>>();
    for job in &workflow.jobs {
        out.push_str(&unavailable(manifest, workflow, job));
    }
    out.push_str(&format!("\n  publication_scheduled_required:\n    name: '{} scheduled required'\n    needs: [{}]\n    if: ${{{{ always() }}}}\n    runs-on: ubuntu-24.04\n    timeout-minutes: 5\n    steps:\n", workflow.member, ids.join(", ")));
    out.push_str(&final_step(&ids));
    out
}
