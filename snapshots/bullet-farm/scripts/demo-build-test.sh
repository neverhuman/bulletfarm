#!/usr/bin/env bash
# Run the real wrapper against a copied family and fake Cargo only.
set -euo pipefail
HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
fixture_root="$(mktemp -d /tmp/bullet-demo-wrapper-test.XXXXXX)"
trap 'rm -rf -- "$fixture_root"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
mkdir "$fixture_root/bin"

cat >"$fixture_root/bin/mktemp" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
case "$*" in
  '-d /tmp/bullet-txn.XXXXXX') exec /usr/bin/mktemp -d "$CASE/tmp/data.XXXXXX" ;;
  '-d /tmp/bullet-demo-target.XXXXXX') exec /usr/bin/mktemp -d "$CASE/tmp/target.XXXXXX" ;;
  '-d /tmp/bullet-verifier-fixture.XXXXXX') exec /usr/bin/mktemp -d "$CASE/tmp/verifier.XXXXXX" ;;
  *) [[ "$#" == 2 && "$1" == -d && "$2" == "$CASE/"* ]] || exit 95
     exec /usr/bin/mktemp "$@" ;;
esac
SH

cat >"$fixture_root/bin/cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
[[ ! ${BULLET_VERIFIER_FIXTURE_FD+x} ]] || exit 96
[[ "$#" == 8 || "$#" == 10 ]] || exit 97
[[ "$1 $2 $3 $4 $6 $8" == 'build --locked -q -p --bin --message-format=json-render-diagnostics' ]] || exit 98
package="$5"
binary="$7"
feature="${10:-}"
[[ "$#" == 8 || "$9" == --features ]] || exit 99
case "$binary:$package:$feature" in
  bullet-gitd:bullet-gitd:|bullet-gitd-fixture:bullet-gitd:fixture-authority)
    [[ "$PWD" == "$CASE/family/bullet-git" ]] || exit 100 ;;
  bullet-farmd:bullet-farmd:|transaction_demo:bullet:|bullet-verifier-fixture:bullet-verifier:fixture-executor)
    [[ "$PWD" == "$CASE/family/bullet-kernel" ]] || exit 101 ;;
  *) exit 102 ;;
esac
[[ "$(stat -c '%u:%a' "$CARGO_TARGET_DIR")" == "$(id -u):700" ]] || exit 103
printf '%s\t%s\t%s\t%s\n' "$PWD" "$package" "$binary" "$feature" >>"$CASE/calls"
mkdir -p "$CARGO_TARGET_DIR/selected output"
artifact="$CARGO_TARGET_DIR/selected output/$binary"
if [[ "$binary" == transaction_demo ]]; then
  cp "$CASE/transaction-fixture" "$artifact"
else
  printf '#!/usr/bin/env bash\n# exact fixture bytes: %s\nexit 0\n' "$binary" >"$artifact"
fi
chmod 700 "$artifact"
if [[ "$binary" == transaction_demo && "$VARIANT" == build-failed ]]; then
  printf 'raw failed compiler output\n'
  printf 'raw failed compiler stderr\n' >&2
  exit 42
fi
if [[ "$binary" == transaction_demo ]]; then
  case "$VARIANT" in
    missing-file) rm "$artifact" ;;
    symlink-artifact) mv "$artifact" "$CASE/outside"; ln -s "$CASE/outside" "$artifact" ;;
    outside-artifact) mv "$artifact" "$CASE/outside"; artifact="$CASE/outside" ;;
    non-executable) chmod 600 "$artifact" ;;
    writable-artifact) chmod 722 "$artifact" ;;
  esac
fi
row="$(jq -cn --arg name "$binary" --arg src "$PWD/src/$binary.rs" \
  --arg path "$artifact" --arg feature "$feature" \
  '{reason:"compiler-artifact",package_id:"fixture",target:{name:$name,kind:["bin"],src_path:$src},
    profile:{test:false},features:(if $feature == "" then [] else [$feature] end),executable:$path}')"
