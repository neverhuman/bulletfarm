#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fixture="$(mktemp -d)"
trap 'rm -rf -- "$fixture"' EXIT
failures=0

# Execute the real wrapper and custody helpers, with deliberately failing
# doctor/lane/observer children. These are status-routing component fixtures,
# not real lane execution or complete diagnostic-artifact custody evidence.
mkdir -p "$fixture/template/scripts" "$fixture/template/ops/ci"
cp "$REPO_ROOT/scripts/ci-local.sh" "$fixture/template/scripts/ci-local.sh"
for helper in artifact-path family-custody scratch-floor; do
  cp "$REPO_ROOT/ops/ci/$helper.sh" "$fixture/template/ops/ci/$helper.sh"
done
cat >"$fixture/template/scripts/ci-doctor.sh" <<'CHILD'
#!/usr/bin/env bash
set -eu
printf 'doctor\n' >> doctor-started
printf 'doctor-native-status=%s\n' "$FIXTURE_DOCTOR"
exit "$FIXTURE_DOCTOR"
CHILD
cat >"$fixture/template/ops/ci/fast.sh" <<'CHILD'
#!/usr/bin/env bash
set -eu
printf 'lane\n' >> lane-started
printf 'lane-native-status=%s\n' "$FIXTURE_LANE"
if [[ "$FIXTURE_TAMPER" == lane ]]; then
  printf 'replaced-owner\n' > .git/bullet-ci.lock.d/owner
fi
exit "$FIXTURE_LANE"
CHILD
cat >"$fixture/template/scripts/ci-observation.sh" <<'CHILD'
#!/usr/bin/env bash
set -eu
[[ "$1" == fast ]]
printf 'observer\n' >> observer-started
printf '%s\n' "$2" > observed-primary
printf 'observer-native-status=%s primary=%s\n' "$FIXTURE_OBSERVER" "$2"
if [[ "$FIXTURE_TAMPER" == observer ]]; then
  printf 'replaced-owner\n' > .git/bullet-ci.lock.d/owner
fi
exit "$FIXTURE_OBSERVER"
CHILD

# name doctor lane observer tamper final observer_called observed_primary
while read -r name doctor lane observer tamper expected called primary; do
  case_root="$fixture/$name"
  cp -R "$fixture/template" "$case_root"
  mkdir "$case_root/.git"
  printf 'ref: refs/heads/main\n' > "$case_root/.git/HEAD"
  set +e
  (
    cd "$case_root" || exit 97
    FIXTURE_DOCTOR="$doctor" FIXTURE_LANE="$lane" \
      FIXTURE_OBSERVER="$observer" FIXTURE_TAMPER="$tamper" \
      bash scripts/ci-local.sh fast
  ) > "$case_root/stdout" 2> "$case_root/stderr"
  status=$?
  set -e
  valid=1
  [[ "$status" -eq "$expected" ]] || valid=0
  [[ -f "$case_root/doctor-started" \
    && "$(<"$case_root/doctor-started")" == doctor \
    && "$(wc -l < "$case_root/doctor-started")" -eq 1 ]] || valid=0
  if [[ "$doctor" -eq 0 ]]; then
    [[ -f "$case_root/lane-started" \
      && "$(<"$case_root/lane-started")" == lane \
      && "$(wc -l < "$case_root/lane-started")" -eq 1 ]] || valid=0
  else
    [[ ! -e "$case_root/lane-started" ]] || valid=0
  fi
  if [[ "$called" -eq 1 ]]; then
    [[ -f "$case_root/observer-started" \
      && "$(<"$case_root/observer-started")" == observer \
      && "$(wc -l < "$case_root/observer-started")" -eq 1 \
      && -f "$case_root/observed-primary" \
      && "$(<"$case_root/observed-primary")" == "$primary" ]] || valid=0
  else
    [[ ! -e "$case_root/observer-started" \
      && ! -e "$case_root/observed-primary" ]] || valid=0
  fi
  diagnostics="$(<"$case_root/stderr")"
  if [[ "$tamper" == none ]]; then
    [[ ! -e "$case_root/.git/bullet-ci.lock.d" ]] || valid=0
  else
    [[ -f "$case_root/.git/bullet-ci.lock.d/owner" \
      && "$(<"$case_root/.git/bullet-ci.lock.d/owner")" == replaced-owner \
      && "$diagnostics" == *CI_PROOF_LOCKED_OR_STALE* ]] || valid=0
  fi
  if [[ "$tamper" == lane ]]; then
    [[ "$diagnostics" == *"stage=post-lane-custody primary=$primary secondary=75"* ]] || valid=0
  fi
  if [[ "$called" -eq 1 && "$observer" -ne 0 ]]; then
    [[ "$diagnostics" == *"stage=observation primary=$primary secondary=$observer"* ]] || valid=0
  fi
  if [[ "$tamper" != none ]]; then
    release_primary="$primary"
    if [[ "$release_primary" -eq 0 ]]; then
      if [[ "$tamper" == lane ]]; then release_primary=75; else release_primary="$observer"; fi
    fi
    [[ "$diagnostics" == *"stage=release primary=$release_primary secondary=75"* ]] || valid=0
  fi
  printf '[ci] OBSERVED_STATUS_CASE %s expected=%s actual=%s valid=%s\n' \
    "$name" "$expected" "$status" "$valid"
  cat "$case_root/stdout"
  cat "$case_root/stderr" >&2
  [[ "$valid" -eq 1 ]] || failures=$((failures + 1))
done <<'CASES'
success 0 0 0 none 0 1 0
lane_failure 0 19 0 none 19 1 19
doctor_failure 17 0 0 none 17 1 17
observation_failure 0 0 23 none 23 1 0
lane_and_observation 0 19 23 none 19 1 19
doctor_and_observation 17 0 23 none 17 1 17
successful_lane_lost_custody 0 0 0 lane 75 0 0
failed_lane_lost_custody 0 19 0 lane 19 0 19
successful_lane_release_failure 0 0 0 observer 75 1 0
failed_lane_release_failure 0 19 0 observer 19 1 19
observation_and_release_failure 0 0 23 observer 23 1 0
all_failures 0 19 23 observer 19 1 19
CASES
[[ "$failures" -eq 0 ]] || {
  printf '[ci] OBSERVED_STATUS_FAILURES: %s\n' "$failures" >&2
  exit 1
}
printf '[ci] observed status precedence: 12 cases passed\n'
