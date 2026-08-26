# Test and evidence strategy

Status: **normative; pre-release gaps remain**  
Owner: Bullet Farm maintainers  
Last reviewed: 2026-08-25

Tests establish bounded facts about exact subjects. They do not authorize a
provider, forge, installation, effect, or release merely because a process
exited successfully.

## Repository lanes and product profiles

| Repository-local lane | Purpose | Required behavior |
| --- | --- | --- |
| `fast` | bounded component feedback | Hub metadata/onboarding checks, schema-2 setup refusal, no-ambient-Cargo bootstrap refusal, and the exact Hub test partition |
| `lint` | source and CI policy | rustfmt, strict Clippy, actionlint over both workflow suffixes, ShellCheck over every shell/shebang entrypoint, partition/JUnit/observation/aggregation meta-tests, and hosted/Jeryu workflow policy |
| `contract` | cross-boundary contracts | exact wire partition, generated-contract drift refusal, and exactly two pinned TLC models with sanitized formal diagnostics |
| `security` | pre-resolution source and supply-chain checks | current-tree secrets, a structured real-finding canary, RustSec freshness, full cargo-deny licenses/advisories/bans/sources, and strict offline zizmor |
| `docs` | public truth and reproducibility | rustdoc/doctests, repository-relative links, generated release-truth drift, doctor/release BLOCKED agreement, and deterministic README media |
| `required` | standalone repository convergence | `security`, `fast`, `lint`, `contract`, and `docs` sequentially, exactly once; security runs first |

No repository lane has skip-green behavior. A missing required tool, zero-test
partition, unexpected ignored test, timeout, unsupported result, flaky result,
infrastructure error, or `UNKNOWN` fails its owning lane. A green local
`required` is a component observation for that repository, not release
authority.

There are two deliberately different command layers:

- `scripts/ci-local.sh <lane>` and the thin repository `just`/workflow
  wrappers are component merge lanes. A green local `required` proves the
  checks that repository currently implements; it is not product readiness.
- `bullet-family check fast|required` and explicit `check release --profile
  <profile> --receipts <absolute-registry>` are the family/product gate
  runner. It executes only its sealed command catalog
  (`src/check/catalog.rs`: `fast` = 5 commands, the four member `fast` lanes
  plus `scripts/sync-family-contracts.sh check`; `required` = 2 commands,
  `ops/ci/family-contract.sh` and `scripts/demo.sh`). Release executes no
  external proof command: it selects the named profile's dependency closure
  and evaluates only typed, admitted receipt-registry state. Unprofiled release
  is `PROFILE_REQUIRED`; `legacy-v1-26` preserves the historical 26-gate
  diagnostic but is not a GA profile. Today, component CI can be green while
  product `required` and every release profile correctly remain `BLOCKED`.

### Execution budgets and stop evidence

The generated V1 policy is the executable source, not this prose. Its current
exact bounds are a 15-second maximum lease TTL, a 1,800-second maximum Attempt,
and 128 changed paths. `unknown_quota_is_headroom` is `false`. The contract
drift lane and `assurance_controls` test bind these documented values back to
`policy/v1alpha1/policy.json`; changing the policy without updating the
operator explanation fails required CI.

These are ceilings, not reservations. Paid provider dispatch remains blocked
until Kernel durably records a budget and quota reservation tied to the exact
task, provider profile, Attempt, fence, routing/configuration/policy snapshots,
expiry, and settlement. An absent, expired, contradictory, or `UNKNOWN` quota
observation is never schedulable headroom. A bounded probe reservation may be
introduced only through a separately versioned policy and receipt; none exists
in the V1 implementation today.

Mutation freezes and the provider process tree is terminated when lease
renewal updates zero rows, the authority service is unavailable, the Attempt
deadline expires, cancellation is acknowledged, the admitted byte/turn/cost
budget is exhausted, or the provider violates its framed protocol. The
workspace is preserved before cleanup. Restart requires a successor fence and
exact checkpoint; it does not extend the old budget implicitly.

Release evidence for this control is not a log line. It must bind the policy
digest, reservations and settlements, stop command, runner acknowledgements or
lease expiry, process-tree death, preservation receipt, and final portal
projection. Until that connected receipt exists, the kill switch, budget, and
cost controls remain release blockers even when their component tests pass.

### Lanes

