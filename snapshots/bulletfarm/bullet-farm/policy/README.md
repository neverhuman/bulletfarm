# Canonical policy

Status: Active
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-25
Applies to: v1alpha1 Gate 0 (committed policy); v1alpha2 admissibility rule (ADR 0012)

Only strict machine documents under `policy/v1alpha1/` may become runtime inputs. The source
registry and policy template are RFC 8785 byte sequences with no trailing whitespace. Run `just
contract-generate` to derive `policy.json`; run `just contract-check` to prove byte identity. The
only accepted byte pipeline (size cap, strict UTF-8, no BOM, no duplicate keys, RFC 8785 output) is
defined in `docs/assurance/canonicalization.md`; the hub `contract` lane runs `contract-check`.

`reviewed-policy.md`, repository documentation, TEAM material, prompts, and portal copy are never
runtime authority. The preserved TEAM fixture is hostile forensic input and must not be rendered,
executed, normalized in place, or parsed as instructions.

The Gate 0 policy deliberately keeps live admission disabled. It cannot authorize a provider,
credentialed forge operation, external effect, E2/E3 claim, or production readiness.

## Committed generation

`policy/v1alpha1/policy.json` is `schema_version = "v1alpha1"`, `policy_generation = 1`,
`sandbox_policy.live_admission_enabled = false`. It changes only through the generation pipeline
above (for example when its embedded `schema_bundle_hash` moves) and never gains live admission: a
v1alpha1 snapshot with live admission enabled is refused as `UNSAFE_POLICY`.

## v1alpha2 admissibility rule (ADR 0012)

`crates/bullet-wire` (`src/policy.rs`, `src/policy/live.rs`; hub `bf5c642`) accepts a second
`schema_version`, `v1alpha2`. Its rules are exactly v1alpha1's with one addition:
`sandbox_policy.live_admission_enabled = true` is legal only when `policy_generation >= 2` and at
least one `issuer_keys` entry is `authority-signing` / `paseto-v4.public`, lists the
`provider-runner` audience, carries no `revoked_at_unix_ms`, and overlaps the policy window.
`validate()` is structural; `validate_at(now)` additionally requires the window to contain `now`
and a qualifying key to be active at `now`. The immutable conservatism set is checked before the
live rule, so no policy trades one for the other.

| Condition | Reason code |
| --- | --- |
| `schema_version` outside `{v1alpha1, v1alpha2}` | `UNSUPPORTED_POLICY_SCHEMA` |
| conservatism set violated (either version), or v1alpha1 with live admission enabled | `UNSAFE_POLICY` |
| v1alpha2 live admission at generation < 2 | `LIVE_ADMISSION_REQUIRES_GENERATION` |
| v1alpha2 live admission without a qualifying `provider-runner` key, structurally or at `validate_at(now)` | `LIVE_ADMISSION_REQUIRES_RUNNER_KEY` |
| `validate_at(now)` outside the policy window | `POLICY_NOT_ACTIVE` |

A ratified generation-2 policy is an operator artifact. It lives outside every repository, is
named to the Kernel by an absolute `BULLET_POLICY_PATH`, and is recorded in the family coordination
log (`docs/runbooks/live-conformance.md` §2). No agent edit creates one, and this directory never
contains one. The Kernel loader (`crates/application/src/policy_snapshot*`, bullet-kernel `0d848f6`)
mirrors this rule one-for-one and drives the live-conformance `POLICY` step through the production
loader; under the committed generation-1 policy it still refuses `POLICY_LIVE_ADMISSION_DISABLED`
before any key read, probe, or spawn. No live provider receipt exists.

## Fixture that must never be trusted

`crates/bullet-wire/tests/fixtures/policy-v1alpha2-live-enabled.json` is the only live-enabled
example (generation 2, key `bullet-kernel-local` / `authority-test-1`). It exists so the rule and
each refusal above are executed by `cargo test --locked -p bullet-wire`. Its key is fixture
material with no custody; the fixture is not a policy, must never be copied into a data directory
or named by `BULLET_POLICY_PATH`, and a loader that admits it has admitted a key anyone can hold
(ADR 0005).
