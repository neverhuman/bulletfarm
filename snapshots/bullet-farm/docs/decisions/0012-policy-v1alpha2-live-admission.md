# 0012 — Policy v1alpha2: operator-ratified live provider admission

Status: Proposed — pending operator ratification (hub validator `bf5c642` and Kernel loader mirror `0d848f6` landed; no committed policy enables live admission)
Owner: Bullet Farm maintainers
Related: 0005 (signed authority and key lifecycle), 0011 (signed launch grants and provider egress isolation)

## Decision

`PolicySnapshotV1` accepts a second `schema_version`, `v1alpha2`. Its rules are exactly v1alpha1's with one
addition: `sandbox_policy.live_admission_enabled = true` is legal only when

1. `policy_generation >= 2` — generation 1 is the committed Gate 0 offline policy and can never admit a provider
   (`LIVE_ADMISSION_REQUIRES_GENERATION` otherwise); and
2. at least one `issuer_keys` entry is `authority-signing` / `paseto-v4.public`, lists the `provider-runner`
   audience, carries no `revoked_at_unix_ms`, and overlaps the policy validity window
   (`LIVE_ADMISSION_REQUIRES_RUNNER_KEY` otherwise).

`validate()` stays structural and never reads a wall clock. `validate_at(now)` adds the instant semantics
`authority_key_at` already uses (activation inclusive, expiry and revocation exclusive): the policy window must
contain `now` (`POLICY_NOT_ACTIVE`) and, when live admission is enabled, a qualifying key must be active at `now`
(`LIVE_ADMISSION_REQUIRES_RUNNER_KEY`).

## What stays immutable

In both versions the following remain `UNSAFE_POLICY`: `maximum_lease_ttl_seconds > 15`,
`unknown_quota_is_headroom`, `arbitrary_shell_gates`, `author_evidence_is_independent`, `unknown_satisfies_gate`,
`!r2_requires_sealed_product_holdout`, `universal_incumbent != "T0"`, and `evolutionary_authority` (campaign spend
is a separate, later gate). The conservatism set is checked before the live-admission rule, so no v1alpha2 policy
can trade one for the other. Checked immediately after it, in both versions, is the A7 STONITH inequality:
`maximum_lease_ttl_seconds = 0` is `UNSAFE_POLICY` with the reason `self-kill grace must be strictly less than lease
TTL`, because the Kernel runner's self-kill budget (4/5 of the admitted TTL) and the remaining grace must both fall
strictly inside the TTL (hub `policy.rs`, Kernel `policy_snapshot.rs`; Kernel test `crates/runner/tests/stonith.rs`). A v1alpha1 policy with live admission enabled is refused as `UNSAFE_POLICY` with the
same reason as before; the existing `policy_registry.rs` suite is untouched. Nested policy records and
`IssuerKeyV1` stay `v1alpha1`. Any other snapshot version is `UNSUPPORTED_POLICY_SCHEMA`.

| Condition | Reason code |
| --- | --- |
| `schema_version` outside `{v1alpha1, v1alpha2}` | `UNSUPPORTED_POLICY_SCHEMA` |
| immutable conservatism set violated (either version) | `UNSAFE_POLICY` |
| v1alpha1 with live admission enabled | `UNSAFE_POLICY` (unchanged) |
| v1alpha2 live admission at generation < 2 | `LIVE_ADMISSION_REQUIRES_GENERATION` |
| v1alpha2 live admission without a qualifying provider-runner key, structurally or at `validate_at(now)` | `LIVE_ADMISSION_REQUIRES_RUNNER_KEY` |
| `validate_at(now)` outside the policy window | `POLICY_NOT_ACTIVE` |

## Contract consequence

The JSON-Schema for `PolicySnapshotV1.schema_version` is `enum ["v1alpha1", "v1alpha2"]` (catalog field type
`policy_schema_version`); every other record keeps `const "v1alpha1"`. The generated Rust and TypeScript records
already carried `schema_version: String`, so consumers only observe a new schema-bundle hash. The committed
`policy/v1alpha1/policy.json` remains generation 1, v1alpha1, live admission disabled, and changes only in its
embedded `schema_bundle_hash`.

## Ratification and follow-up

The only live-enabled example is the fixture `crates/bullet-wire/tests/fixtures/policy-v1alpha2-live-enabled.json`,
which registers the golden fixture-only key (`bullet-kernel-local` / `authority-test-1`); normative policy must never
trust it (ADR 0005). Enabling live admission is an operator act: a separately generated and protected v1alpha2
policy at generation ≥ 2 registering a real `provider-runner` authority key, recorded in the coordination log,
never an agent edit. The Kernel loader (`crates/application/src/policy_snapshot*`) mirrors this rule since
bullet-kernel `0d848f6` and the live-conformance `POLICY` step reads it through the production loader; under the
committed generation-1 policy it keeps refusing `POLICY_LIVE_ADMISSION_DISABLED`.
