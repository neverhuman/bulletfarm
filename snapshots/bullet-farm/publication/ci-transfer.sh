#!/usr/bin/env bash
# Same-run diagnostic tool transfer. No installed-runtime or release admission.
set -euo pipefail
# shellcheck source=publication/ci-required.sh
source "$(dirname "${BASH_SOURCE[0]}")/ci-required.sh"
for variable in "${!GIT_@}"; do unset "$variable"; done
export LC_ALL=C TZ=UTC GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_NO_REPLACE_OBJECTS=1
umask 077

transfer_manifest() {
  local root="$1" expected="$2" document="$1/transfer.json"
  publication_inventory "$root" $'f bullet-publish\nf gitleaks\nf transfer.json'
  publication_json "$document"
  [[ "$expected" =~ ^[0-9a-f]{64}$ && "$(publication_hash "$document")" == "$expected" ]] \
    || { publication_refuse PUBLICATION_TRANSFER_DIGEST; return 1; }
  jq -e --arg event "$GITHUB_SHA" --arg workflow "$GITHUB_WORKFLOW_SHA" \
    --arg run "$GITHUB_RUN_ID" --arg attempt "$GITHUB_RUN_ATTEMPT" \
    --arg manifest "$(publication_hash "$GITHUB_WORKSPACE/publication.json")" \
    --arg script "$(publication_hash "$PUBLICATION_HUB/publication/ci-transfer.sh")" \
    --arg bootstrap "${BULLET_BOOTSTRAP_COMPLETION_SHA256:-}" --arg hub "${BULLET_HUB_COMPLETION_SHA256:-}" '
    (keys|sort) == (["schema_version","aggregate_commit","workflow_sha","run_id","run_attempt","manifest_sha256",
      "script_sha256","bootstrap_sha256","hub_sha256","verifier_sha256","scanner_sha256",
      "tool_closure_admitted","signed","release_authority","evidence_class"]|sort) and
    .schema_version == "bullet.publication-tool-transfer.v1" and .aggregate_commit == $event and
    .workflow_sha == $workflow and .run_id == $run and .run_attempt == $attempt and
    .manifest_sha256 == $manifest and .script_sha256 == $script and
    .bootstrap_sha256 == $bootstrap and .hub_sha256 == $hub and
    all([$bootstrap,$hub,.verifier_sha256,.scanner_sha256][]; type == "string" and test("^[0-9a-f]{64}$")) and
    .scanner_sha256 == "50b742abd7daad8bbddb6301f3017efb680632d9a5b3b4d8f137b3aac250e359" and
    .tool_closure_admitted == false and .signed == false and .release_authority == false and
    .evidence_class == "DIAGNOSTIC_ONLY"
  ' "$document" >/dev/null || { publication_refuse PUBLICATION_TRANSFER_SUBJECT; return 1; }
  local name key
  for name in bullet-publish gitleaks; do
    key=verifier_sha256; [[ "$name" != gitleaks ]] || key=scanner_sha256
    publication_file "$root/$name" 1073741824
    [[ "$(publication_hash "$root/$name")" == "$(jq -r --arg key "$key" '.[$key]' "$document")" ]] \
      || { publication_refuse PUBLICATION_TRANSFER_TOOL_CHANGED; return 1; }
  done
}

