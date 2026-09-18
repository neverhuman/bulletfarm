#!/usr/bin/env bash
# Drive an already-authenticated `bullet tui` inside record-tui.sh's PTY.
# Writes run.json with observation-only identities. Never reads credentials.
set -euo pipefail
umask 077

: "${BULLET_RECORD_RUN_JSON:?}"
: "${BULLET_RECORD_RUN_ID:?}"
: "${BULLET_RECORD_BULLET_BIN:?}"

python3 - "$BULLET_RECORD_RUN_JSON" "$BULLET_RECORD_RUN_ID" <<'PY'
import hashlib, json, sys
path, run_id = sys.argv[1], sys.argv[2]

def ident(prefix, label):
    return prefix + hashlib.sha256(f"bullet.operator-console {run_id} {label}".encode()).hexdigest()

with open(path, "w", encoding="utf-8") as stream:
    json.dump({
        "schema": "bullet.record-run.v1",
        "run_id": run_id,
        "command_id": ident("cmd_", "command"),
        "attempt_id": ident("atm_", "attempt"),
        "candidate_id": None,
        "receipt": {"algorithm": "sha256", "digest": ident("", "receipt")},
        "provider": {"name": "none", "runtime_version": "0.0.0"},
        "claims": ["local-observation", "no-gate-cleared", "real-recording"],
    }, stream, indent=2, sort_keys=True)
    stream.write("\n")
PY

exec python3 -I -- "$(dirname -- "$(realpath -e -- "$0")")/narrate-operator-tui.py"
