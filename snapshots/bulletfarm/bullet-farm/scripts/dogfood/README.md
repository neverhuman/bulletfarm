# Dogfood operator loop

Four wrappers that stand up the loopback farmd command path an operator needs
to drive a `run_coding` command through `bullet-command-worker`, and one that
prepares an immutable Rust toolchain for a future offline gate.

None of them grants authority. None of them spends money. None of them runs a
provider CLI, `bullet dogfood read-only`, or `sudo`. Every input lives under the
operator's own 0700 directories, every secret is written at 0600 and never
printed, and every refusal is typed `DOGFOOD_OPS_<CODE>: <value>` on stderr.

| Script | Role |
| --- | --- |
| `serve.sh` | start/stop one loopback `bullet-farmd`, exchange the one-time bootstrap for a session |
| `prepare-harness.sh` | derive the whole `BULLET_HARNESS_*` set and prove the binding with `bullet coding harness-check` |
| `worker-loop.sh` | build the binary manifest and re-invoke the one-shot `bullet-command-worker` |
| `../stage-rust-toolchain.sh` | stage a root-owned immutable Rust 1.97.1 under `$HOME` and print the operator's `sudo` commands |

## Operator flow

Pick a lane directory outside the family repos and outside `/tmp`. The kernel
refuses proof roots whose ancestors are world-writable, so `/tmp` is unusable:

```sh
LANE="$HOME/bullet-ops-lane-$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -m 0700 "$LANE"
```

### 1. serve

```sh
scripts/dogfood/serve.sh --data-dir "$LANE/data" --bind 127.0.0.1:0 \
  --farmd /abs/path/to/bullet-farmd
```

Admits or creates the 0700 data dir, provisions `custody/lease-transport.key`
once, writes `custody/peer-registry.json` (`farmd_uid`, `socket_gid`, one
runner with `runner_epoch: 1`), launches farmd under `setsid` with 0600 logs,
waits for `/health`, takes the one-time bootstrap token, exchanges it for a
cookie and CSRF token, and writes a 0600 `session.json`. It prints only
`origin`, `farmd`, `pid`, `data_dir`, `session_file`, `bootstrap_path` and
`log`; the token, cookie and CSRF value never reach stdout.

`bootstrap_path=token_file` means the farmd binary lists `--bootstrap-token-file`
and the token travelled through a 0600 file. `bootstrap_path=stdout` means the
binary predates that flag and the token was scraped from its startup line. Both
are supported; the printed value says which one ran.

Stop with `serve.sh --stop --data-dir "$LANE/data"`. It SIGTERMs farmd's whole
process group, escalates to SIGKILL after ten seconds, and removes the pid and
session files.

### 2. prepare-harness

```sh
scripts/dogfood/prepare-harness.sh \
  --data-dir "$LANE/data" \
  --source-repo /home/ubuntu/bullet/bullet-kernel \
  --objective "Read this checkout and satisfy the repository gate" \
  --gate-id gat_8888888888888888888888888888888888888888888888888888888888888888 \
  --scope src \
  --preservation-dir "$LANE/preserve" \
  --dogfood-data-dir /home/ubuntu/bullet-dogfood-2 \
  --dogfood-executable /usr/lib/bullet/providers/claude/2.1.266/bin/claude \
  --credential "$HOME/.claude/.credentials.json,.claude/.credentials.json,<blake3>" \
  --bullet /abs/path/to/bullet --allow-placeholders
```

Writes a 0600 env file, runs `bullet coding harness-check --json` with exactly
that environment, and exits 0 only on `BOUND`. The env file is **not** a shell
script: one `NAME=VALUE` line per binding with a raw value.

Three of the seventeen REQUIRED names have no producer anywhere in the kernel
today and are written as `PLACEHOLDER_DRY_RUN_ONLY:<value>`:

* `BULLET_HARNESS_WORK_PACKAGE_ID` — a Kernel-selected work package. Nothing in
  the operator path materializes one for `run_coding`. The value has the exact
  `WorkPackageId::from_seed` shape (`blake3("wpk:<seed>")`) and no ledger row.
* `BULLET_HARNESS_CANDIDATE_REQUEST_DIGEST` — the digest of a *registered*
  Candidate-preparation source. Registration lives in the ledger; no CLI writes
  a row.
* `BULLET_HARNESS_IDEMPOTENCY_KEY` — the Kernel's pre-acquisition key. The
  operator invents it; it is unrelated to the `run_coding` envelope's own key.

Without `--allow-placeholders` the script refuses so nobody mistakes the env
file for a live binding. Two things are real and derived, not invented:

* `BULLET_HARNESS_CANDIDATE_VERIFICATION_KEY` is written for real. farmd derives
  its Candidate-preparation signing key from the bytes of
  `--lease-transport-key` under the fixed labels `kernel-local` /
  `candidate-preparation-1` (`apps/bullet-farmd/src/main/launch.rs`). A PASETO
  v4.public secret key is `seed || public`, so the record's `public_key_hex` is
  the last 32 bytes of that file. No secret is emitted.
* `BULLET_HARNESS_LEASE_RECOVERY` names an absent file under a 0700 parent;
  `load_recovery` treats an absent record as a fresh journal.

`--credential` is admitted by shape and by the existence of the source only.
Its bytes are never read, so the digest is never recomputed here.

### 3. submit

`bullet coding submit` only sends `run_coding`. For a `run_demo` smoke, POST the
envelope directly, using the cookie and CSRF token from the session file:

```sh
S="$LANE/data/session.json"
curl --silent -b "$(jq -r .cookie "$S")" \
  -H "Origin: $(jq -r .origin "$S")" -H "x-bullet-csrf: $(jq -r .csrf "$S")" \
  -H 'content-type: application/json' \
  --data '{"idempotency_key":"<key>","kind":"run_demo","payload":{}}' \
  "$(jq -r .farmd "$S")/api/v1/commands"
```

A 202 returns `{id, status: "PENDING", kind, payload_digest, result: null}`.
Read it back with `GET /api/v1/commands/<id>` and the same cookie.

### 4. worker-loop

```sh
scripts/dogfood/worker-loop.sh --data-dir "$LANE/data" \
  --manifest "$LANE/command-worker-binaries.json" \
  --worker /abs/bullet-command-worker \
  --transaction-offline /abs/transaction_offline --farmd /abs/bullet-farmd \
  --runner /abs/bullet-runner --gitd /abs/bullet-gitd --verifier /abs/bullet-verifier-fixture \
  --interval 2 --max-idle 2 --env-file "$LANE/data/harness/harness.env"
```

Builds the `bullet.command-worker-binary-manifest.v1` manifest when it is
absent, with the same canonical sorted JSON, no terminal newline and mode 0600
that `bullet-kernel/ops/ci/proof-public-command-component.sh` uses — the worker
compares the file bytes against `canonical_json`, so a stray newline is
`BINARY_MANIFEST_INVALID`. Every subject must be a canonical, self-owned,
non-group/other-writable native ELF; a `target/debug` binary at mode 0775 is
refused, so copy it to a 0700 lane directory rather than changing another
agent's tree.

Each iteration runs the one-shot worker in its own session and logs
`<utc> iter=<n> exit=<code> result=<token> elapsed_ms=<n>` to stdout and to
`<data-dir>/logs/worker-loop-*.log`. `NO_COMMAND` is idle and doubles the sleep
from `--interval` up to eight times it; `--max-idle N` exits 0 after N
consecutive idles. SIGTERM/SIGINT stop the loop only after the running child
finishes on its own — a claim in flight is never killed or restarted. A nonzero
worker exit is a typed refusal, not a transient: the loop prints the worker's
stderr verbatim and stops with exit 1.

`--env-file` is required for `run_coding` and unnecessary for `run_demo`.

### 5. receipts

Worker state is `<data-dir>/worker/current.json`
(`bullet.command-worker-state.v1`: `stage`, `claim`, `binary_manifest_sha256`,
`receipt_sha256`, `receipt_digest`). A settled run leaves its receipt under
`<data-dir>/worker/<claim_id>/run/`. A `run_coding` child additionally writes a
`bullet.coding-harness-observation.v1` record with `cost: "UNPRICED"` — an
observation, not Evidence, not a `COMPONENT_PROOF`.

## Refusal codes

All four scripts print `DOGFOOD_OPS_<CODE>: <value>` on stderr and exit 1
(`prepare-harness.sh` exits 2 for an UNBOUND harness, matching
`bullet coding harness-check`).

### Shared

| Code | Meaning |
| --- | --- |
| `ARG_UNKNOWN` | an argument this script does not define |
| `TOOL_MISSING` | a required command is not on PATH |
| `DATA_DIR_REQUIRED` / `_NOT_ABSOLUTE` / `_MISSING` | `--data-dir` absent, relative, or not there |
| `DATA_DIR_UNTRUSTED` | the data dir is not self-owned mode 0700 |
| `DATA_DIR_NOT_CANONICAL` / `DATA_DIR_SYMLINK` | the path resolves elsewhere |
| `SESSION_FILE_*` | the session file is absent, relative, not 0600 self-owned, or not `bullet.dogfood-ops-session.v1` |
| `LEASE_SOCKET_MISSING` | no lease socket — serve.sh is not running |
| `SESSION_IDENTITY_INVALID` | uid/gid/epoch in the session file are not numeric |