if [[ "$binary" == transaction_demo ]]; then
  case "$VARIANT" in
    wrong-kind) row="$(jq -c '.target.kind=["lib"]' <<<"$row")" ;;
    wrong-name) row="$(jq -c '.target.name="other"' <<<"$row")" ;;
    wrong-source) row="$(jq -c '.target.src_path="/outside/src.rs"' <<<"$row")" ;;
    test-artifact) row="$(jq -c '.profile.test=true' <<<"$row")" ;;
    unexpected-feature) row="$(jq -c '.features=["fixture-authority"]' <<<"$row")" ;;
    null-executable) row="$(jq -c '.executable=null' <<<"$row")" ;;
    control-path) row="$(jq -c '.executable+="\n"' <<<"$row")" ;;
    empty-stream) exit 0 ;;
    malformed) printf 'not json\n'; exit 0 ;;
    duplicate) printf '%s\n' "$row" ;;
    missing-artifact) row='{"reason":"compiler-message"}' ;;
  esac
fi
if [[ "$binary" == bullet-verifier-fixture && "$VARIANT" == missing-feature ]]; then
  row="$(jq -c '.features=[]' <<<"$row")"
fi
if [[ "$binary" == bullet-verifier-fixture ]]; then
  case "$VARIANT" in
    later-build-drift) printf '# later build drift\n' >>"$CARGO_TARGET_DIR/selected output/bullet-gitd" ;;
    later-build-replaced)
      cp "$CARGO_TARGET_DIR/selected output/bullet-gitd" "$CASE/replacement"
      mv "$CASE/replacement" "$CARGO_TARGET_DIR/selected output/bullet-gitd" ;;
  esac
fi
printf '%s\n' "$row"
if [[ "$binary" == transaction_demo && "$VARIANT" == missing-finish ]]; then exit 0; fi
if [[ "$binary" == transaction_demo && "$VARIANT" == failed-finish ]]; then
  printf '{"reason":"build-finished","success":false}\n'
else
  printf '{"reason":"build-finished","success":true}\n'
fi
if [[ "$binary" == transaction_demo && "$VARIANT" == trailing-record ]]; then
  printf '{"reason":"compiler-message"}\n'
fi
SH
chmod 700 "$fixture_root/bin/cargo" "$fixture_root/bin/mktemp"

