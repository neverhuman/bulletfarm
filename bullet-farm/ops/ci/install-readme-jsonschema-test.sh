#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/install-readme-jsonschema.sh
source "$(dirname "${BASH_SOURCE[0]}")/install-readme-jsonschema.sh"

scratch="$(/usr/bin/mktemp -d /tmp/bullet-jsonschema-installer-test.XXXXXX)"
readonly scratch
cleanup() {
  [[ "$scratch" == /tmp/bullet-jsonschema-installer-test.* && -d "$scratch" \
    && ! -L "$scratch" ]] || return 1
  /usr/bin/rm -rf --one-file-system -- "$scratch"
}
trap cleanup EXIT

baseline="$scratch/baseline"
case_root="$scratch/case"
/usr/bin/mkdir -p "$baseline"
/usr/bin/cp -a --reflink=auto "$README_JSONSCHEMA_WHEELHOUSE/." "$baseline/"
/usr/bin/cp -a --reflink=auto "$README_JSONSCHEMA_REQUIREMENTS" "$scratch/requirements.txt"

reset_case() {
  /usr/bin/rm -rf --one-file-system -- "$case_root"
  /usr/bin/mkdir -p "$case_root"
  /usr/bin/cp -a --reflink=auto "$baseline/." "$case_root/"
}

expect_failure() {
  local label="$1"
  shift
  if "$@" >"$scratch/$label.log" 2>&1; then
    printf 'install-readme-jsonschema-test: expected refusal for %s\n' "$label" >&2
    exit 1
  fi
}

validate_case() {
  readme_jsonschema_validate_wheelhouse "$case_root" "$scratch/requirements.txt"
}

reset_case
validate_case

captured_python_sha256="$README_JSONSCHEMA_PYTHON_SHA256"
README_JSONSCHEMA_PYTHON_SHA256=0000000000000000000000000000000000000000000000000000000000000000
expect_failure interpreter-digest validate_case
[[ "$README_JSONSCHEMA_PYTHON_SHA256" == \
  0000000000000000000000000000000000000000000000000000000000000000 ]] || {
  echo 'install-readme-jsonschema-test: interpreter baseline was overwritten' >&2
  exit 1
}
README_JSONSCHEMA_PYTHON_SHA256="$captured_python_sha256"

reset_case
: >"$case_root/unexpected.whl"
expect_failure extra validate_case

reset_case
/usr/bin/rm -- "$case_root/manifest-v1.json"
expect_failure missing validate_case

reset_case
printf 'x' >>"$case_root/attrs-25.4.0-py3-none-any.whl"
expect_failure digest validate_case

reset_case
/usr/bin/cp -- "$case_root/attrs-25.4.0-py3-none-any.whl" "$scratch/outside-wheel"
/usr/bin/rm -- "$case_root/attrs-25.4.0-py3-none-any.whl"
/usr/bin/ln -s -- "$scratch/outside-wheel" "$case_root/attrs-25.4.0-py3-none-any.whl"
expect_failure symlink validate_case

reset_case
/usr/bin/cp -- "$case_root/attrs-25.4.0-py3-none-any.whl" "$scratch/outside-hardlink"
/usr/bin/rm -- "$case_root/attrs-25.4.0-py3-none-any.whl"
/usr/bin/ln -- "$scratch/outside-hardlink" "$case_root/attrs-25.4.0-py3-none-any.whl"
expect_failure hardlink validate_case
/usr/bin/rm -- "$scratch/outside-hardlink"

reset_case
/usr/bin/rm -- "$case_root/attrs-25.4.0-py3-none-any.whl"
/usr/bin/mkfifo -- "$case_root/attrs-25.4.0-py3-none-any.whl"
expect_failure fifo validate_case

reset_case
/usr/bin/chmod 0666 "$case_root/attrs-25.4.0-py3-none-any.whl"
expect_failure mode validate_case

reset_case
/usr/bin/mv -- "$case_root/rpds_py-0.30.0-cp312-cp312-manylinux_2_17_x86_64.manylinux2014_x86_64.whl" \
  "$case_root/rpds_py-0.30.0-cp312-cp312-manylinux_2_17_aarch64.manylinux2014_aarch64.whl"
expect_failure platform-wheel validate_case

reset_case
printf '{"unknown":true}\n' >"$case_root/manifest-v1.json"
preserve_tools="$scratch/preserve-tools"
/usr/bin/mkdir -p "$preserve_tools/readme-jsonschema"
printf 'preserve-me\n' >"$preserve_tools/readme-jsonschema/marker"
expect_failure preflight-preserves install_readme_jsonschema_from \
  "$case_root" "$scratch/requirements.txt" "$preserve_tools"