### `serve.sh`

| Code | Meaning |
| --- | --- |
| `FARMD_BIN_REQUIRED` / `_INVALID` / `_NOT_CANONICAL` | `--farmd` (or `$BULLET_FARMD_BIN`) is missing, not executable, or a symlink |
| `BIND_NOT_LOOPBACK` | `--bind` is not `127.0.0.1:<port>` |
| `PORTAL_ORIGIN_NOT_LOOPBACK` | `--portal-origin` is not an exact loopback origin |
| `DATA_DIR_ANCESTOR_WORLD_WRITABLE` / `_SYMLINK` / `_MISSING` | an ancestor would make the kernel refuse the proof root (this is what rules out `/tmp`) |
| `ALREADY_RUNNING` | the pid file names a live process |
| `PID_FILE_INVALID` / `NOT_RUNNING` | `--stop` found no usable pid file |
| `SUBDIR_UNTRUSTED` / `SOCKET_DIR_UNTRUSTED` | `logs/`, `custody/` or `socket/` exists with the wrong owner or mode |
| `SOCKET_PATH_TOO_LONG` | the lease socket path exceeds the `sun_path` budget |
| `KEY_PROVISION_FAILED` / `_INVALID` / `KEY_CUSTODY_INVALID` | `--provision-lease-transport-key` failed, or the key is not a self-owned 0600 64-byte file |
| `REGISTRY_CUSTODY_INVALID` / `REGISTRY_MISMATCH` | an existing peer registry has the wrong custody, or does not match this uid/gid/runner |
| `BOOTSTRAP_PROVISION_FAILED` / `BOOTSTRAP_TOKEN_CUSTODY_INVALID` | the token-file path failed |
| `FARMD_EXITED` / `FARMD_NOT_READY` / `HEALTH_INVALID` | farmd died, never bound, or `/health` was not `ok` |
| `BOOTSTRAP_TOKEN_INVALID` | no `boot_<64hex>` on either path |
| `BOOTSTRAP_EXCHANGE_FAILED` / `CSRF_INVALID` / `SESSION_COOKIE_MISSING` | the bootstrap exchange did not return a usable session |

### `prepare-harness.sh`

| Code | Meaning |
| --- | --- |
| `PROVIDER_UNSUPPORTED` | only `claude` has the wired `--dogfood-*` argv path |
| `SOURCE_REPO_*` / `BASE_SHA_UNAVAILABLE` | `--source-repo` is not a canonical absolute git repo with a resolvable HEAD |
| `OBJECTIVE_REQUIRED` / `_MULTILINE` / `_TOO_LONG` | the objective must be one line of at most 4096 bytes |
| `GATE_ID_INVALID` | not `gat_` + 64 lowercase hex |
| `GATE_ID_MULTIPLE_UNBINDABLE` / `SCOPE_MULTIPLE_UNBINDABLE` | `BULLET_HARNESS_GATE_ID` and `BULLET_HARNESS_SCOPE` each become exactly one `bullet-runner` flag, so more than one cannot be carried through this seam. The script refuses rather than silently dropping a grant. |
| `SCOPE_INVALID` | scope must be a single-line relative prefix without `..` |
| `PRESERVATION_DIR_EXISTS` | the runner requires an exact new directory |
| `PRESERVATION_PARENT_UNTRUSTED` | its parent is not self-owned 0700 |
| `DOGFOOD_INPUT_MISSING` / `_UNTRUSTED` | policy, binding or enrollment is absent or not 0600 self-owned |
| `DOGFOOD_EXECUTABLE_WRITABLE` | the staged provider executable is group- or other-writable |
| `MAX_BUDGET_INVALID` / `_NOT_POSITIVE` | the runner never starts billable work without a positive cap |
| `CREDENTIAL_SHAPE_INVALID` / `_SOURCE_MISSING` / `_TARGET_INVALID` / `_DIGEST_INVALID` | a `source,target,blake3` grant is malformed; the digest is checked for shape only, never recomputed |
| `LEASE_KEY_MISSING` / `LEASE_KEY_CUSTODY_INVALID` | the lease-transport key is not a self-owned 0600 64-byte file |
| `CANDIDATE_KEY_DERIVATION_FAILED` / `_CUSTODY_INVALID` | the public half could not be derived, or the record is not a single-link 0600 file |
| `DOGFOOD_RECEIPT_EXISTS` | the receipt path is create-once |
| `ENV_VALUE_MULTILINE` | a binding value contains a newline, which the env-file shape forbids |
| `HARNESS_CHECK_UNREADABLE` | `bullet coding harness-check --json` produced no readable report |
| `HARNESS_UNBOUND` (exit 2) | at least one REQUIRED name is missing |
| `HARNESS_PLACEHOLDERS_PRESENT` | the env file binds placeholders; pass `--allow-placeholders` to acknowledge a dry run |

