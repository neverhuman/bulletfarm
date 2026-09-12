#!/usr/bin/env bash
# Sourced by the admitted dispatcher. Source-built fixture runs, never live proof.
[[ "${BASH_SOURCE[0]}" != "$0" ]] || {
  printf 'OPERATOR_TUI_WRAPPER_REQUIRED: use scripts/ci-local.sh operator-tui\n' >&2
  exit 78
}

hosted_verify_source() {
  local entry mode kind expected relative actual other
  git ls-tree -r HEAD >/dev/null || return 1
  git ls-files --others >/dev/null || return 1
  while IFS= read -r -d '' entry; do
    IFS=' ' read -r mode kind expected <<<"${entry%%$'\t'*}"
    relative="${entry#*$'\t'}"
    [[ "$kind" == blob && ( "$mode" == 100644 || "$mode" == 100755 ) \
      && -f "$relative" && ! -L "$relative" \
      && "$(realpath -e "$relative")" == "$REPO_ROOT/$relative" ]] \
      || { refuse OPERATOR_TUI_RAW_SOURCE_CHANGED "$relative"; return 1; }
    actual="$(git hash-object --no-filters "$relative")" || return 1
    [[ "$actual" == "$expected" ]] \
      || { refuse OPERATOR_TUI_RAW_SOURCE_CHANGED "$relative"; return 1; }
    [[ ( "$mode" == 100755 && -x "$relative" ) || ( "$mode" == 100644 && ! -x "$relative" ) ]] \
      || { refuse OPERATOR_TUI_RAW_SOURCE_MODE_CHANGED "$relative"; return 1; }
  done < <(git ls-tree -rz HEAD)
  # Ignored files can still affect compilation (for example Cargo config or
  # build-script inputs). Only the declared diagnostic output is exempt.
  while IFS= read -r -d '' other; do
    [[ "$other" == .ci-artifacts/* ]] \
      || { refuse OPERATOR_TUI_UNTRACKED_SOURCE_INPUT "$other"; return 1; }
  done < <(git ls-files --others -z)
}

hosted_build_and_run() {
  local stage cargo rustc cargo_sha rustc_sha name binary digest status source_hash after
  local admitted_context="${hosted_before:?dispatcher context required}"
  [[ ${RUNNER_TEMP:-} == /* && -d "$RUNNER_TEMP" && ! -L "$RUNNER_TEMP" \
    && "$(realpath -e "$RUNNER_TEMP")" == "$RUNNER_TEMP" ]] \
    || { refuse OPERATOR_TUI_RUNNER_TEMP_INVALID 'canonical runner temporary directory required'; return 1; }
  stage="$RUNNER_TEMP/bullet-operator-tui-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
  [[ ! -e "$stage" && ! -L "$stage" ]] \
    || { refuse OPERATOR_TUI_OUTPUT_NOT_FRESH "$stage"; return 1; }
  hosted_verify_source
  mkdir -m 0700 "$stage"
  mkdir -m 0700 "$stage/bin"
  cargo="$(rustup which cargo)"; rustc="$(rustup which rustc)"
  cargo_sha="$(sha256_file "$cargo")"; rustc_sha="$(sha256_file "$rustc")"
  admit_binary cargo "$cargo" "$cargo_sha"
  admit_binary rustc "$rustc" "$rustc_sha"
  git ls-tree -r HEAD >"$stage/source-tree.txt"
  # Both manifests are locked. Fetch happens explicitly before frozen builds.
  env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS \
    RUSTC="$rustc" "$cargo" fetch --locked >"$stage/fetch.stdout" 2>"$stage/fetch.stderr"
  env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS \
    RUSTC="$rustc" "$cargo" fetch --locked --manifest-path ops/qualification/tuiwright/Cargo.toml \
    >"$stage/harness-fetch.stdout" 2>"$stage/harness-fetch.stderr"
  env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS \
    RUSTC="$rustc" "$cargo" build --frozen -p bullet --bin bullet --bin bulletfarm \
    --message-format=json >"$stage/cli-build.jsonl" 2>"$stage/cli-build.stderr"
  jq -se 'any(.[]; .reason == "build-finished" and .success == true)' "$stage/cli-build.jsonl" >/dev/null
  for name in bullet bulletfarm; do
    binary="$CARGO_TARGET_DIR/debug/$name"
    jq -se --arg name "$name" --arg executable "$binary" '
      [.[] | select(.reason == "compiler-artifact" and .target.name == $name
        and .executable == $executable)] | length == 1' "$stage/cli-build.jsonl" >/dev/null
    digest="$(sha256_file "$binary")"
    admit_binary "$name" "$binary" "$digest"
    cp "$binary" "$stage/bin/$name"
    chmod 0500 "$stage/bin/$name"
    admit_binary "$name" "$stage/bin/$name" "$digest"
  done
  hosted_verify_source
  admit_binary cargo "$cargo" "$cargo_sha"
  admit_binary rustc "$rustc" "$rustc_sha"
  after="$(hosted_subjects)"
  [[ "$after" == "$admitted_context" ]] \
    || { refuse OPERATOR_TUI_HOSTED_SUBJECT_CHANGED 'during source build'; return 1; }
  source_hash="$(find ops/qualification/tuiwright -type f -print0 | LC_ALL=C sort -z \
    | xargs -0 -r sha256sum | sha256sum | cut -d ' ' -f1)"
  # Sequential suites reserve this runner for terminal load. Each six-client
  # scenario still owns all six concurrent sessions; no case is filtered out.
  for name in bullet bulletfarm; do
    hosted_verify_source
    status=0
    BULLET_TUIWRIGHT_CARGO_BIN="$cargo" BULLET_TUIWRIGHT_CARGO_SHA256="$cargo_sha" \
    BULLET_TUIWRIGHT_RUSTC_BIN="$rustc" BULLET_TUIWRIGHT_RUSTC_SHA256="$rustc_sha" \
    BULLET_TUIWRIGHT_BULLET_BIN="$stage/bin/$name" BULLET_TUIWRIGHT_BULLET_SHA256="$(sha256_file "$stage/bin/$name")" \
    BULLET_TUIWRIGHT_SOURCE_SHA256="$source_hash" BULLET_TUIWRIGHT_OUTPUT="$stage/$name" \
      bash ops/ci/operator-tui.sh >"$stage/$name.stdout" 2>"$stage/$name.stderr" || status=$?
    printf '%s\n' "$status" >"$stage/$name.exit-status"
    [[ "$status" == 0 ]] || return "$status"
    hosted_verify_source
  done
  hosted_verify_source
  admit_binary cargo "$cargo" "$cargo_sha"
  admit_binary rustc "$rustc" "$rustc_sha"
  after="$(hosted_subjects)"
  [[ "$after" == "$admitted_context" ]] \
    || { refuse OPERATOR_TUI_HOSTED_SUBJECT_CHANGED 'after both alias runs'; return 1; }
  # Full artifact inventory is portable; original paths remain diagnostic subjects.
  local artifacts='[]' relative executables='{}'
  while IFS= read -r -d '' binary; do
    [[ -f "$binary" && ! -L "$binary" ]] || return 1
    relative="${binary#"$stage/"}"
    artifacts="$(jq -cn --argjson rows "$artifacts" --arg path "$relative" \
      --arg sha256 "$(sha256_file "$binary")" '$rows + [{path:$path,sha256:$sha256}]')"
  done < <(find "$stage" -type f -print0 | LC_ALL=C sort -z)
  for name in bullet bulletfarm; do
    executables="$(jq -cn --argjson rows "$executables" --arg name "$name" --arg path "bin/$name" \
      --arg original_path "$stage/bin/$name" --arg sha256 "$(sha256_file "$stage/bin/$name")" \
      '$rows + {($name):{path:$path,original_path:$original_path,sha256:$sha256}}')"
  done
  jq -n --argjson context "$admitted_context" --argjson executables "$executables" \
    --arg build_target "$CARGO_TARGET_DIR" --argjson artifacts "$artifacts" '
    {schema:"bullet.operator-tui.hosted.v1",context:$context,process_exit:0,
      executables:$executables,build_target:$build_target,artifacts:$artifacts}' >"$stage/receipt.json"
  printf 'OPERATOR_TUI_HOSTED_COMPONENT_PASS: both source-built aliases; artifacts=%s\n' "$stage"
}