| Lane | Where | What it proves |
| --- | --- | --- |
| `fast`, `lint`, `contract`, `security`, `docs`, `required`, `audit` | each repository's `scripts/ci-local.sh` inventory | Component checks of that repository only. Each repository defines its own exact partitions and artifacts; local `required` invokes its atomic lanes without hosted-job duplication |
| `source-scan` | hosted preflight and the start of local `security` | Scans source and lockfiles before dependency installation/resolution. A failed preflight prevents downstream work; scheduled dependants run under `always()` and fail red rather than becoming skipped |
| `security` | every repository `scripts/ci-local.sh security` | Current-tree gitleaks plus a genuine structured canary in all four repositories; complete committed dependency policy in the Rust workspaces and npm auditing in Portal; strict offline/no-ignore zizmor. Component hygiene only: no result registers a release scan receipt, so `release.scan.*` stays BLOCKED |
| `CI / required` | GitHub mirror workflow aggregator | Runs source scan first, fans out five atomic jobs in parallel, then rejects any failed, skipped, cancelled, missing, dirty, wrong-subject, wrong-command/tool, malformed, or hash-mismatched observation. PR workflows are secretless/read-only; this context is prepared locally but has no hosted-run or ruleset read-back yet |
| scheduled diagnostics | GitHub mirror schedule | Full-history secrets, external links, fresh advisories, coverage, and macOS 15/Windows 2025 compile-and-typed-refusal. Every job depends on a successful source scan and fails red if that prerequisite is not successful |
| `family` | hub only (`just check-family`) | Requires four clean ordinary checkouts; runs BulletGit required and builds the exact canonical `bullet-gitd` under BulletGit's assigned Rust 1.97.1 in a fresh non-inheriting shell, Kernel required plus its explicit family inventory with that absolute daemon path, Portal required plus real-farmd browser proof, then cross-family contract drift and the Hub wire-contract/pinned-model lane; rechecks cleanliness and emits a deterministic unsigned `bullet.family-ci-observation.v1` over immutable commit/tree subjects and normalized nonzero outcome counters. Raw report/binary hashes are checked only during the same invocation and are excluded from the stable identity. The observation is `DIAGNOSTIC_ONLY`, has `release_authority:false`, and does not repeat Hub standalone `required` |
| `family-contract` | hub only (`just family-contract`; also `required.family-contract` in the sealed `check required` catalog) | Compatibility alias for the same ordered family proof; it does not repeat Hub contract execution |
| `egress` | bullet-kernel `ops/ci/egress.sh` | Linux user+net namespace, `slirp4netns` uplink, in-namespace nftables default-drop, host CONNECT proxy; exits 78 (neutral) when a tool or unprivileged namespaces are absent, never green without running the probes |
| `nightly` | bullet-kernel `ops/ci/nightly.sh` | Per provider in `BULLET_LIVE_PROVIDERS`: the feature-gated refusal test plus the positive live-conformance half. Default marker mode points the positive half at a marker script, so any spawn under the committed policy fails the lane and `POLICY_LIVE_ADMISSION_DISABLED` (exit 78) is the expected neutral outcome. `BULLET_LIVE_REAL=1` with an absolute `BULLET_POLICY_PATH` is operator real mode: the resolved real binary is used and receipts are kept under `target/live/<provider>/<utc>/` |
| `toolchain-msrv`, `toolchain-pinned` | bullet-kernel and bullet-git `just toolchain-msrv` (Rust 1.95.0); hub `just toolchain-pinned` (Rust 1.97.1) | The second toolchain named by the [release contract](release.md). Each repository's `rust-toolchain.toml` and hosted CI pin exactly one toolchain (hub 1.95.0; Kernel and BulletGit 1.97.1), so these explicit local lanes are the only place the other required toolchain is exercised. Each runs the exact receipt argv and environment from `src/check/release_evidence/verify.rs` (`cargo build --workspace --all-targets --locked`, then `cargo test --workspace --all-targets --locked --no-fail-fast`, with `CARGO_INCREMENTAL=0`, `CARGO_NET_OFFLINE=true`, `RUSTC=<absolute rustc>`, `RUSTUP_TOOLCHAIN=<version>`) in an isolated `target/toolchain-<version>/`, then writes the ignored machine-local observation `.bullet-family/toolchain-<version>-<repository>.json` beside its two raw output logs. A missing rustup toolchain, `b3sum` 1.8.2, or `jq` is a typed refusal (exit 1), never a skip; a red build or test fails the lane after the observation is written. Compile and test only: no fmt, clippy, or contract step. The observation is an input for a future operator-signed `release.rust-msrv-1-95` / `release.rust-pinned-1-97-1` receipt and is never itself a receipt; both gates stay `BLOCKED` |
| `release-truth` | hub `just release-truth` (`scripts/release-truth.sh write`) | Renders the later maximum-scope `universal-v1` profile (43 selected gates, bound to all 46 global crosswalk rows) with a scratch absolute registry and `--report --portable` into `docs/assurance/release-truth.generated.md`; `scripts/release-truth.sh check` runs inside Hub `docs`. Exit 3 (`BLOCKED`) is preserved; the page is a projection, not a first-GA decision or receipt. Historical `legacy-v1-26` remains a separate diagnostic only |
| prepared Jeryu jobs | hub `ci.toml` | Eight inactive nodes prepare activation, source scan, five atomic lanes, and required convergence while every node fails closed. They become authoritative only after forge ratification, immutable provisioning, predecessor-result/artifact semantics, hosted execution, and API read-back |