### `worker-loop.sh`

| Code | Meaning |
| --- | --- |
| `MANIFEST_REQUIRED` / `_NOT_ABSOLUTE` / `_MISSING` | `--manifest` is absent or relative |
| `<SUBJECT>_BIN_REQUIRED` / `_INVALID` / `_NOT_CANONICAL` / `_UNPROTECTED` | a manifest subject (`TRANSACTION_OFFLINE`, `FARMD`, `RUNNER`, `GITD`, `VERIFIER`) is missing, not executable, a symlink, or group/other-writable |
| `MANIFEST_ENCODER_INVALID` | jq did not emit the expected terminal newline to strip |
| `MANIFEST_UNPROTECTED` / `MANIFEST_SCHEMA_INVALID` | an existing manifest is writable by others or is not the admitted schema |
| `WORKER_BIN_REQUIRED` / `_INVALID` | `--worker` (or `$BULLET_COMMAND_WORKER_BIN`) is missing |
| `STATE_DIR_UNTRUSTED` | `<data-dir>/worker` exists with the wrong owner or mode |
| `ENV_FILE_*` | the harness env file is absent, relative, not 0600 self-owned, empty, or contains a name outside `BULLET_HARNESS_*` |
| `INTERVAL_INVALID` / `MAX_IDLE_INVALID` / `DEADLINE_MS_INVALID` | out-of-range loop bounds |
| `RUNNER_ID_INVALID` | the session's runner id is not `run_` + 64 hex |

### `../stage-rust-toolchain.sh`

| Code | Meaning |
| --- | --- |
| `TOOLCHAIN_INVALID` / `HOST_INVALID` | malformed `--toolchain` or `--host` |
| `TOOLCHAIN_ROOT_MISSING` / `_NOT_CANONICAL` / `_INCOMPLETE` | the rustup toolchain directory is absent, a symlink, or missing `bin/cargo`, `bin/rustc` or `lib` |
| `STAGING_ROOT_EXISTS` / `_NOT_ABSOLUTE` / `_IN_FAMILY_TREE` | the staging root must be a new absolute directory outside `/home/ubuntu/bullet` |
| `MEMBER_NOT_A_WORKSPACE` / `MEMBER_LOCK_MISSING` | a `--member` has no `Cargo.toml` or no `Cargo.lock` for `--locked` |
| `MEMBER_LOCK_MUTATED` | `cargo fetch` changed a member's `Cargo.lock`; this lane never writes another agent's tree |
| `CARGO_FETCH_FAILED` | `cargo fetch --locked` failed (usually no network); the tree is still staged and `cargo-home` is reported `EXCLUDED` |
| `RUSTC_VERSION_UNAVAILABLE` | the source `rustc` would not report a version |

## What these scripts do NOT prove

* **No transaction, no Evidence, no release.** Everything here is
  `COMPONENT_PROOF` / `OPERATOR_STAGING` at best. `transaction_gate_eligible`,
  `independent_evidence_eligible` and `release_gate_eligible` stay false.
* **No real coding turn.** No script here spawns a provider. A green
  `harness-check` proves that seventeen environment variables are non-empty in
  the process that ran it — nothing about whether farmd, the ledger or the
  runner would accept those values.
* **The dogfood loop is not closed.** `BULLET_HARNESS_WORK_PACKAGE_ID`,
  `BULLET_HARNESS_CANDIDATE_REQUEST_DIGEST` and
  `BULLET_HARNESS_IDEMPOTENCY_KEY` have no producer; until they do, a
  `run_coding` command cannot be driven end to end from this operator path.
* **Credentials do not reach the provider.** `--credential` is recorded and
  admitted, but `claude_dogfood_args()` in
  `apps/bullet-runner/src/bin/bullet-command-worker/child/coding.rs` emits no
  `--dogfood-credential` flag, and `child.rs` clears the child environment. A
  real Claude turn launched through farmd would therefore start with no staged
  credential.
* **serve.sh is not an installed daemon.** It is a foreground-launched loopback
  farmd under the operator's own uid, with no unit file, no restart policy, no
  log rotation, and no `--kernel-authority-socket`.
* **The staged toolchain is not qualified.** `stage-rust-toolchain.sh` copies
  bytes and prints commands. It does not run them, does not prove the toolchain
  is reproducible, and does not prove any gate passes under it. Cargo takes a
  package-cache lock even with `--offline`, so a gate must seed a writable
  `CARGO_HOME` from the staged read-only one.
* **Nothing here is a coordinator.** There is no fleet, no scheduling, no
  cancellation: `bullet coding stop` is still `STOP_UNIMPLEMENTED`.
