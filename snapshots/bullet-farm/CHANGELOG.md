# Changelog

All notable Bullet Farm changes are recorded here. A release entry is added
only when a signed tag and its release manifest exist; working-tree progress
belongs under `Unreleased`.

## Unreleased

### ASAP operator loop (HOLD stays)

Contributor loopback only. Not first-GA, not schema-3, not a HOLD lift.
`live_admission_enabled` stays false. `release_eligible` stays false.

Hub ([PR #16](https://github.com/neverhuman/bullet-farm/pull/16) `33805d3`,
oid pin [PR #17](https://github.com/neverhuman/bullet-farm/pull/17) `58a742ce`
on `main`):

- `just console` starts farmd, Vite Portal, and `worker-loop.sh` when farmd,
  runner, gitd, verifier, and `transaction_offline` exist. Missing gitd is
  `worker=UNBOUND`, not a hang.
- `--leave-bootstrap` writes a cookie-free ops session so the worker can
  claim. Portal honors `BULLET_PORTAL_PORT` and `BULLET_FARMD_TEST_PROXY`.
- Default no-args `bullet` is the TUI after login. `coding stop` stays
  `STOP_UNIMPLEMENTED`. Ctrl+C detaches the TUI only.

Kernel ([PR #18](https://github.com/neverhuman/bullet-kernel/pull/18) merged
`088e53ad` on `main`):

- No-args `bullet` runs `tui`. `coding harness-bind` produces the three
  ledger identities. `harness-check` is BOUND only when all seventeen
  `BULLET_HARNESS_*` names are present.
- Signed-in Codex/Cursor `send` admits harness-egress or refuses
  `SIGNED_IN_CONTAINMENT_UNAVAILABLE`. Cursor stream-json without a
  `PatchProposal` stays `CURSOR_ACP_EVENTS_EMPTY` (Fork B). A text ping is
  not a turn.

Local observation 2026-09-11 on a new `$HOME` data dir (`127.0.0.1:7424`,
not `bullet-live` / 7420): worker claimed once and typed
`COMMAND_CODING_HARNESS_UNBOUND` (verification key). The loop stopped.
Duplicate submit reused the same command id; no second spawn. Claude,
Codex, and Cursor each have a PENDING row. No enrollments and no keys
were written. Three-provider attempt→proposal stays unclosed. Public GIF
remains HOLD + PENDING rows; LIVE n is not “agents finished.”

Hosted **required** is the merge gate. Scheduled diagnostics stay named
red or unexecuted.

### v0.2.0-operator-cli

Named operator-CLI campaign, not a schema-3 lock and not `self-hosted-v1`.
`release_eligible` stays false. Operating HOLD stays printed.

Hub (this repository, after PR #11, #12, and #14):

- Recorder paints the product TUI (`github-dark`, outer PTY, `CAST_ENTROPY`
  gate) so `operator-tui.gif` is a readable 1920×1080 dark tape with HOLD and
  PENDING coding rows, not an off-white slide and not the historical blank
  light GIF.
- `prepare-harness.sh` issues work-package IDs from the admitted command, not
  dry-run seeds.

Kernel (on kernel `main` as of [neverhuman/bullet-kernel#18](https://github.com/neverhuman/bullet-kernel/pull/18)
`088e53ad`; v0.2 closeout was #17 `53f6d96`):

- v2 `run_coding` admission binds nonce and quota so farmd can dispatch an
  admitted task.
- v0.2 D1 is HTTP then the real `bullet-runner` plus a stub: one failure is
  retained and a second spawn is refused.
- After ASAP, `just console` writes a `bullet.command-worker-binary-manifest.v1`
  and starts `worker-loop.sh` when subjects exist. A loopback claim reached
  `COMMAND_CODING_HARNESS_UNBOUND` (verification key); that is a typed refuse,
  not “no worker.” Production gitd settlement and three-provider
  attempt→proposal stay unclosed without operator enrollments.
- Codex signed-in argv can emit a `PatchProposal`. Cursor stream-json without
  a proposal is `CURSOR_ACP_EVENTS_EMPTY` (Fork B; not live ACP). A text ping
  is not a turn.
- `bullet tui` overlays admitted coding commands when mission tables are empty
  and redacts 64-hex ids. Rows stay queued/unknown. They are not VERIFIED.

Hosted **required** on member `main` is the merge gate. Scheduled Jankurai is
machine-local; hosted required is the merge gate. Fixing hosted Jankurai is G8,
not this version.

### Added

- Canonical v1alpha1 policy and wire contracts with hostile fixtures, generated
  consumer bindings, and two bounded formal models.
- Rust family coordination, lock verification, doctor, hub validation, and
  repository-path dependency checks.
- Explicit component, synthetic, transaction, live, and release evidence
  classes with self-hosted, separately certified GitHub, and later distributed
  deployment stages.
- Signed launch-grant wire contract (`SignedLaunchGrantV1`, hub `a2d6b2a`) and
  Kernel signed launch-grant admission with Linux provider egress isolation
  (`bullet-harness-egress`; kernel `d388733`); ADR 0011.
- Policy `v1alpha2` operator-ratified live-admission rule and its five reason
  codes (hub `bf5c642`); Kernel loader mirror (kernel `0d848f6`); ADR 0012. The
  committed policy stays `v1alpha1`, generation 1, live admission disabled.
- Fail-closed release-truth report, now invoked as `check release --profile
  <profile> --receipts <absolute-registry> --json`; the earlier unprofiled
  `--report` spelling is retired. `just release-truth` produces the
  drift-checked diagnostic page `docs/assurance/release-truth.generated.md`.
- Release bundle verification (`release verify`, `352f963`), constrained
  one-target extraction (`release extract`, `ba09056`), and the signed receipt
  verifier (`release receipt-verify`, `143f8b9`).
- Quarantined former unprofiled component verifier for one historical gate,
  `release.rust-msrv-1-95`, with fixed-descriptor tests (`d762f86`). No public
  command or current profile invokes it: `legacy-v1-26` is a static all-BLOCKED
  diagnostic that ignores registries, and profiled release commands use their
  selected structural registry boundary while kind-specific semantics remain
  open engineering work.
- Sealed setup tool subjects for the source-setup transaction (`7efe2f3`).
- Five read-only farmd projections — fleet, sessions, merge rail, quality lab,
  audit (kernel `529bad1`) — and the Portal views over them (portal `95108e3`).
- End-to-end policy-gated live-conformance path `bullet provider
  live-conformance` with sealed thirteen-step receipts (kernel `ba485d5`),
  nightly real-binary mode `BULLET_LIVE_REAL=1` (kernel `b4735da`), and the
  operator runbook `docs/runbooks/live-conformance.md` (`48cca46`).

### Security

- Fail-closed secret/dependency and ratcheted Jankurai audit lanes.
- BulletGit repository-local command configuration admission, including a
  clean-filter canary regression.
- Portable-path ancestor collisions are refused in `bullet-wire` proposal
  validation (`65a5ea7`).

### Known limitations

- No release profile is authorized. Hub-only source installation,
  production transaction storage, admitted live providers/forges, recovery,
  packaging, signing, provenance, and the Jankurai 90 release threshold remain
  blocked as listed in `docs/release.md`. The former unprofiled 26/26 inventory
  survives only as non-authoritative `legacy-v1-26`; an unprofiled release check
  returns typed `PROFILE_REQUIRED`, and every named profile remains blocked.
- Everything above is `COMPONENT_PROOF` at most. No `TRANSACTION_PROOF`,
  `LIVE_PROOF`, or `RELEASE_PROOF` receipt exists: no provider has a live
  conformance receipt, no policy generation 2 has been ratified, and the
  launch-grant, egress, and projection components have admitted no real
  provider or effect.
