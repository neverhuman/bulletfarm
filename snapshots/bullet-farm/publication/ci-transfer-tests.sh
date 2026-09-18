#!/usr/bin/env bash
# Real transferred verifier/scanner fixtures. Called inside the existing wrapper.
set -euo pipefail
transfer="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/ci-transfer.sh"
[[ "$#" == 4 ]] || exit 2
mode="$1"; hub_digest="$2"; bootstrap_digest="$3"; reconstructed="$4"
[[ "$hub_digest" =~ ^[0-9a-f]{64}$ && "$bootstrap_digest" =~ ^[0-9a-f]{64}$ ]]
scratch="$(mktemp -d)"
trap 'rm -rf -- "$scratch"' EXIT
base=(env "BULLET_BOOTSTRAP_COMPLETION_SHA256=$bootstrap_digest" "BULLET_HUB_COMPLETION_SHA256=$hub_digest")
exported="$RUNNER_TEMP/bullet-publication-transfer"

hash() { sha256sum "$1" | cut -d ' ' -f 1; }
refuses() {
  local code="$1" name="$2"
  shift 2
  if "$@" >"$scratch/$name.log" 2>&1; then
    printf 'transfer fixture unexpectedly succeeded: %s\n' "$name" >&2; exit 1
  fi
  grep -Fq "$code" "$scratch/$name.log" || { cat "$scratch/$name.log" >&2; exit 1; }
  printf 'publication transfer fixture: %s ... ok\n' "$name"
}

prepare_fixture() {
  "${base[@]}" GITHUB_JOB=publication_integrity bash "$transfer" prepare
  local digest candidate expected code
  digest="$(sed -n 's/^transfer_sha256=//p' "$GITHUB_OUTPUT")"
  [[ "$digest" =~ ^[0-9a-f]{64}$ && "$(hash "$exported/transfer.json")" == "$digest" ]]
  local name
  for name in tool scanner missing empty extra symbolic duplicate wrong_digest stale_run stale_attempt subject source; do
    candidate="$scratch/$name"
    mkdir "$candidate"
    cp -R "$exported" "$candidate/bullet-publication-transfer-download"
    local download="$candidate/bullet-publication-transfer-download"
    expected="$digest"; code=PUBLICATION_TRANSFER_TOOL_CHANGED
    local -a context=()
    case "$name" in
      tool) printf 'changed\n' >>"$download/bullet-publish" ;;
      scanner) printf 'changed\n' >>"$download/gitleaks" ;;
      missing) rm "$download/gitleaks"; code=PUBLICATION_REQUIRED_INVENTORY ;;
      empty) : >"$download/bullet-publish"; code=PUBLICATION_REQUIRED_SIZE ;;
      extra) printf 'extra\n' >"$download/extra"; code=PUBLICATION_REQUIRED_INVENTORY ;;
      symbolic)
        rm "$download/bullet-publish"
        ln -s "$exported/bullet-publish" "$download/bullet-publish"
        code=PUBLICATION_REQUIRED_INVENTORY ;;
      duplicate)
        sed -i '1s/{/{"run_id":"duplicate",/' "$download/transfer.json"
        code=STRICT_JSON_ ;;
      wrong_digest) expected="$(printf '%064d' 0)"; code=PUBLICATION_TRANSFER_DIGEST ;;
      stale_run) context=(GITHUB_RUN_ID=124); code=PUBLICATION_TRANSFER_SUBJECT ;;
      stale_attempt) context=(GITHUB_RUN_ATTEMPT=3); code=PUBLICATION_TRANSFER_SUBJECT ;;
      subject|source)
        local field=aggregate_commit length=40
        if [[ "$name" == source ]]; then field=script_sha256; length=64; fi
        jq --arg field "$field" --argjson length "$length" '.[$field] = ("0" * $length)' "$download/transfer.json" >"$scratch/changed.json"
        mv "$scratch/changed.json" "$download/transfer.json"
        expected="$(hash "$download/transfer.json")"; code=PUBLICATION_TRANSFER_SUBJECT ;;
    esac
    refuses "$code" "$name" "${base[@]}" "RUNNER_TEMP=$candidate" GITHUB_JOB=bullet_git_source_scan \
      "${context[@]}" BULLET_BOOTSTRAP_RESULT=success "BULLET_TRANSFER_SHA256=$expected" bash "$transfer" receive
    [[ ! -e "$candidate/bullet-publication-target" && ! -e "$candidate/bullet-tools" ]]
  done
  candidate="$scratch/results"; mkdir "$candidate"
  cp -R "$exported" "$candidate/bullet-publication-transfer-download"
  for name in failure cancelled skipped neutral malformed ''; do
    refuses PUBLICATION_TRANSFER_BOOTSTRAP_FAILED "result-${name:-empty}" "${base[@]}" \
      "RUNNER_TEMP=$candidate" GITHUB_JOB=bullet_git_source_scan "BULLET_BOOTSTRAP_RESULT=$name" \
      "BULLET_TRANSFER_SHA256=$digest" bash "$transfer" receive
  done
  # Remove every expected bootstrap input path; preserved originals are never used by receive.
  mkdir "$scratch/bootstrap"
  mv "$RUNNER_TEMP/bullet-publication-family" "$RUNNER_TEMP/bullet-publication-target" \
    "$RUNNER_TEMP/bullet-tools" "$scratch/bootstrap/"
  cp -R "$exported" "$RUNNER_TEMP/bullet-publication-transfer-download"
  "${base[@]}" GITHUB_JOB=bullet_git_source_scan BULLET_BOOTSTRAP_RESULT=success \
    "BULLET_TRANSFER_SHA256=$digest" bash "$transfer" receive
  [[ "$(hash "$RUNNER_TEMP/bullet-publication-target/debug/bullet-publish")" == "$(hash "$exported/bullet-publish")" ]]
  [[ "$(hash "$RUNNER_TEMP/bullet-tools/gitleaks")" == "$(hash "$exported/gitleaks")" ]]
  # This existing fixture independently cloned exact member objects from the aggregate.
  # The hosted job separately uses the reviewed public-source-ref reconstruction CLI.
  mv "$reconstructed" "$RUNNER_TEMP/bullet-publication-family"
  for name in bullet-farm bullet-git; do
    git -C "$RUNNER_TEMP/bullet-publication-family/$name" config user.name 'Publication fixture'
    git -C "$RUNNER_TEMP/bullet-publication-family/$name" config user.email fixture@bullet.invalid
  done
  cp -R "$scratch/bootstrap/bullet-publication-family/bullet-farm/.ci-artifacts" \
    "$RUNNER_TEMP/bullet-publication-family/bullet-farm/.ci-artifacts"
  printf 'publication transfer fixture: real-verifier-import ... ok\n'
}

