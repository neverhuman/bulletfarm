#!/usr/bin/env bash
set -euo pipefail

# shellcheck source=ops/ci/readme-jsonschema-custody.sh
source "$(dirname "${BASH_SOURCE[0]}")/readme-jsonschema-custody.sh"

readme_jsonschema_safe_remove() {
  local path="$1" tools_root="$2" name
  name="${path##*/}"
  [[ "$tools_root" == /* && "$tools_root" != / && "${path%/*}" == "$tools_root" \
    && ("$name" == readme-jsonschema || "$name" == .readme-jsonschema-*) ]] || {
    refuse JSONSCHEMA_CLEANUP_SCOPE_INVALID "$path"
    return 1
  }
  [[ ! -e "$path" && ! -L "$path" ]] || /usr/bin/rm -rf --one-file-system -- "$path"
}

readme_jsonschema_prepare_tools_root() {
  local tools_root="$1" target_root="${1%/*}" path
  [[ "$tools_root" == /* && "$tools_root" != / && "$target_root" != / ]] || {
    refuse JSONSCHEMA_TOOL_ROOT_INVALID "$tools_root"
    return 1
  }
  for path in "$target_root" "$tools_root"; do
    [[ ! -L "$path" ]] || {
      refuse JSONSCHEMA_TOOL_ROOT_INVALID "$path"
      return 1
    }
  done
  /usr/bin/mkdir -p -- "$tools_root"
  readme_jsonschema_require_directory "$target_root" tool-target || return 1
  readme_jsonschema_require_directory "$tools_root" tool-root
}

readme_jsonschema_build_environment() {
  local wheelhouse="$1" requirements="$2" venv="$3" private_home="$4"
  local report="$venv/.bullet-install-report.json" pip_version
  /usr/bin/env -i HOME="$private_home" PATH=/usr/bin:/bin LC_ALL=C TZ=UTC \
    PYTHONDONTWRITEBYTECODE=1 \
    "$README_JSONSCHEMA_PYTHON" -I -B -m venv --without-pip "$venv" || return 1
  [[ -d "$venv" && ! -L "$venv" ]] || {
    refuse JSONSCHEMA_VENV_CREATION_FAILED "$venv"
    return 1
  }
  pip_version="$(readme_jsonschema_run_pip "$venv/bin/python3" "$private_home" \
    "$wheelhouse/pip-26.2.1-py3-none-any.whl" \
    --isolated --disable-pip-version-check --no-input --version)" || return 1
  [[ "$pip_version" == pip\ 26.2.1\ from\ *\ \(python\ 3.12\) ]] || {
    refuse JSONSCHEMA_PIP_VERSION_INVALID "$pip_version"
    return 1
  }
  readme_jsonschema_run_pip "$venv/bin/python3" "$private_home" \
    "$wheelhouse/pip-26.2.1-py3-none-any.whl" \
    --isolated --disable-pip-version-check --no-input install \
    --no-index --no-cache-dir --only-binary=:all: --require-hashes --no-compile \
    --find-links "$wheelhouse" --report "$report" --requirement "$requirements" || return 1
  readme_jsonschema_validate_wheelhouse "$wheelhouse" "$requirements" || return 1
  readme_jsonschema_validate_installed "$venv" "$report" "$wheelhouse" "$private_home"
}

install_readme_jsonschema_from() {
  local wheelhouse="$1" requirements="$2" tools_root="$3"
  local venv="$tools_root/readme-jsonschema" backup="$tools_root/.readme-jsonschema-previous"
  local private_home status old_present=false
  readme_jsonschema_validate_wheelhouse "$wheelhouse" "$requirements" || return 1
  readme_jsonschema_prepare_tools_root "$tools_root" || return 1
  [[ ! -L "$venv" && (! -e "$venv" || -d "$venv") ]] || {
    refuse JSONSCHEMA_VENV_SUBJECT_INVALID "$venv"
    return 1
  }
  [[ ! -e "$backup" && ! -L "$backup" ]] || {
    refuse JSONSCHEMA_BACKUP_COLLISION "$backup"
    return 1
  }
  private_home="$(/usr/bin/mktemp -d "$tools_root/.readme-jsonschema-home.XXXXXX")" \
    || return 1
  /usr/bin/chmod 0700 "$private_home"

  if [[ -d "$venv" \
    && -f "$venv/.bullet-install-report.json" \
    && ! -L "$venv/.bullet-install-report.json" ]] \
    && readme_jsonschema_validate_installed "$venv" \
      "$venv/.bullet-install-report.json" "$wheelhouse" "$private_home"; then
    readme_jsonschema_safe_remove "$private_home" "$tools_root"
    log "README jsonschema validator already matches the admitted offline subject"
    return 0
  fi

  if [[ -d "$venv" ]]; then
    /usr/bin/mv -- "$venv" "$backup"
    old_present=true
  fi
  if (umask 077 && readme_jsonschema_build_environment \
    "$wheelhouse" "$requirements" "$venv" "$private_home"); then
    status=0
  else
    status=$?
  fi
  if [[ "$status" -ne 0 ]]; then
    readme_jsonschema_safe_remove "$venv" "$tools_root" || return 1
    if [[ "$old_present" == true ]]; then
      /usr/bin/mv -- "$backup" "$venv" || return 1
    fi
    readme_jsonschema_safe_remove "$private_home" "$tools_root" || return 1
    refuse JSONSCHEMA_OFFLINE_INSTALL_FAILED "status=$status; prior environment restored"
    return "$status"
  fi
  if [[ "$old_present" == true ]]; then
    readme_jsonschema_safe_remove "$backup" "$tools_root" || return 1
  fi
  readme_jsonschema_safe_remove "$private_home" "$tools_root" || return 1
  readme_jsonschema_validate_wheelhouse "$wheelhouse" "$requirements" || return 1
  log "installed admitted offline validator at $venv/bin/python3"
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  [[ "$#" -eq 0 ]] || {
    refuse JSONSCHEMA_ARGUMENT_INVALID no-arguments
    exit 2
  }
  install_readme_jsonschema_from \
    "$README_JSONSCHEMA_WHEELHOUSE" "$README_JSONSCHEMA_REQUIREMENTS" \
    "$REPO_ROOT/target/.ci-tools"
fi