`nightly` and `egress` have no hub or family wrapper; run them from the Kernel
checkout. The toolchain lanes have no hosted job and are not in any `check`
catalog: they are operator/developer lanes that produce observations only. Neither produces `LIVE_PROOF` today: no provider has a live receipt.

A contract check generates into a temporary directory and diffs tracked
output; it does not repair drift while claiming to verify it. Neither command
layer may translate a skipped, missing, unsupported, flaky, infrastructure, or
`UNKNOWN` result into success.

The `release receipt-verify` component validates canonical receipt/policy
bytes, the exact policy digest, an admitted OpenSSH Ed25519 signer, signature
namespace, and validity interval. Its success message explicitly says
`contract only`; it does not provide trusted time, revocation/custody,
kind-specific semantic adjudication, registry/replay protection, or gate
registration. Those independent subjects must exist before a verified receipt
can clear any product gate.

## Evidence ownership

| Subject | Producer | Independent check |
| --- | --- | --- |
| Rust/TypeScript unit | owning repository | required profile with locked dependencies |
| Wire contract | DTO/schema generator | cross-repository golden JSON/hash vectors |
| Candidate | BulletGit | verifier clean reconstruction and proof-root recomputation |
| Provider contract | pure adapter transcript | frozen official/runtime schema fixture plus adversarial mutation suite |
| Provider live | admitted supervised runner | exact provider/version/profile/environment/canary receipt |
| Effect | effect broker | remote read-back and reconciliation |
| Portal | generated client and projection | built embedded portal against a real farmd |
| Package/install | release builder | separate verifier and fresh-host installer smoke |

Writer state, provider prose, portal color, HTTP 2xx, Git push success, or a
simulator cannot satisfy an independent gate.

## Mandatory release suites

Release acceptance requires deterministic negative coverage at every relevant
boundary for:

- lease races, fence replay, expiry, supersession, and authority outage;
- crash injection at SQL, CAS, journal, and active-generation switch points;
- duplicate/conflicting paths, stale preimages, traversal, symlink/reparse and
  Unicode/case collisions, hostile Git configuration/filters, and unsafe cleanup;
- Candidate sensitivity for every bound field and proof invalidation after
  rebase or other subject change;
- provider malformed, duplicate, delayed, cancelled, timed-out, and oversized
  events, process-tree cleanup, and canary-secret absence;
- effect success with lost response, exact read-back adoption, and no second
  write;
- SSE gaps, failed snapshots, exclusive cursor replay, reconnect, retention
  gaps, malformed 200 responses, and command timeout reconciliation; and
- hub-only setup twice in a fresh home with exact clean OIDs and zero tracked
  changes.

This is the acceptance inventory, not a claim that every suite already exists.
Current implementation and evidence gaps are tracked by G1–G4 and G9–G15 in
[`assurance/product-gaps.md`](assurance/product-gaps.md) and by Waves 0–7 in
[`assurance/closure-roadmap.md`](assurance/closure-roadmap.md).

The deterministic demo must use real child-process boundaries and a protected
local forge simulator. It proves only synthetic/component behavior until the
same exact-subject scenario is replayed through admitted production adapters.

## Provider conformance

Offline provider tests are pure transcript machines. They may bind a frozen
native message subset and reject everything else. They must not spawn a CLI,
read OAuth state, inherit ambient credentials, use the network, write a
workspace, convert free text to a proposal, or mark model output verified.
Enabling a `live` Cargo feature without signed admission must run a non-ignored
refusal test, not a hidden quota-spending smoke test.

Live conformance requires all of:

1. absolute canonical executable path, exact digest/version, and runtime probe;
2. signed short-lived authority binding provider, profile, policy, request,
   scope, budget/quota, environment, runner, Attempt, fence, and expiry;
3. ephemeral HOME and the minimum read-only provider credential material;
4. allowlisted environment, provider-only egress, no SCM/cloud/SSH secrets;
5. bounded JSONL frames, deadlines, cancellation agreement, and full process-
   tree termination;
6. schema-valid `PatchProposal` with exact admitted gate IDs; and
7. receipts proving canary host secrets are absent from environment, output,
   proposal, artifacts, and logs.

