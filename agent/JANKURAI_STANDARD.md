# Jankurai Standard Binding

Standard version: `0.9.0`
Target stack: Rust core, TypeScript/React/Vite product surface, SQLite local
truth (PostgreSQL later), generated contracts. No Python product truth.

The repository binding is this file plus `agent/standard-version.toml`,
`agent/owner-map.json`, `agent/test-map.json`, `agent/proof-lanes.toml`,
`agent/generated-zones.toml`, `agent/boundaries.toml`,
`agent/security-policy.toml`, `agent/tool-adoption.toml`,
`agent/audit-policy.toml` and `agent/exceptions.toml`. An installed auditor may
expose its full operating standard through `jankurai doctor`; never commit a
machine-local auditor checkout path as public project authority.

## Hard rules for this repository

- Keep files small. Split before 500 LOC; prefer under 300.
- Do not hand-edit generated zones. Every zone in `agent/generated-zones.toml`
  records the command that regenerates it; repair from the source, never the
  output.
- Do not create Git worktrees.
- One-command setup and one-command validation must exist.
- Public API clients are generated from contracts, never handwritten DTOs.
- Authority never lives in the portal, README, or a provider session.
- `minimum_score` in `agent/audit-policy.toml` is the Hub's audit floor and is
  an upward-only ratchet; `ops/ci/audit.sh` passes this file as `--policy`.
- Never skip-green: a missing tool, a skipped case, or an `UNKNOWN` result is
  never success.

## Repair receipts and the agent-friendly exception surface

Every failure in the Hub is a typed value that carries its own repair routing,
so the next agent does not have to read a log to know what to do.
`CoordError` (`src/coord/mod.rs`) is the shape, and it holds these fields
directly:

- **purpose** — what the refusal is protecting, in one phrase: "preserve an
  ambiguous mutation outcome", "preserve one-writer and exact-subject
  transaction boundaries", "refuse substitution for an unavailable exact
  prerequisite", "reject a schema outside the admitted version set", "stop
  replay of incomplete or corrupt durable state".
- **reason** — the stable machine code plus the specific detail.
  `UNSUPPORTED_SCHEMA`, `CLAIM_OVERLAP`, `MISSING_OPTION`, `INVALID_ARGUMENT`,
  `DUPLICATE_OPTION` and the `bullet-wire` codes such as
  `INVARIANT_ALIAS_COLLISION` are wire contract: the family coordinator, the
  installer and the sibling repositories all match on those strings, so they are
  renamed only with a contract change, never for tidiness.
- **common fixes** — the recoverable actions for that class, already enumerated
  per code by `repair_metadata`. An `_UNKNOWN` or `TIMEOUT` outcome says
  "reconcile by the original request and desired-state identity" and "do not
  dispatch a second write or switch providers"; a conflict says "read back
  current state before issuing a new command".
- **repair_hint** — the next command to run: "run authoritative read-back and
  keep the subject frozen", "inspect coordinator status and the exact current
  subject", "run `bullet-family doctor --json`", "run the owning integrity and
  recovery proof before mutation".
- **docs_url** — the section that explains the class:
  `docs/errors.md#outcome-unknown`, `#conflict-or-changed-subject`,
  `#unsupported-or-corrupt-state`.

The lane that reproduces a failure is the repair receipt. `bash ops/ci/audit.sh`
leaves `.jankurai/repo-score.json`, `.jankurai/repo-score.md` and
`.jankurai/repair-queue.jsonl`; `just check` is the merge gate; `just contract`
proves generated and fixture drift; `just security` proves the scans; `just
family` runs the sibling required lanes. Rerunning the named lane and keeping
its artifact is how a repair is evidenced. An `UNKNOWN` outcome is reported as
`UNKNOWN`; it is never reconciled into success.

## No ZYAL runbooks live here

A ZYAL v1 runbook is a `RUN_FOREVER` daemon envelope. The Hub has no
run-forever agent loop, so its proof surface is `agent/proof-lanes.toml` and the
`ops/ci/<lane>.sh` scripts instead. The previous `agent/zyal/fast.zyal` was a
three-line stub that was not a ZYAL envelope at all; it was removed rather than
dressed up into a fake envelope to satisfy a checker.

## Accepted audit exceptions

Exceptions are declared where the auditor reads them, never as a silent
suppression: `agent/audit-policy.toml` (`[dead_language] allow_terms`, with the
justification for each term next to it), `agent/exceptions.toml`, and inline
`jankurai:allow <detector> reason=… owner=… expires=…` comments. A wire-contract
string is never renamed to satisfy a marker rule.