transfer_prepare() {
  publication_host publication_integrity
  local root="$RUNNER_TEMP/bullet-publication-transfer" binary="$RUNNER_TEMP/bullet-publication-target/debug/bullet-publish"
  local scanner="$RUNNER_TEMP/bullet-tools/gitleaks" hub="$RUNNER_TEMP/bullet-publication-member-report"
  publication_validate_bootstrap "$RUNNER_TEMP/bullet-publication-report" "${BULLET_BOOTSTRAP_COMPLETION_SHA256:-}"
  publication_validate_member "$hub" "${BULLET_HUB_COMPLETION_SHA256:-}"
  publication_file "$binary" 1073741824
  [[ "$(publication_hash "$binary")" == "$(jq -r '.verifier_sha256' "$hub/completion.json")" ]] \
    || { publication_refuse PUBLICATION_TRANSFER_VERIFIER; return 1; }
  publication_tools "$scanner" >/dev/null
  mkdir "$root"
  cp "$binary" "$root/bullet-publish"
  cp "$scanner" "$root/gitleaks"
  jq -n --arg event "$GITHUB_SHA" --arg workflow "$GITHUB_WORKFLOW_SHA" --arg run "$GITHUB_RUN_ID" \
    --arg attempt "$GITHUB_RUN_ATTEMPT" --arg manifest "$(publication_hash "$GITHUB_WORKSPACE/publication.json")" \
    --arg script "$(publication_hash "$PUBLICATION_HUB/publication/ci-transfer.sh")" \
    --arg bootstrap "$BULLET_BOOTSTRAP_COMPLETION_SHA256" --arg hub "$BULLET_HUB_COMPLETION_SHA256" \
    --arg verifier "$(publication_hash "$binary")" --arg scanner "$(publication_hash "$scanner")" '
    {schema_version:"bullet.publication-tool-transfer.v1",aggregate_commit:$event,workflow_sha:$workflow,
     run_id:$run,run_attempt:$attempt,manifest_sha256:$manifest,script_sha256:$script,
     bootstrap_sha256:$bootstrap,hub_sha256:$hub,verifier_sha256:$verifier,scanner_sha256:$scanner,
     tool_closure_admitted:false,signed:false,release_authority:false,evidence_class:"DIAGNOSTIC_ONLY"}
  ' >"$root/transfer.json"
  local digest
  digest="$(publication_hash "$root/transfer.json")"
  transfer_manifest "$root" "$digest"
  publication_host publication_integrity
  [[ -f "${GITHUB_OUTPUT:-}" && ! -L "$GITHUB_OUTPUT" ]] || { publication_refuse PUBLICATION_TRANSFER_OUTPUT; return 1; }
  transfer_verifier "$hub/completion.json" "$(jq -r '.verifier_sha256' "$root/transfer.json")"
  printf 'transfer_sha256=%s\n' "$digest" >>"$GITHUB_OUTPUT"
  printf 'verifier_sha256=%s\n' "$(jq -r '.verifier_sha256' "$root/transfer.json")" >>"$GITHUB_OUTPUT"
}

transfer_receive() {
  publication_host bullet_git_source_scan
  [[ "${BULLET_BOOTSTRAP_RESULT:-}" == success ]] \
    || { publication_refuse PUBLICATION_TRANSFER_BOOTSTRAP_FAILED; return 1; }
  local root="$RUNNER_TEMP/bullet-publication-transfer-download" target="$RUNNER_TEMP/bullet-publication-target"
  local tools="$RUNNER_TEMP/bullet-tools" expected="${BULLET_TRANSFER_SHA256:-}"
  transfer_manifest "$root" "$expected"
  # A download's own checksum cannot establish trust: expected comes from this
  # attempt's successful bootstrap job output, bound to the current aggregate.
  mkdir "$target" "$target/debug" "$tools"
  cp "$root/bullet-publish" "$target/debug/bullet-publish"
  cp "$root/gitleaks" "$tools/gitleaks"
  chmod 500 "$target/debug/bullet-publish" "$tools/gitleaks"
  [[ "$(publication_hash "$target/debug/bullet-publish")" == "$(publication_hash "$root/bullet-publish")" \
    && "$(publication_hash "$tools/gitleaks")" == "$(publication_hash "$root/gitleaks")" ]] \
    || { publication_refuse PUBLICATION_TRANSFER_TOOL_CHANGED; return 1; }
  publication_tools "$tools/gitleaks" >/dev/null
  # Validation precedes the first execution of the transferred verifier.
  (ulimit -f 16384; timeout --signal=TERM --kill-after=5s 60s \
    "$target/debug/bullet-publish" verify "$GITHUB_WORKSPACE") >"$target/transfer-verify.log" 2>&1 \
    || { publication_refuse PUBLICATION_TRANSFER_VERIFY_FAILED; return 1; }
  transfer_manifest "$root" "$expected"
  transfer_verifier "$root/transfer.json" "$(publication_hash "$target/debug/bullet-publish")"
  publication_tools "$tools/gitleaks" >/dev/null
  publication_host bullet_git_source_scan
}

transfer_verifier() {
  publication_file "$1"
  [[ "$2" =~ ^[0-9a-f]{64}$ && "$(jq -er '.verifier_sha256' "$1")" == "$2" ]] \
    || { publication_refuse PUBLICATION_TRANSFER_VERIFIER; return 1; }
}

[[ "$#" == 1 ]] || { publication_refuse PUBLICATION_TRANSFER_USAGE; exit 2; }
case "$1" in
  prepare) transfer_prepare ;;
  receive) transfer_receive ;;
  git-required)
    transfer_verifier "$RUNNER_TEMP/bullet-publication-git-downloaded/publication-git-source-scan-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT/completion.json" \
      "${BULLET_EXPECTED_VERIFIER_SHA256:-}"
    publication_git_required
    ;;
  *) publication_refuse PUBLICATION_TRANSFER_USAGE; exit 2 ;;
esac
