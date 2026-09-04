#!/usr/bin/env bash
set -euo pipefail

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAMILY="$(cd "$HUB/.." && pwd)"
MEDIA="$HUB/docs/readme-media"
SNAPSHOT="$MEDIA/snapshot.json"

for tool in cargo git jq python3 sudo unshare; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'readme-record: missing required tool %s\n' "$tool" >&2
    exit 1
  }
done
sudo -n true 2>/dev/null || {
  echo "readme-record: passwordless sudo is required only to create the network namespace" >&2
  exit 1
}
bash "$HUB/scripts/readme-schema-check.sh" --strict-json "$SNAPSHOT" || {
  echo "readme-record: frozen snapshot is not strict JSON" >&2
  exit 1
}
jq -e '
  .schema_version == "bullet.readme-snapshot.v1"
  and .release_authority == false
  and (.subject_committer_epochs | keys | sort) == (.repositories | keys | sort)
  and all(.subject_committer_epochs[]; type == "number" and . > 0)
  and ((.observed_at | fromdateiso8601) >= ([.subject_committer_epochs[]] | max))
' "$SNAPSHOT" >/dev/null

record_root="$(mktemp -d)"
cleanup() {
  rm -rf "$record_root"
}
trap cleanup EXIT

family_clone="$record_root/family"
empty_home="$record_root/home"
runtime_tmp="$record_root/runtime"
target_dir="$family_clone/target"
demo_data="$record_root/demo-data"
mkdir -p "$family_clone" "$empty_home" "$runtime_tmp" "$target_dir" "$demo_data"

ambient_cargo_home="${CARGO_HOME:-$HOME/.cargo}"
tool_rustup_home="${RUSTUP_HOME:-$HOME/.rustup}"
source_epoch="$(jq -r '.source_date_epoch' "$SNAPSHOT")"

tool_bin="$record_root/tool-bin"
tool_cargo_home="$record_root/cargo-home"
mkdir -p "$tool_bin" "$tool_cargo_home"
recording_tools=(
  ar as awk bash cargo cargo-clippy cargo-fmt cargo-nextest cc chmod cmp comm cp cut date
  dirname env gcc git grep head jq just ld make mkdir mktemp mv ranlib readlink
  realpath rm rustc rustdoc rustfmt sed sha256sum sh sleep sort stat strip touch tr uname wc
)
for tool in "${recording_tools[@]}"; do
  resolved="$(command -v "$tool")" || {
    printf 'readme-record: recording tool allowlist cannot resolve %s\n' "$tool" >&2
    exit 1
  }
  ln -s "$resolved" "$tool_bin/$tool"
done
if [[ -d "$ambient_cargo_home/registry" ]]; then
  ln -s "$ambient_cargo_home/registry" "$tool_cargo_home/registry"
fi
[[ ! -e "$tool_cargo_home/credentials" && ! -e "$tool_cargo_home/credentials.toml" ]]
tool_path="$tool_bin"

clone_subject() {
  local repo="$1"
  local source="$FAMILY/$repo"
  local destination="$family_clone/$repo"
  local commit tree expected_epoch actual_epoch actual_tree
  commit="$(jq -r --arg repo "$repo" '.repositories[$repo].commit_oid' "$SNAPSHOT")"
  tree="$(jq -r --arg repo "$repo" '.repositories[$repo].tree_oid' "$SNAPSHOT")"
  expected_epoch="$(jq -r --arg repo "$repo" '.subject_committer_epochs[$repo]' "$SNAPSHOT")"
  git -C "$source" cat-file -e "$commit^{commit}"
  actual_epoch="$(git -C "$source" show -s --format=%ct "$commit")"
  [[ "$actual_epoch" == "$expected_epoch" ]] || {
    printf 'readme-record: %s committer epoch drift: expected %s, found %s\n' \
      "$repo" "$expected_epoch" "$actual_epoch" >&2
    exit 1
  }
  actual_tree="$(git -C "$source" rev-parse "$commit^{tree}")"
  [[ "$actual_tree" == "$tree" ]] || {
    printf 'readme-record: %s tree drift: expected %s, found %s\n' "$repo" "$tree" "$actual_tree" >&2
    exit 1
  }
  GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null git clone --quiet --no-hardlinks --no-checkout "$source" "$destination"
  GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null git -C "$destination" checkout --quiet --detach "$commit"
  [[ "$(git -C "$destination" rev-parse HEAD)" == "$commit" ]]
  [[ "$(git -C "$destination" rev-parse 'HEAD^{tree}')" == "$tree" ]]
  [[ -z "$(git -C "$destination" status --porcelain=v1 --untracked-files=all)" ]] || {
    printf 'readme-record: extracted %s subject is dirty\n' "$repo" >&2
    exit 1
  }
  if [[ -f "$destination/Cargo.lock" ]] && grep -Eq '^source = "git\+' "$destination/Cargo.lock"; then
    printf 'readme-record: %s frozen lock requires an unadmitted Git dependency cache\n' "$repo" >&2
    exit 1
  fi
}

