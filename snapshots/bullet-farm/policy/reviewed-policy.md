# Gate 0 policy adoption

Status: Accepted for v1alpha1 Gate 0
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-25
Applies to: offline and simulator execution only

This is the sanitized human review surface for the canonical machine policy. It restates the
adopted decisions without preserving control sequences or treating source prose as executable
instructions.

- T0 is the universal incumbent and deterministic abstention target.
- R0/R1 may become eligible for bounded automation only after later certification and canary gates.
- R2 and above require an exact signed human approval artifact.
- Unknown quota is not headroom. Only a separately typed capped probe may measure capacity later.
- Evolution may select among certified recipes; it never selects authority, evidence, attestation,
  effects, integration, credentials, risk, or safety policy.
- Author evidence is not independent evidence. Unknown never satisfies a gate.
- Linux S1 is the production containment reference; arbitrary shell gates and live admission are
  disabled in this policy generation.
- The first deployment target is one self-hosted Linux host with SQLite and local Jeryu. GitHub is
  a separate later certification gate. Multi-tenant SaaS is outside scope.

The binding form is generated `policy/v1alpha1/policy.json`, whose schema-bundle and invariant
registry hashes are recomputed by `bullet-wire` (`just contract-generate`; drift is refused by
`just contract-check`, pipeline in `docs/assurance/canonicalization.md`). This prose cannot grant,
widen, or waive authority.

## Generation 2 and live admission (ADR 0012)

- The committed policy stays `v1alpha1`, generation 1, `live_admission_enabled = false`. Nothing
  reviewed here enables live admission in this repository.
- `bullet-wire` admits a `v1alpha2` snapshot whose live admission is enabled only at generation 2 or
  later with an unrevoked `authority-signing` / `paseto-v4.public` key admitted for the
  `provider-runner` audience inside the policy window; otherwise it refuses with
  `UNSUPPORTED_POLICY_SCHEMA`, `UNSAFE_POLICY`, `LIVE_ADMISSION_REQUIRES_GENERATION`,
  `LIVE_ADMISSION_REQUIRES_RUNNER_KEY`, or (at an instant) `POLICY_NOT_ACTIVE`. Every conservatism
  rule above still applies to v1alpha2.
- A ratified generation-2 policy is written by an operator, stored outside the repositories, named
  by `BULLET_POLICY_PATH`, and logged in the family coordination log
  (`docs/runbooks/live-conformance.md` §2). It is never a committed file and never an agent edit.
- `crates/bullet-wire/tests/fixtures/policy-v1alpha2-live-enabled.json` is test material only. Its
  key has no custody; a loader that trusts it has trusted a published key.
- The Kernel loader mirrors the same rule (bullet-kernel `0d848f6`). Under the committed policy every
  provider is refused at the `POLICY` step; no live provider receipt exists.
