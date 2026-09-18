# Effect reconciliation — the offline half

Status: **offline worker reconciliation available; live read-back blocked**  
Owner: Bullet Farm maintainers  
Last reviewed: 2026-08-25  
Component receipt baselines (minimum; replay current-head lanes before use): bullet-kernel `c4731aa` (authenticated offline command reconciliation), bullet-farm `347da232`,
bullet-portal `6b294ce` (projection rules and same-origin component proof)

This runbook covers what an operator can settle **today**: a `PENDING` public command that no execution
adapter will ever pick up. It does not cover live forge read-back; that half cannot be written until the
Jeryu or GitHub effect lanes have a receipt (see §5).

## 1. The ladder

Every public mutation is `POST /api/v1/commands`, answers `202` with a `PENDING` subject, and thereafter moves
only through the closed set below (`status_name` in `apps/bullet-farmd/src/commands.rs` and the current
[`glossary`](../glossary.md)). The connected effect order and remaining authority closure are governed by
Waves 5–6 of the active [`closure roadmap`](../assurance/closure-roadmap.md); the old
[`v1-closure-plan.md`](../assurance/v1-closure-plan.md) records historical V1 slice names only.

| Phase | Meaning | Who may set it today |
| --- | --- | --- |
| `PENDING` | admitted, durable, one correlated outbox row and one `command_submitted` event | `POST /api/v1/commands` |
| `APPLIED` | dispatched to an execution adapter; not yet read back | nobody (no adapter exists) |
| `VERIFIED` | read-back proved the exact intended effect | nobody (no adapter exists) |
| `FAILED` | durably refused; nothing was executed | the offline worker, for an unknown command kind |
| `UNKNOWN` | an adapter or response was lost; the effect may or may not have happened | the offline worker, for a recognised kind with no executor |

Transport success never implies verification: the Portal renders `PENDING`/`APPLIED` amber, only a persisted
`VERIFIED` green, and `UNKNOWN` as unknown — never healthy, never "empty" (`bullet-portal/docs/architecture.md`,
"Status vocabulary").

## 2. What the offline worker settles, and what it refuses to invent

`POST /internal/v1/commands/{id}/reconcile` (`apps/bullet-farmd/src/api.rs`) calls
`CommandRequest::offline_worker_resolution()` (`crates/application/src/commands.rs`). The only two outcomes
it may persist:

| Command kind | Persisted phase | `result.code` | Meaning |
| --- | --- | --- | --- |
| `run_demo` | `UNKNOWN` | `EXECUTION_ADAPTER_UNAVAILABLE` | recognised, but no admitted execution and read-back adapter is connected; epistemically unknown, not failed |
| anything else | `FAILED` | `UNSUPPORTED_COMMAND_KIND` | durably refused; nothing ran |

It never produces `APPLIED` or `VERIFIED`, never dispatches, and never touches a provider, verifier, or forge.
Inside one `IMMEDIATE` SQLite transaction (`crates/adapters/src/sqlite/commands.rs`) it requires exactly one
matching dispatch outbox row and exactly one `command_submitted` event, writes the final row, marks the outbox
row with the same phase (acked, never delivered), and appends exactly one `command_reconciled` event. A second
call on a settled command returns the byte-identical body without writing; a ledger whose rows disagree with
that shape is refused as a store failure rather than re-settled. This is the semantics the Kernel commit
`77a0ecd` receipt covers ("Authenticated offline command reconciliation", historical component inventory) —
COMPONENT_PROOF class, nothing more.

## 3. Procedure (observed on this host, 2026-08-25)

The worker route exists only when farmd is started with `--worker-token-file`; without it the router has no
worker authority and the route answers `503 WORKER_AUTHORITY_UNAVAILABLE`. The token is `wrk_` + 64 lowercase
hex in a private, regular, single-line, non-symlink file (`apps/bullet-farmd/src/main.rs`). The reference
flow is `bullet-portal/ops/ci/real-farmd.sh`; the commands below use its exact flags.

```bash
umask 077
printf 'wrk_%s\n' "$(head -c 32 /dev/urandom | od -An -tx1 | tr -d ' \n')" > /abs/worker.token
bullet-farmd --data-dir /abs/data --bind 127.0.0.1:7420 \
  --portal-origin http://127.0.0.1:7420 --worker-token-file /abs/worker.token
# stdout: "Bullet Farm one-time bootstrap: boot_<64 hex>"  (capture it; never log it)
```

1. Exchange the bootstrap for a browser session (this is the Portal's path; the operator needs it only to
   submit a command by hand):

   ```bash
   curl -s -D headers -H 'Origin: http://127.0.0.1:7420' -H 'Content-Type: application/json' \
     -d '{"bootstrap_token":"boot_…"}' http://127.0.0.1:7420/api/v1/auth/bootstrap
   # 200; set-cookie: bullet_session=…; body carries "csrf_token"
   ```