for repo in bullet-farm bullet-kernel bullet-git bullet-portal; do
  clone_subject "$repo"
done
# The scratch family index comes from the frozen Hub subject, not mutable
# container metadata outside every recorded Git tree.
cp "$family_clone/bullet-farm/repos.manifest.toml" "$family_clone/repos.manifest.toml"

run_isolated() {
  local workdir="$1"
  shift
  local invoking_uid invoking_gid
  invoking_uid="$(id -u)"
  invoking_gid="$(id -g)"
  local -a isolated_env=(
    env -i
    PATH="$tool_path"
    LANG=C.UTF-8
    LC_ALL=C.UTF-8
    TZ=UTC
    SOURCE_DATE_EPOCH="$source_epoch"
    HOME="$empty_home"
    XDG_CONFIG_HOME="$empty_home/.config"
    XDG_CACHE_HOME="$empty_home/.cache"
    XDG_DATA_HOME="$empty_home/.local/share"
    CARGO_HOME="$tool_cargo_home"
    RUSTUP_HOME="$tool_rustup_home"
    CARGO_NET_OFFLINE=true
    CARGO_INCREMENTAL=0
    CARGO_TARGET_DIR="$target_dir"
    TMPDIR="$runtime_tmp"
    BULLET_DATA_DIR="$demo_data"
    BULLET_README_SPAWN_MARKER="$record_root/provider-spawned"
  )
  sudo -n unshare --net --fork --kill-child=KILL \
    --setgid "$invoking_gid" --setuid "$invoking_uid" --wd "$workdir" \
    "${isolated_env[@]}" "$@"
}

show_component_log() {
  local log="$1"
  [[ -f "$log" ]] || return 0
  sed "s#$record_root#<snapshot>#g" "$log" >&2
}

hub_clone="$family_clone/bullet-farm"
kernel_clone="$family_clone/bullet-kernel"
doctor_json="$record_root/doctor.json"
doctor_log="$record_root/doctor.log"
doctor_exit=0
run_isolated "$hub_clone" cargo run --locked --quiet --bin bullet-family -- doctor --json >"$doctor_json" 2>"$doctor_log" || doctor_exit=$?
[[ "$doctor_exit" -eq 3 ]] || {
  echo "readme-record: doctor did not return expected exit 3" >&2
  show_component_log "$doctor_log"
  exit 1
}
jq -e '.status == "BLOCKED"' "$doctor_json" >/dev/null

run_isolated "$hub_clone" bash scripts/ci-local.sh fast >"$record_root/hub-fast.log" 2>&1 || {
  echo "readme-record: Hub component checks failed" >&2
  show_component_log "$record_root/hub-fast.log"
  exit 1
}

run_isolated "$hub_clone" bash scripts/demo.sh >"$record_root/demo.log" 2>&1 || {
  echo "readme-record: deterministic demo failed" >&2
  show_component_log "$record_root/demo.log"
  exit 1
}
demo_receipt="$demo_data/receipts.json"
jq -e '
  .materialize_idempotent == true
  and .stale_refused == true
  and .fence_first == 1
  and .fence_second == 2
  and .effect_unknown_outcome == "unknown"
' "$demo_receipt" >/dev/null
component_lines=(
  'Bullet Farm component preview'
  'doctor                 BLOCKED (exit 3)'
  'hub component checks   PASS'
  'materialize replay     IDEMPOTENT'
  'fence                  1 -> 2'
  'stale authority        REFUSED'
  'lost response effect   UNKNOWN'
)
component_transcript="$record_root/component-transcript.txt"
printf '%s\n' "${component_lines[@]}" >"$component_transcript"

component_observation="$record_root/component-observation.json"
jq -n --slurpfile snapshot "$SNAPSHOT" '
  {
    schema_version: "bullet.readme-demo.v1",
    document_type: "observation",
    demo_id: "component-preview",
    observed_at: $snapshot[0].observed_at,
    classification: "UNSIGNED_COMPONENT_OBSERVATION",
    release_authority: false,
    live_provider_spawned: false,
    network: "isolated",
    subjects: $snapshot[0].repositories,
    outcomes: [
      {name: "doctor", status: "BLOCKED", exit_code: 3},
      {name: "hub-component-checks", status: "PASS", exit_code: 0},
      {name: "materialize-replay", status: "IDEMPOTENT", exit_code: 0},
      {name: "fence", status: "1 -> 2", exit_code: 0},
      {name: "stale-authority", status: "REFUSED", exit_code: 0},
      {name: "lost-response-effect", status: "UNKNOWN", exit_code: 0}
    ]
  }
' >"$component_observation"

provider_packages=(
  bullet-harness-claude
  bullet-harness-codex
  bullet-harness-cursor
  bullet-harness-antigravity
)
for package in "${provider_packages[@]}"; do
  run_isolated "$kernel_clone" cargo test --locked -p "$package" --test offline >"$record_root/$package.log" 2>&1 || {
    printf 'readme-record: %s offline suite failed\n' "$package" >&2
    exit 1
  }
  grep -Eq 'test result: ok\. [1-9][0-9]* passed' "$record_root/$package.log" || {
    printf 'readme-record: %s offline suite executed zero tests\n' "$package" >&2
    exit 1
  }
