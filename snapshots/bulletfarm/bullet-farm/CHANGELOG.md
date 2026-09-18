# Changelog

All notable Bullet Farm changes are recorded here. A release entry is added
only when a signed tag and its release manifest exist; working-tree progress
belongs under `Unreleased`.

## Unreleased

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