Each provider/version/profile certifies only itself. First-GA `self-hosted-v1`
selects Claude. Codex, Cursor, and Antigravity need separate conformant live
receipts for their profiles; later `universal-v1` selects all four.

### Implemented path

The positive live-conformance path is implemented in
`bullet-kernel/crates/application/src/live_conformance/` (`mod.rs`, `steps.rs`;
kernel `ba485d5`, loader mirror `0d848f6`) and driven by the operator entry
point `bullet provider live-conformance --data-dir <abs> --provider
{claude,codex,cursor,agy} [--executable <abs>]`. It records thirteen ordered
step statuses, never collapsed: `POLICY`, `OPERATOR_KEY`, `LEASE`, `ADMISSION`,
`MINT`, `VERIFY_GRANT`, `ADMIT_SIGNED`, `EGRESS_PREPARE`, `ADMIT_EGRESS`,
`REQUIRE_DISPATCH`, `DISPATCH`, `CANARY_SCAN`, `PONG_MATCH`; a step that did not
run is `NOT_RUN`. Exit 0 is a sealed `PONG` receipt; exit 78 is a neutral policy
refusal (`POLICY_LIVE_ADMISSION_DISABLED` under the committed generation-1
policy, before any key read, probe, namespace, or spawn); any other exit names
the failing step in the receipt. The policy step reads the production loader
(v1alpha1, or an operator-ratified v1alpha2 generation per ADR 0012 loaded from
`BULLET_POLICY_PATH`). The Kernel suite exercises the full path only against a
fake provider process; no `LIVE_PROOF` receipt exists for any provider. The
operator procedure is [`runbooks/live-conformance.md`](runbooks/live-conformance.md).

## Live forge and effect tests

Live tests never alter the running Jeryu service to make a test pass. They use a
separately authorized test repository, exact idempotency key, protected target,
read-back, and reconciliation. GitHub tests use an installed GitHub App with
least-privilege repository credentials; a personal token is not equivalent
release evidence.

A lost response after remote success records `UNKNOWN`. The test must prove
read-back adopts that exact effect and that the broker performs no duplicate
write. Jeryu is mandatory for first-GA local production proof. GitHub and both
GitLab variants are independently certified effect adapters selected by their
own profiles and by later `universal-v1`.

## CI ownership and portability

The inactive Jeryu `ci.toml` jobs and GitHub mirror workflows both delegate to
the repository-local scripts; only those scripts define lane behavior. Jeryu
runners are not activated, no public mirror run exists, and no branch ruleset
has been read back, so neither surface is currently hosted evidence. Every
action uses a full commit SHA; downloaded tools are version/checksum pinned;
PRs have `contents: read`, disabled checkout credentials, and no
`pull_request_target`. Required workflows have no path filter. Only pull-request
runs cancel superseded work; main, tag, merge-group, family-lock, and release
runs are never deliberately cancelled. Thin shell/Just/workflow wrappers may
dispatch; policy, lock, generation, proof, and release decisions remain in
Rust/TypeScript.

Each atomic job emits unsigned `bullet.ci-observation.v1` diagnostics bound to
its commit, tree, cleanliness, exact commands, required tool versions,
outcomes, and sanitized artifact hashes. The `CI / required` aggregator
re-derives the expected tree and hashes downloaded artifacts. These files are
diagnostics, not Bullet Evidence or release receipts. Cache writes are absent
from the prepared workflows; any future cache must be main-push-only, and
release jobs remain cache-free.

Nightly exists only for meaningful fuzz, soak, or admitted live-adapter work.
There is intentionally no green no-op nightly; the Kernel `nightly` lane above
fails on any provider spawn under the committed policy. Platform packages may
be built for Linux x86_64/aarch64, macOS x86_64/arm64, and Windows x64, but
non-Linux mutation remains fail-closed until equivalent containment passes.

## Local verification order

From a clean canonical family checkout:

```bash
bullet-family checkout verify
bullet-family lock verify --tag <version>
bullet-family check fast
bullet-family check required
registry="$(mktemp -d)"
bullet-family check release --profile self-hosted-v1 --receipts "$registry"
bullet-family check release --profile universal-v1 --receipts "$registry"
bullet-family check release --profile legacy-v1-26 --receipts "$registry" --report --portable
rmdir "$registry"
```

Every release invocation requires an explicit profile and absolute receipt
registry. `self-hosted-v1` is the first-GA target; `universal-v1` is the later
maximum-scope composition. `legacy-v1-26` exists only to preserve the
historical 26-gate operator projection. All remain nonzero
while required live, package, signing, recovery, security, or platform receipts
are absent; the portable report keeps exit 3. Current subjects, receipt counts,
and blockers are maintained in the
[closure roadmap](assurance/closure-roadmap.md) and [release index](release.md).