done

run_isolated "$kernel_clone" cargo build --locked -p bullet --bin bullet >"$record_root/bullet-build.log" 2>&1 || {
  echo "readme-record: provider refusal CLI build failed" >&2
  exit 1
}
bullet_bin="$target_dir/debug/bullet"
[[ -x "$bullet_bin" ]]
marker_executable="$record_root/provider-marker"
provider_path="$record_root/provider-path"
mkdir -p "$provider_path"
printf "#!/bin/sh\n: \"\${BULLET_README_SPAWN_MARKER:?}\"\nprintf \"spawned\\\\n\" >>\"\$BULLET_README_SPAWN_MARKER\"\nexit 99\n" \
  >"$marker_executable"
chmod 700 "$marker_executable"

provider_names=(claude codex cursor agy)
for provider in "${provider_names[@]}"; do
  provider_data="$record_root/provider-data/$provider"
  mkdir -p "$provider_data/policy"
  cp "$hub_clone/policy/v1alpha1/policy.json" "$provider_data/policy/policy.json"
  provider_exit=0
  run_isolated "$kernel_clone" env PATH="$provider_path" "$bullet_bin" provider live-conformance --data-dir "$provider_data" --provider "$provider" --executable "$marker_executable" >"$record_root/provider-$provider.log" 2>&1 || provider_exit=$?
  [[ "$provider_exit" -eq 78 ]] || {
    printf 'readme-record: %s refusal returned %s, expected 78\n' "$provider" "$provider_exit" >&2
    show_component_log "$record_root/provider-$provider.log"
    exit 1
  }
  grep -q 'POLICY_LIVE_ADMISSION_DISABLED' "$record_root/provider-$provider.log" || {
    printf 'readme-record: %s refusal reason missing\n' "$provider" >&2
    show_component_log "$record_root/provider-$provider.log"
    exit 1
  }
  [[ ! -e "$record_root/provider-spawned" ]] || {
    echo "readme-record: a provider marker executable was spawned" >&2
    exit 1
  }
done

provider_lines=(
  'Bullet Farm provider safety'
  'Claude offline contract       PASS'
  'Codex offline contract        PASS'
  'Cursor offline contract       PASS'
  'Antigravity offline contract  PASS'
  'Claude live admission         POLICY_LIVE_ADMISSION_DISABLED'
  'Codex live admission          POLICY_LIVE_ADMISSION_DISABLED'
  'Cursor live admission         POLICY_LIVE_ADMISSION_DISABLED'
  'Antigravity live admission    POLICY_LIVE_ADMISSION_DISABLED'
  'provider processes spawned    0'
  'live provider proof           ABSENT'
)
provider_transcript="$record_root/provider-transcript.txt"
printf '%s\n' "${provider_lines[@]}" >"$provider_transcript"

provider_observation="$record_root/provider-observation.json"
jq -n --slurpfile snapshot "$SNAPSHOT" '
  {
    schema_version: "bullet.readme-demo.v1",
    document_type: "observation",
    demo_id: "provider-safety",
    observed_at: $snapshot[0].observed_at,
    classification: "UNSIGNED_COMPONENT_OBSERVATION",
    release_authority: false,
    live_provider_spawned: false,
    network: "isolated",
    subjects: $snapshot[0].repositories,
    outcomes: [
      {name: "claude-offline-contract", status: "PASS", exit_code: 0},
      {name: "codex-offline-contract", status: "PASS", exit_code: 0},
      {name: "cursor-offline-contract", status: "PASS", exit_code: 0},
      {name: "antigravity-offline-contract", status: "PASS", exit_code: 0},
      {name: "claude-live-admission", status: "POLICY_LIVE_ADMISSION_DISABLED", exit_code: 78},
      {name: "codex-live-admission", status: "POLICY_LIVE_ADMISSION_DISABLED", exit_code: 78},
      {name: "cursor-live-admission", status: "POLICY_LIVE_ADMISSION_DISABLED", exit_code: 78},
      {name: "antigravity-live-admission", status: "POLICY_LIVE_ADMISSION_DISABLED", exit_code: 78},
      {name: "provider-spawn-count", status: "0", exit_code: 0},
      {name: "live-provider-proof", status: "ABSENT", exit_code: 0}
    ]
  }
' >"$provider_observation"

install -m 0644 "$component_transcript" "$MEDIA/component-preview/transcript.txt"
install -m 0644 "$component_observation" "$MEDIA/component-preview/observation.json"
install -m 0644 "$provider_transcript" "$MEDIA/provider-safety/transcript.txt"
install -m 0644 "$provider_observation" "$MEDIA/provider-safety/observation.json"

echo "readme-record: wrote normalized component and provider-safety observations"