2. Submit (or locate) the pending command:

   ```bash
   curl -s -H 'Origin: http://127.0.0.1:7420' -H 'Cookie: bullet_session=…' -H 'X-Bullet-CSRF: …' \
     -H 'Content-Type: application/json' \
     -d '{"idempotency_key":"<unique>","kind":"run_demo","payload":{}}' http://127.0.0.1:7420/api/v1/commands
   # 202 {"id":"cmd_…","status":"PENDING","kind":"run_demo","payload_digest":"…"}
   ```

3. Reconcile it with the independent worker bearer — exactly one `Authorization` header:

   ```bash
   curl -s -X POST -H "Authorization: Bearer $(cat /abs/worker.token)" \
     http://127.0.0.1:7420/internal/v1/commands/cmd_…/reconcile
   ```

   Observed: `200` with `{"id":…,"status":"UNKNOWN","kind":"run_demo","payload_digest":…,
   "result":{"code":"EXECUTION_ADAPTER_UNAVAILABLE","command_id":…,"payload_digest":…,"detail":…,"repair":…}}`.
   Re-running the same call returned a byte-identical body; `GET /api/v1/commands/{id}` with the session cookie
   returned the same body; the ledger held exactly one `command_submitted` and one `command_reconciled` event.

4. Record the settled body with the command id and payload digest as the maintenance evidence. Do not submit
   a "retry" under the same idempotency key: the repair text says to submit a **new** command key once an
   adapter exists, because the original is now durably `UNKNOWN`.

Refusals observed on this host (all HTTP `401`, `curl --fail` exit 22; problem+json bodies):

| Request shape | `code` |
| --- | --- |
| no `Authorization` header, or only the session cookie | `WORKER_AUTHORITY_REQUIRED` |
| `Bearer wrk_invalid`, a wrong 64-hex token, or two `Authorization` headers | `WORKER_AUTHORITY_INVALID` |

An unknown command id currently answers `500 STORE_FAILURE` (`retryable: true`), not `404`; treat it as
"check the id", not as a transient outage. A daemon started without a worker token answers `503`.

## 4. The effect broker's rule for a lost response

The command worker above settles commands; the effect broker (`crates/effects/src/broker.rs`) settles
external effects, and its rule is what "reconciliation" will mean once a forge is admitted:

- states run `PROPOSED → AUTHORIZED → DISPATCHING → RECEIPT_PENDING → VERIFIED → COMMITTED`; a push whose
  response is lost lands in `OUTCOME_UNKNOWN` and the correlated outbox row is marked `UNKNOWN`;
- `dispatch` on an `OUTCOME_UNKNOWN` intent is refused with `RETRY_WITHOUT_RECONCILE` — no public path can
  retry without reconciling first;
- `reconcile` reads the exact target ref back (`READ_BACK_METHOD = "git-ls-remote-read-back"`): if the
  remote already holds the desired OID the original effect is **adopted** as `VERIFIED`; if non-execution is
  proven (create: ref absent; update: ref still at `expected_old_oid`) and no unknown-retry has been spent,
  exactly one retry runs; any other observation, or a spent retry, is **quarantined**. There is never a blind
  second write.

Today this runs only against the in-process `LocalBareForge`/`LostResponseForge` fixtures
(`apps/bullet-effects`, Kernel tests): SYNTHETIC/COMPONENT evidence.

## 5. What waits (do not document it as available)

| Missing | Why the runbook stops here | Tracked as |
| --- | --- | --- |
| Live forge read-back | typed Jeryu/GitHub capability, delivery/check, integration/read-back/reconciliation adapters and semantic receipt admission remain local work; authenticated protected test repositories and role-separated credentials remain operator work, and the running forge must not be modified to fake capability | [ADR 0013](../decisions/0013-operator-decision-register.md) OD-B/OD-C; gates `release.forge.jeryu`, `release.forge.github-app` |
| Identity-exact adoption (C9: fence + desired OID) | command idempotency is a component; graph mint is not a live path | [`../assurance/product-gaps.md`](../assurance/product-gaps.md) C9 |
| `APPLIED`/`VERIFIED` for any command | no admitted runner/verifier/effect adapter is connected to the worker | Active roadmap Waves 2, 5, and 6; historical V1-S4/V1-S5; [`../release.md`](../release.md) "Production Kernel transaction" |
| Runner dispatch | the product `bullet-runner` exits with `LEASE_TRANSPORT_ADMISSION_UNAVAILABLE`; the existing Unix/HTTP components are not production-admitted authority | Active roadmap Waves 2 and 5; historical V1-S4; peer/process identity, inherited connected transport, replay/read-back, and restart-safe request settlement remain |
| Backup/restore settling outbox ambiguity | restore is quarantined and does not reconcile effects | [`backup-restore.md`](backup-restore.md) |

A `200` from the reconcile route is an honest `UNKNOWN`, not progress toward green.