[[ "$(<"$preserve_tools/readme-jsonschema/marker")" == preserve-me ]] || {
  echo 'install-readme-jsonschema-test: refused preflight changed the prior venv' >&2
  exit 1
}

reset_case
tools="$scratch/tools"
install_log="$scratch/offline-install.log"
caller_umask="$(umask)"
if ! PIP_CONFIG_FILE="$scratch/poison-config" \
  PIP_INDEX_URL=http://127.0.0.1:9/simple \
  PIP_REQUIREMENT="$scratch/absent-requirement" \
  PYTHONPATH="$scratch/poison-pythonpath" \
  PATH=/definitely-missing \
  install_readme_jsonschema_from "$case_root" "$scratch/requirements.txt" "$tools" \
  >"$install_log" 2>&1; then
  /usr/bin/cat "$install_log" >&2
  exit 1
fi
[[ "$(umask)" == "$caller_umask" ]] || {
  echo 'install-readme-jsonschema-test: installer changed the caller umask' >&2
  exit 1
}
if /usr/bin/grep -E 'https?://|127\.0\.0\.1' "$install_log"; then
  echo 'install-readme-jsonschema-test: offline install consulted a network location' >&2
  exit 1
fi
private_home="$scratch/check-home"
/usr/bin/mkdir -m 0700 "$private_home"
venv="$tools/readme-jsonschema"
report="$venv/.bullet-install-report.json"
readme_jsonschema_validate_installed "$venv" "$report" "$case_root" "$private_home"

/usr/bin/cp -- "$report" "$scratch/report.safe"
printf 'x' >>"$report"
expect_failure report-tamper readme_jsonschema_validate_installed \
  "$venv" "$report" "$case_root" "$private_home"
/usr/bin/cp -- "$scratch/report.safe" "$report"

site="$venv/lib/python3.12/site-packages"
attrs_source="$site/attr/__init__.py"
/usr/bin/cp -- "$attrs_source" "$scratch/attrs.safe"
printf '# mutation\n' >>"$attrs_source"
expect_failure record-tamper readme_jsonschema_validate_installed \
  "$venv" "$report" "$case_root" "$private_home"
/usr/bin/cp -- "$scratch/attrs.safe" "$attrs_source"

printf 'unrecorded\n' >"$site/unrecorded.txt"
expect_failure unrecorded readme_jsonschema_validate_installed \
  "$venv" "$report" "$case_root" "$private_home"
/usr/bin/rm -- "$site/unrecorded.txt"
readme_jsonschema_validate_installed "$venv" "$report" "$case_root" "$private_home"

printf '#!/usr/bin/env bash\nexit 99\n' >"$venv/bin/cargo"
/usr/bin/chmod 0700 "$venv/bin/cargo"
expect_failure executable-shadow readme_jsonschema_validate_installed \
  "$venv" "$report" "$case_root" "$private_home"
/usr/bin/rm -- "$venv/bin/cargo"

/usr/bin/rm -- "$venv/bin/python"
/usr/bin/ln -s -- /usr/bin/true "$venv/bin/python"
expect_failure python-link-substitution readme_jsonschema_validate_installed \
  "$venv" "$report" "$case_root" "$private_home"
/usr/bin/rm -- "$venv/bin/python"
/usr/bin/ln -s -- python3.12 "$venv/bin/python"
readme_jsonschema_validate_installed "$venv" "$report" "$case_root" "$private_home"

/usr/bin/chmod 0777 "$venv/bin"
expect_failure writable-bin-directory readme_jsonschema_validate_installed \
  "$venv" "$report" "$case_root" "$private_home"
/usr/bin/chmod 0700 "$venv/bin"

/usr/bin/chmod 0777 "$venv/bin/jsonschema"
expect_failure writable-jsonschema-cli readme_jsonschema_validate_installed \
  "$venv" "$report" "$case_root" "$private_home"
/usr/bin/chmod 0700 "$venv/bin/jsonschema"
readme_jsonschema_validate_installed "$venv" "$report" "$case_root" "$private_home"

grep -Fq -- '--no-index --no-cache-dir --only-binary=:all: --require-hashes --no-compile' \
  ops/ci/install-readme-jsonschema.sh
printf 'install-readme-jsonschema-test: PASS\n'