case_count=0
run_case() {
  local variant="$1" expected="$2" selected="${3:-explicit}" data_selection="${4:-explicit}" status=0
  local case_dir="$fixture_root/case $variant" reports target data
  mkdir -m 700 "$case_dir"
  mkdir -m 700 "$case_dir/family" "$case_dir/tmp" "$case_dir/target" "$case_dir/data"
  for member in bullet-farm bullet-kernel bullet-git bullet-portal; do
    mkdir -p "$case_dir/family/$member/scripts" "$case_dir/family/$member/target/debug"
    touch "$case_dir/family/$member/Cargo.toml"
    for binary in bullet-gitd bullet-gitd-fixture bullet-farmd bullet-verifier-fixture transaction_demo; do
      # shellcheck disable=SC2016 # CASE belongs to the launched sentinel process.
      printf '#!/usr/bin/env bash\nprintf stale >"$CASE/stale-executed"\nexit 89\n' \
        >"$case_dir/family/$member/target/debug/$binary"
      chmod 700 "$case_dir/family/$member/target/debug/$binary"
    done
  done
  cp "$HUB/scripts/demo.sh" "$case_dir/family/bullet-farm/scripts/demo.sh"
  printf 'preserved data\n' >"$case_dir/data/keep"
  cat >"$case_dir/transaction-fixture" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
[[ "$PWD" == "$CASE/family/bullet-kernel" ]] || exit 110
[[ "$0" == "$CARGO_TARGET_DIR/selected output/transaction_demo" ]] || exit 111
[[ "$BULLET_FARMD_BIN" == "$CARGO_TARGET_DIR/selected output/bullet-farmd" ]] || exit 112
[[ ! ${BULLET_VERIFIER_FIXTURE_BIN+x} ]] || exit 113
[[ "$BULLET_VERIFIER_FIXTURE_FD" =~ ^[1-9][0-9]*$ && "$BULLET_VERIFIER_FIXTURE_FD" -gt 2 ]] || exit 114
fd_path="/proc/self/fd/$BULLET_VERIFIER_FIXTURE_FD"
staged="$(readlink -e "$fd_path")"
[[ "$staged" == "$CASE/tmp/verifier."*/bullet-verifier-fixture ]] || exit 115
[[ "$(stat -Lc '%u:%h' "$fd_path")" == "$(id -u):1" ]] || exit 116
[[ "$(stat -c '%u:%a' "$(dirname "$staged")")" == "$(id -u):700" ]] || exit 117
cmp "$fd_path" "$CARGO_TARGET_DIR/selected output/bullet-verifier-fixture"
[[ "$(sha256sum "$fd_path" | cut -d' ' -f1)" == "$BULLET_VERIFIER_FIXTURE_SHA256" ]] || exit 118
[[ "$BULLET_GITD_BIN" == "$CARGO_TARGET_DIR/selected output/bullet-gitd" \
  && "$BULLET_GITD_FIXTURE_BIN" == "$CARGO_TARGET_DIR/selected output/bullet-gitd-fixture" ]] || exit 119
[[ "$(sha256sum "$BULLET_GITD_BIN" | cut -d' ' -f1)" == "$BULLET_GITD_SHA256" \
  && "$(sha256sum "$BULLET_GITD_FIXTURE_BIN" | cut -d' ' -f1)" == "$BULLET_GITD_FIXTURE_SHA256" ]] || exit 120
[[ "$(stat -c '%u:%a' "$BULLET_DATA_DIR")" == "$(id -u):700" ]] || exit 121
printf '%s\n' "$BULLET_DATA_DIR" >"$CASE/data-used"
printf 'fixture transaction executed\n'
if [[ "$VARIANT" == component-drift || "$VARIANT" == component-failed-drift ]]; then
  printf '# component artifact drift\n' >>"$BULLET_FARMD_BIN"
fi
if [[ "$VARIANT" == component-failed || "$VARIANT" == component-failed-drift ]]; then
  printf 'raw component failure\n' >&2
  exit 37
fi
printf '{"evidence_class":"COMPONENT_PROOF"}\n' >"$BULLET_DATA_DIR/fixture-receipt.json"
SH
  chmod 700 "$case_dir/transaction-fixture"
  target="$case_dir/target"
  case "$selected" in
    relative) target=relative-target ;;
    empty) target='' ;;
    missing) target="$case_dir/missing" ;;
    root) target=/ ;;
    symlink) ln -s "$target" "$case_dir/alias"; target="$case_dir/alias" ;;
    mode) chmod 775 "$target" ;;
    family) target="$case_dir/family/bullet-kernel/target"; chmod 700 "$target" ;;
  esac
  data="$case_dir/data"
  case "$data_selection" in
    relative) data=relative-data ;;
    missing) data="$case_dir/missing-data" ;;
    file) touch "$case_dir/data-file"; data="$case_dir/data-file" ;;
    symlink) ln -s "$data" "$case_dir/data-alias"; data="$case_dir/data-alias" ;;
    mode) mkdir -m 775 "$case_dir/writable-data"; data="$case_dir/writable-data" ;;
    ancestor|sticky)
      mkdir -m 700 "$case_dir/unsafe-parent" "$case_dir/unsafe-parent/data"
      chmod 775 "$case_dir/unsafe-parent"
      [[ "$data_selection" != sticky ]] || chmod 1777 "$case_dir/unsafe-parent"
      data="$case_dir/unsafe-parent/data" ;;
    ancestor-symlink)
      ln -s "$case_dir" "$case_dir/parent-alias"
      data="$case_dir/parent-alias/data" ;;
  esac
  local -a invocation=(env -i "PATH=$fixture_root/bin:/usr/bin:/bin" "CASE=$case_dir" \
    "VARIANT=$variant" BULLET_DEMO_PORTAL=0 \
    BULLET_VERIFIER_FIXTURE_FD=999 BULLET_VERIFIER_FIXTURE_BIN=/untrusted)
  [[ "$data_selection" == default ]] || invocation+=("BULLET_DATA_DIR=$data")
  [[ "$selected" == default ]] || invocation+=("CARGO_TARGET_DIR=$target")
  "${invocation[@]}" bash "$case_dir/family/bullet-farm/scripts/demo.sh" \
    >"$case_dir/stdout" 2>"$case_dir/stderr" || status="$?"
  if [[ "$status" != "$expected" ]]; then
    cat "$case_dir/stdout" "$case_dir/stderr" >&2
    printf 'demo wrapper case %s: status %s, expected %s\n' "$variant" "$status" "$expected" >&2
    exit 1
  fi
  [[ ! -e "$case_dir/stale-executed" && "$(<"$case_dir/data/keep")" == 'preserved data' ]]
  if [[ "$variant" == data-* && "$expected" != 0 ]]; then
    [[ ! -e "$case_dir/calls" && ! -e "$case_dir/data-used" ]]
    grep -Fq DEMO_DATA_INVALID "$case_dir/stderr"
  elif [[ "$variant" == target-* ]]; then
    [[ ! -e "$case_dir/calls" && ! -e "$case_dir/data-used" ]]
    grep -Fq DEMO_TARGET_INVALID "$case_dir/stderr"
  else
    reports="$(sed -n 's/^retained build reports: //p' "$case_dir/stdout")"
    [[ "$reports" == "$case_dir/"* && -d "$reports" ]]
    [[ "$(stat -c '%u:%a' "$reports")" == "$(id -u):700" ]]
    if [[ "$expected" == 0 || "$variant" == component-* ]]; then
      [[ "$(wc -l <"$case_dir/calls")" == 5 && -s "$case_dir/data-used" ]]
      jq -es 'length == 5 and ([.[].binary] == ["bullet-gitd","bullet-gitd-fixture","bullet-farmd","transaction_demo","bullet-verifier-fixture"])' \
        "$reports/binary-subjects.jsonl" >/dev/null
      grep -Fq 'fixture transaction executed' "$reports/transaction_demo.stdout"
      if [[ "$variant" == component-drift ]]; then
        grep -Fq 'DEMO_BUILD_SUBJECT_CHANGED: bullet-farmd' "$case_dir/stderr"
      fi
    else
      [[ ! -e "$case_dir/data-used" ]]
      if [[ "$variant" == build-failed ]]; then
        grep -Fxq 'raw failed compiler output' "$reports/transaction_demo.jsonl"
        grep -Fxq 'raw failed compiler stderr' "$reports/transaction_demo.stderr"
      elif [[ "$variant" == later-build-* ]]; then
        [[ "$(wc -l <"$case_dir/calls")" == 5 ]]
        grep -Fq 'DEMO_BUILD_SUBJECT_CHANGED: bullet-gitd' "$case_dir/stderr"
        if [[ "$variant" == later-build-replaced ]]; then
          [[ "$(sha256sum "$target/selected output/bullet-gitd" | cut -d' ' -f1)" \
            == "$(jq -r 'select(.binary=="bullet-gitd") | .sha256' "$reports/binary-subjects.jsonl")" ]]
        fi
      else
        grep -Fq DEMO_BUILD_ARTIFACT_INVALID "$case_dir/stderr"
      fi
    fi
    if [[ "$expected" == 0 ]]; then
      grep -Fq 'component demo complete (TRANSACTION_PROOF remains absent)' "$case_dir/stdout"
      grep -Fq 'release_gate_eligible: false' "$case_dir/stdout"
    else
      if grep -Fq 'component demo complete' "$case_dir/stdout"; then exit 1; fi
    fi
    if [[ "$variant" == component-failed || "$variant" == component-failed-drift ]]; then
      grep -Fxq 'raw component failure' "$reports/transaction_demo.stderr"
      grep -Fq DEMO_COMPONENT_FAILED "$case_dir/stderr"
      if grep -Fq DEMO_BUILD_SUBJECT_CHANGED "$case_dir/stderr"; then exit 1; fi
    fi
  fi
  case_count=$((case_count + 1))
}

run_case explicit 0
run_case default 0 default
run_case data-default 0 explicit default
for data_selection in relative missing file symlink mode ancestor ancestor-symlink; do
  run_case "data-$data_selection" 1 explicit "$data_selection"
done
if [[ "$(id -u)" == 0 ]]; then
  run_case data-root-sticky 0 explicit sticky
else
  run_case data-user-sticky 1 explicit sticky
fi
for selected in relative empty missing root symlink mode family; do
  run_case "target-$selected" 1 "$selected"
done
for variant in missing-file symlink-artifact outside-artifact non-executable writable-artifact \
  wrong-kind wrong-name wrong-source test-artifact unexpected-feature null-executable control-path \
  empty-stream malformed duplicate missing-artifact missing-feature missing-finish failed-finish trailing-record; do
  run_case "$variant" 1
done
run_case build-failed 42
run_case component-failed 37
run_case later-build-drift 1
run_case later-build-replaced 1
run_case component-drift 1
run_case component-failed-drift 37
[[ "$case_count" == 44 ]]
printf 'demo build wrapper fixtures: %s passed\n' "$case_count"