final_fixture() {
  local directory="$RUNNER_TEMP/bullet-publication-git-downloaded/publication-git-source-scan-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
  local completion="$directory/completion.json" digest expected
  digest="$(hash "$completion")"
  expected="$(jq -er '.verifier_sha256' "$exported/transfer.json")"
  local -a final=(env GITHUB_JOB=publication_required BULLET_GIT_SOURCE_SCAN_RESULT=success
    "BULLET_GIT_MEMBER_COMPLETION_SHA256=$digest" "BULLET_EXPECTED_VERIFIER_SHA256=$expected")
  "${final[@]}" bash "$transfer" git-required
  refuses PUBLICATION_TRANSFER_VERIFIER wrong-final-verifier "${final[@]}" \
    "BULLET_EXPECTED_VERIFIER_SHA256=$(printf '%064d' 0)" bash "$transfer" git-required
  refuses PUBLICATION_TRANSFER_VERIFIER missing-final-verifier "${final[@]}" \
    BULLET_EXPECTED_VERIFIER_SHA256= bash "$transfer" git-required
  cp "$completion" "$scratch/completion.json"
  jq '.verifier_sha256 = ("0" * 64)' "$completion" >"$scratch/changed.json"
  mv "$scratch/changed.json" "$completion"
  refuses PUBLICATION_TRANSFER_VERIFIER changed-final-verifier "${final[@]}" \
    "BULLET_GIT_MEMBER_COMPLETION_SHA256=$(hash "$completion")" bash "$transfer" git-required
  cp "$scratch/completion.json" "$completion"
  "${final[@]}" bash "$transfer" git-required
  printf 'publication transfer fixture: exact-final-verifier ... ok\n'
}

case "$mode" in
  prepare) prepare_fixture ;;
  final) final_fixture ;;
  *) exit 2 ;;
esac
