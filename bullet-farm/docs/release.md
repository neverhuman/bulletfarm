# Bullet Farm release contract

Status: **BLOCKED — no V1 release candidate is authorized**  
Owner: Bullet Farm maintainers  
Last reviewed: 2026-08-25

Applies to: the four-repository Bullet Farm family

This document is the short release index. It does not replace generated wire
contracts, the family lock, policy registry, test maps, or signed receipts.
Historical design material under `docs/spec/` has no release authority.
Unreleased product changes are recorded in the [changelog](../CHANGELOG.md).

## Evidence classes

| Class | Proves | Does not prove |
| --- | --- | --- |
| `COMPONENT_PROOF` | One crate, service, or portal surface passed its mapped tests | Cross-process transaction safety |
| `SYNTHETIC_PROOF` | Deterministic simulator behavior | A provider, forge, or production mutation |
| `TRANSACTION_PROOF` | One exact offline five-plane transaction with independent receipts | External provider or forge conformance |
| `LIVE_PROOF` | An admitted provider or effect adapter passed the same exact-subject transaction | A release on every platform |
| `RELEASE_PROOF` | Packages, installer, recovery, security, signatures, provenance, and required live profiles passed from tagged bytes | Future versions or untested environments |

An exit code, model statement, process shutdown, HTTP success, branch push, or
pull request is never release evidence by itself. `UNKNOWN`, timeout, zero
tests, unsupported, skipped, flaky, or infrastructure error never equals
`VERIFIED`.

## Deployment stages

| Stage | Required baseline | Additional requirement |
| --- | --- | --- |
| `self-hosted-v1` | **First GA:** exact offline transaction; conformant Claude; protected local Jeryu effect; signed Ubuntu 24.04 x86_64/systemd distribution | Install, operations, recovery, security, Linux containment, supply chain, and profile-bound receipt admission |
| `evolution-v1` | Passing `self-hosted-v1` | Separate study, shadow/canary, promotion, rollback, and drift receipts; never implied by another profile |
| `provider-*`, forge-adapter, and `platform-*` | One exact independent certification slice after the self-hosted substrate; Wave 10 follows evolution Wave 9 in implementation order | One slice never certifies another; the current structural registry cannot PASS |
| `universal-v1` | Later compatible composition of `self-hosted-v1`, all four providers, Jeryu, GitHub, both GitLab adapters, and five platforms | Fresh composition smoke and closure checks; it does not include evolution, team, or saga |
| `team-v1` | Passing `self-hosted-v1` | PostgreSQL/workload-mTLS partition, failover, freeze, and restore receipts |
| `saga-v1` | Passing `team-v1` | Cross-repository staged Candidate, quarantine, compensation, and forward-repair receipts |
| `linux-preview` | A subset of Ubuntu/Jeryu/Claude diagnostics | Non-release only; it cannot replace `self-hosted-v1` or authorize a tag |

Multi-tenant SaaS is outside this roadmap. A provider/model/adapter/profile is
eligible only under its own exact, unexpired certification; one provider's
receipt never certifies another provider or profile.

There is no implicit `v1-ga` profile. The first-GA executable decision is
`self-hosted-v1`. `universal-v1` is a later maximum-scope composition. Every
narrower named profile authorizes only its own exact dependency closure.

## Current hard blockers

| Gate | Status | Evidence needed to clear it |
| --- | --- | --- |
| Hub-only installation | `BLOCKED` | The checked-in alpha.4 lock is schema 2 and intentionally rejected. Descriptor-relative private staging, no-replace publication, fsync, bounded no-follow cleanup, sealed Linux execution of admitted Cargo/Node/Bash/npm/setup-mutation/family-lock/checkout Git subjects, a build-free default-refusing wrapper, and two-run fixture setup are committed. The wrapper's operator-selected external executable is not signed package admission; clone transport Git/helpers, non-Git traversal, and transient/between-child repository stability remain open. Publish a real schema-3 lock with authenticated Jeryu URL/slug and signed exact subjects, then replay the invariant using a signed prebuilt installer from tagged release bytes in a fresh home |
| Production Kernel transaction | `BLOCKED` | Atomic lease/command/event/outbox, snapshots, authenticated ingress, exact offline reconciliation, admitted verifier gates, bounded verifier transport, PASETO launch-grant admission, Linux egress isolation, six read-only operational projections including revision-one Context Lineage, a policy-gated common provider path, and a signed internal UDS lease RPC/client component are committed. That component binds registered Runner ID/epoch to `SO_PEERCRED` UID, pins farmd UID plus socket GID/device/inode across connect, and persists server grant/nonce state. A retained public component now proves authenticated idempotent `run_demo` admission, farmd restart, packaged Portal replay/poll, registered same-UID UDS dispatch, a bounded exact worker, admitted retained fixture receipt, durable exact-digest `UNKNOWN`, and worker restart read-back. It remains private `COMPONENT_PROOF` / `UNSIGNED_FIXTURE` custody, not operator-admitted long-lived custody, distinct service identity, production provider/effect dispatch, process-level response-loss or twelve-boundary chaos evidence, or transaction proof. Current v1alpha1 policy still refuses before provider spawn. The unauthenticated `HttpLeaseClient` must not be exposed. Durable mutation reservation, source/Attempt reconstruction authority, production JSON-RPC/effect dispatch, remaining normalized cognitive truth, CAS/GC, admitted restore, cross-plane crash receipts, and `TRANSACTION_PROOF` remain |
| Production BulletGit transaction | `BLOCKED` | Durable CAS/journal, generation-atomic apply, preservation-bound cleanup, honest post-delete UNKNOWN outcomes, and a complete provenance-bound local Candidate identity are committed; positive online authority/settlement, complete Integration proof, immutable shared-wire tag consumption, and reviewed tagged `jeryu-gitd` remain |
| Offline five-plane proof | `BLOCKED` | One signed `TRANSACTION_PROOF` covering authority, runner death/salvage, independent verification, ambiguous-effect reconciliation, protected integration, preservation, and truthful portal projection |
| Jeryu live effect | `BLOCKED` | Engineering must land typed capability probes, protected integration/read-back reconciliation, and semantic receipt admission; then the operator restores scoped authentication on an unmodified pinned forge and registers exact integration, reconciliation, backup/restore, and drift receipts |
| GitHub live effect | `BLOCKED` | Engineering must land the typed capability/delivery/check/integration/read-back/reconciliation adapter and semantic receipt admission; then the operator configures a GitHub App test repository with role-separated credentials and registers the exact-subject receipt for the independent GitHub adapter profile and later `universal-v1`. GitHub is not source authority and is not selected by first-GA `self-hosted-v1` |
| Provider conformance | `BLOCKED` | The four bounded offline adapters, zero-spawn refusal, launch-grant, and egress components exist, but complete schema-3 policy/enrollment-anchor admission, provider onboarding/runtime probes, and semantic sealed-receipt registration remain local engineering. The operator must then ratify the policy/enrollments and supply exact provider executables, profiles, credentials, and native live runs. First-GA `self-hosted-v1` requires Claude; Codex, Cursor, and Antigravity require independent profile receipts and are all selected again by later `universal-v1` |
| Security quality | `BLOCKED` | The 65 (raw 66), 5-cap, 48-finding Hub Jankurai result is a historical unsigned component checkpoint recorded in G8, not a current-HEAD or release result. Moving working-tree diagnostics may score differently and are never promoted as evidence. Hosted CI also lacks a portable checksum-pinned Jankurai artifact; do not replace that gap with a machine-local or skip-green lane. Release needs at least 90, zero caps/hard findings, and all required scans |
| Release supply chain | `BLOCKED` | A Linux component verifies an exact non-circular signed five-target universal manifest and every declared byte subject, then can safely materialize one exact signed archive at an absent destination. Portal now emits a deterministic clean-commit bundle manifest, and farmd has manifest-verified same-origin embedding as component proof. Neither component is a package builder, installer, activation/rollback mechanism, or release authority. First-GA `self-hosted-v1` needs one signed Ubuntu archive in a new profile-preserving manifest; later `universal-v1` needs all five. Reproducible package production, signed package integration, semantic binary/SBOM/provenance validation, package signatures from protected release keys, installer smoke, and tagged release receipts remain |
| Platform containment | `BLOCKED` | Linux production containment plus fail-closed proof on every other packaged platform until an equivalent native backend passes |

Missing credentials produce a neutral, unregistered live lane only when that
lane is not required for the requested profile. Missing required tools,
adapters, receipts, or signatures fail the release.

Every release command requires an explicit profile and absolute receipt
registry. The first-GA target is `self-hosted-v1`; the later maximum full-
vision target is `universal-v1`, which intentionally includes both GitLab
certification profiles so GitLab configurability cannot be claimed without
proof. No forge adapter substitutes for another. Every profile is currently
`BLOCKED`. `legacy-v1-26 --report
--portable` preserves the historical 26-gate projection, with all 26 rows
`BLOCKED`; it is not any release authority. `linux-preview` is likewise
a non-release diagnostic and deliberately omits required subjects. No static
placeholder or generic receipt envelope is counted as passing evidence. The
portable generated current-profile copy is
[`assurance/release-truth.generated.md`](assurance/release-truth.generated.md);
it is a drift-checked projection, not a receipt.

The public `legacy-v1-26` diagnostic is static: it ignores the supplied receipt
registry and keeps all 26 rows `BLOCKED`. It has no semantic admission command
and reads neither a registry nor a fixed machine descriptor. Profiled release
evaluation selects an explicit registry, but the current implementation proves
only structural envelope and profile-binding checks. Hub engineering still
owes kind-specific semantic verifiers, exact-family admission, and durable
replay/high-water enforcement; operators still owe independently provisioned
signer policy, trusted time, and real signed tagged receipts. Until both halves
exist, selected-registry outcomes are absent, rejected, or structurally valid
but untrusted, and no named product profile can PASS.

An older unprofiled one-gate MSRV evaluator and its fixed-descriptor tests are
quarantined component provenance only. No public command or current profile
invokes that path, and it cannot close or waive any release gate.

## Historical component checkpoints

The OIDs and any test counts below are point-in-time observations at the cited
commits, not current-HEAD totals. They remain useful provenance, but every row
is component evidence—not transaction, live, package, install, or release
evidence. Exact working-tree counts are enforced by each repository's machine
inventory ratchet and are deliberately not mirrored here as “current.” Those
local results are unsigned CI observations and clear no release gate.

| Subject | Committed receipt | Remaining authority boundary |
| --- | --- | --- |
| Kernel command worker | Kernel `77a0ecd` | Authenticated offline exact-ID execution/reconciliation only; no provider or effect dispatch |
| Kernel backup/restore | Kernel `798f0c8` | Exact receipt and quarantined offline restore only; restored state is not admitted for production use |
| Codex offline protocol | Kernel `ca376e4` | Bounded App Server transcript subset; public runtime remains blocked |
| Claude offline protocol | Kernel `c34d578` | Bounded stream-JSON transcript subset; public runtime remains blocked |
| Cursor offline protocol | Kernel `ea89929` | Bounded ACP transcript subset; native typed-extension and live conformance remain unproved |
| Antigravity offline protocol | Kernel `5badc85` | Bounded structured-output subset; native stream schema and live conformance remain unproved |
| Strict provider JSON | Kernel `1bb32bd` | Recursive duplicate-key and trailing-data refusal on all four raw paths, including Codex proposal text; no canonicalization or live authority |
| Admitted gate/verifier | Kernel `528348f` | Fixed catalog ID/argv/timeout and exact-subject E2 Evidence; only one fixture gate, with no executable digest, production framing/source admission, or multi-gate aggregation |
| Verifier transport | Kernel `365bb5d` | Bounded strict one-shot request/output, exact frame and overflow kill/reap; no JSON-RPC, signed reconstruction source, or process-tree contract |
| Signed launch grant + Linux egress | Hub `a2d6b2a`; Kernel `d388733` | Exact PASETO subject, policy key lifecycle, active-lease issuance, single-use nonce, and live namespace/nft/proxy isolation; live dispatch stays policy-disabled and no provider conformance receipt exists |
| Operator-ratifiable live policy | Hub `bf5c642`; Kernel loader mirror `0d848f6`; descendant consumers BulletGit `236f4ef`, Portal `95108e3` | v1alpha2 structural/time validation requires generation >=2 and an active provider-runner key while preserving conservative invariants. ADR 0012 is proposed; the committed policy remains v1alpha1 generation 1 and no committed policy enables live admission |
| Policy-gated provider conformance | Kernel `ba485d5`, nightly real-mode wrapper `b4735da` | At those commits, the common fail-closed orchestration and sealed step receipts ran 439 required tests with 3 intentional live skips, egress 3/3, and four-provider neutral refusal with zero spawn. The present required lane instead excludes live tests and has zero skips. Only Claude has a deep positive fake-process proof; no provider has a live receipt |
| Operational farmd projections | Kernel `529bad1`, `7cdf850`; Portal `3033b67` | At those commits, six atomic watermark-bound surfaces had generated/strictly validated clients and Portal ran 104 unit, 10 mocked-browser, and 2 real-farmd tests. The current component adds manifest-verified same-origin embedding and packaged-farmd browser proof. Six designed surfaces remain explicit UNKNOWN, successor/compression lineage and a signed package/install receipt are absent |
| Initial Context Capsule authority | Kernel `7cdf850` | Immutable revision-one identity/package membership is normalized and atomically bound to graph materialization, lease, fence, and Attempt; cross-graph replay refuses. Successor/compression lineage, task/role/fusion, quota/budget, and routing scheduler truth remain open |
| Setup transaction | Hub `94b6549`, `7efe2f3`, `3039878`, `e8f0180`, `34f3326`, `093a0e2` | Descriptor-relative source/component fixture plus sealed Linux Cargo/Node/Bash/npm, mutation-Git, family-lock verification-Git, and checkout verification-Git subjects and a build-free default-refusing wrapper; signed admission of its external executable, clone transport Git/helpers, transient/between-child object/ref/index/config/file stability, non-Git traversal, allowed-signers admission, production Jeryu/validator replay, public schema-3 authority, and prebuilt installer remain open |
| Legacy MSRV receipt admission | Hub `d762f86` | One fixed root-owned policy/descriptor/evidence path can admit exact independently signed Rust 1.95 evidence for the historical evaluator only; no real admission exists, and this mechanism cannot clear a profiled release gate |
| Release-truth report | Hub `0cc7eec` | The cited historical checkpoint rendered 26 rows. The current generated `universal-v1` projection renders 43/43 `BLOCKED` with explicit mechanical/evidence/review/deployment/survival separation and decision exit 3; the report itself cannot satisfy a gate |
| Bundle verifier/extractor | Hub `352f963`, `ba09056` | Public exact-byte verification plus a quarantined safe-materialization component; public extraction always refuses publication, and neither path owns package production, semantic admission, activation/rollback, or signing authority |
| Signed receipt verifier | Hub `143f8b9` | Canonical receipt/policy and exact OpenSSH signer/namespace/interval verification; no external policy, trusted time/revocation/custody, semantic adjudication, registry/replay, or real receipt |
| BulletGit subject/recovery contract | BulletGit `274fd6d`, `f551736`, `4c508e4` | Exact local freeze/recovery, strict wire-shaped subjects, and provenance-complete Candidate/Content identities; no immutable shared-wire tag, Kernel caller convergence, online authority/settlement, Integration proof, or production Jeryu service |
| BulletGit cleanup/CI contract | BulletGit `2d22c28`, `5dac98e` | Synced tombstone before cleanup success and fail-closed UNKNOWN after ambiguous deletion; no positive online authority/Jeryu proof and Jankurai remains below release floor |
| Generated/browser runtime truth | Kernel `35b6484`; Portal `181cd00`; Hub `601cb82` | The cited checkpoint proved generated consumed DTO validation and exact correlated UNKNOWN/SSE against Vite preview + real farmd. Manifest-verified farmd embedding is now component-proved, but no signed package serves it and Candidate/Evidence/Effect DTOs remain open |
| Portal bundle manifest | Portal `3033b67`; ignored generated root `blake3:3dd9ad08d729247b9889e6e68ee150c6aeca3e47306c7992d4cde509ab999596` | Clean commit/tree, lock, exact Git/Node/npm subjects, and emitted bundle identity at the cited checkpoint. Current same-origin embedding verifies that manifest, but the ignored `dist/` subject is not release evidence. No signed environment, archive, activation, or installer authority exists |

## Local pre-release gates

Run from the public hub in the canonical ordinary-clone family:

```bash
just fast
just contract
just check-family
just family-contract
just security
just audit
bullet-family check release --profile self-hosted-v1 --receipts /absolute/admitted-registry --json
bullet-family check release --profile universal-v1 --receipts /absolute/admitted-registry --json # later composition
bullet-family check release --profile legacy-v1-26 --receipts /absolute/empty-registry --report --portable
bullet-family check release --profile linux-preview --receipts /absolute/registry --json # non-release diagnostic
```

These commands prove repository and family prerequisites only. The real browser
lane still uses Vite preview and a separately built farmd, not extracted package
bytes. They do not
authorize a release until the transaction, live, recovery, packaging, and
signing receipts above exist. `check release` is a read-only, fail-closed
inventory of those blockers; it executes no release mutation while the
mechanisms are absent. There is intentionally no green no-op nightly.

The release build must compile at MSRV Rust 1.95 and pinned Rust 1.97.1, use
`cargo --locked` and `npm ci`, verify generated output in a temporary directory,
and start from clean signed tags matching `family.lock`.

## Installer acceptance

The release installer starts from a hub-only clone and must:

1. verify the hub tag and lock before creating member directories;
2. use Jeryu source metadata from the lock, never a sibling-path guess;
3. create ordinary clones, never Git worktrees, at exact locked commits;
4. reject dirty, symlinked, non-empty, or conflicting destinations before mutation;
5. verify signed tags, commit/tree identities, lockfiles, and generated digests;
6. use locked/offline dependency modes when requested;
7. on the supported Linux path, bound every child process by a deadline and per-stream output cap,
   terminating its full process group when either bound is crossed;
8. be idempotent; and
9. leave exact clean member OIDs and zero tracked changes after two runs in a fresh home.

`scripts/setup.sh` is a build-free, default-refusing bootstrap convenience. It does not resolve Cargo
or `bullet-family` from `PATH`; it runs only the absolute external executable selected through
`BULLET_SETUP_ADMITTED_BIN` and clears the ambient tool-selection environment. That selection alone
does not authenticate the bytes, so running the wrapper is not installer or release evidence. Release
installation requires a signed prebuilt `bullet-family` binary whose release manifest and checksums
have been verified. Before any mutation, that binary must bind the canonical absolute Cargo, Node,
npm, Git, and helper subjects it admits.

The Linux verifier is available as:

```bash
bullet-family release verify \
  --bundle /absolute/path/to/bundle \
  --allowed-signers /absolute/path/to/allowed_signers
```

It binds the manifest, schema-3 lock, five byte-sorted target entries, archive/SBOM/provenance bytes,
detached signatures, and exact Ed25519 signer status. The public extraction command performs that
verification and then unconditionally refuses publication because the required different-identity
or privileged containment backend does not exist:

```bash
bullet-family release extract \
  --bundle /absolute/path/to/bundle \
  --allowed-signers /absolute/path/to/allowed_signers \
  --target x86_64-unknown-linux-gnu \
  --destination /absolute/absent/path
```

On Linux, an exact valid bundle therefore ends in
`RELEASE_PUBLICATION_CONTAINMENT_UNAVAILABLE` with the destination still absent; off Linux, structural
verification may refuse earlier. The internal archive scanner and no-replace publisher remain quarantined
components exercised only by tests. They admit bounded regular-file/directory-only `tar.zst` or canonical stored
ZIP layouts and reject non-ASCII names, case collisions, traversal, platform-special names, links, special files,
ZIP64/multi-disk/data-descriptor ambiguity, oversize/ratio abuse, and existing destinations. Those tests do not
make the internal publisher a reachable installer surface.

This command does not interpret binary, SBOM, or provenance semantics; run an installer; activate or roll back an
installation; provision signing trust; or emit release evidence. No package builder or signed prebuilt
`bullet-family` installer has been published. Passing verification or extraction against a preassembled test fixture
is not package-production, installation, signer, platform, or release evidence.

The Rust setup/checkout mechanism and its signed local four-repository fixture implement these
rules, including two idempotent exact source setups. On Linux it copies admitted Cargo, Node, Bash,
npm, setup mutation-Git, family-lock verification-Git, and checkout verification-Git subjects into sealed read-only memfds and executes them through inherited descriptors; a
post-verification pathname swap cannot execute attacker bytes. It retains the admitted family-root descriptor,
uses 0700 descriptor-relative staging, checks root/staging identity around path-dependent children,
publishes members and the final outer manifest without replacement, fsyncs authority boundaries,
and confines cleanup with no-follow depth/entry limits. Fallible dependency, generated-contract,
and exact-family checks complete before publication; injected transaction boundaries recover from
prior state or an exact partial publication to one complete next state without replacing verified
members, while indeterminate staging/orphans remain preserved. Setup rejects unsupported platforms before mutation. Active same-UID
mutation during clone transport Git/helper use, transient or between-child repository object/ref/index/config/file
changes, non-Git work-tree traversal, and allowed-signers path admission remains beyond this descriptor boundary; a cleanup limit may
safely leave an orphan, and an error after publication requires exact setup/verify reconciliation. The
checked-in alpha.4 lock remains schema 2, so the public command still fails before mutation with
schema-3 regeneration guidance. Release evidence remains blocked until authenticated Jeryu subjects
and a signed prebuilt binary exist and the invariant is replayed from those exact release bytes. No public
schema-3 family lock or live provider, Jeryu, or GitHub receipt exists today.

The positive two-run fixture uses local source transport and a test-only exact
validator; it does not exercise production Jeryu transport or the full setup
validator. Commit `7efe2f3` closes the final-path swap for the Rust boundary's
Cargo, Node, Bash, and npm subjects, `3039878` removes the ambient Cargo
bootstrap, `e8f0180` seals setup mutation-Git, `34f3326` seals family-lock
verification Git, and `093a0e2` seals checkout verification Git against
descriptor-pinned per-child work-tree/`.git` subjects.
None authenticates the wrapper-selected external executable or the remaining
clone transport Git/helper subjects, transient/between-child repository or
non-Git filesystem identity, or allowed-signers path. Public installer
acceptance still requires a signed prebuilt and a two-run replay through
production transport and validation.

## Package matrix

First-GA `self-hosted-v1` requires the signed Ubuntu 24.04 x86_64/systemd
distribution. Independent platform profiles add Linux arm64, macOS
x86_64/arm64, and Windows x64; later `universal-v1` composes all five. The built
Portal is embedded in the Rust distribution. Every platform must refuse real
mutation until its native containment backend has equivalent release evidence.
The `linux-preview` diagnostic cannot replace the self-hosted release profile.

Every certified archive is bound to the same hub tag and family lock and
carries both SBOM formats, checksums, signatures, and provenance. The final
manifest binds the hub tag without embedding its own digest.

### Quarantined single-target builder component

The public `bullet-family release build` command always returns
`RELEASE_BUILD_CONTAINMENT_UNAVAILABLE` before argument parsing, validation, source/tool inspection, child
execution, or output creation. The quarantined internal builder component, exercised by focused tests only,
can produce one unsigned Linux x86_64 bundle from a clean four-repository committed subject: a
`bullet-farm/`-rooted deterministic `tar.zst` carrying eight locked
release binaries, including the read-only `bullet-mcpd`, with `bullet-farmd`
built `--features embedded-portal` from a scratch clone of the committed Portal
subject and its own bundle manifest; all eight direct `bin/` entries are required
and re-read as executable on Unix while package data remains non-executable; a
CycloneDX 1.6 SBOM in which every component carries a name, version, package URL,
and a license admitted from the committed `deny.toml` allow-lists; an unsigned
in-toto provenance statement recording builder identity, every input subject, and
every exact build argv; a BLAKE3 checksum manifest over every archive entry and
bundle file that is re-opened, re-parsed, and re-hashed before the build reports
success; and a canonical-JSON build manifest that binds the four repository
subjects, the family lock, the toolchain, and every artifact digest without ever
binding its own. The internal component refuses any other target with `UNSUPPORTED_RELEASE_TARGET`,
any member with tracked, untracked, or index changes with `DIRTY_SOURCE`, an
absent toolchain with `RELEASE_TOOLCHAIN_MISSING`, and a Portal bundle manifest
that disagrees with its own files with `RELEASE_PORTAL_BUNDLE_INVALID`.

This internal component is not a public builder, is not the five-archive contract, and clears no gate. It signs
nothing; signing remains OD-E and the build only prints the exact `ssh-keygen -Y
sign` commands. It writes no `release-manifest.toml`, because that frozen schema
requires all five byte-sorted targets and a schema-3 `family.lock` while this
host can honestly build one target against the checked-in schema-2 lock. It
emits CycloneDX only, because the frozen schema binds exactly one `.cdx.json`
SBOM per package and a second unbindable document would not be evidence. It
emits BLAKE3 only, because no SHA-256 implementation is pinned in `Cargo.lock`.
`bullet-family release verify` therefore refuses the bundle it produces, and
`release.package-matrix`, `release.checksums`, `release.sbom`,
`release.manifest-non-circular`, and `release.provenance` all remain `BLOCKED`.
The containment/refusal runbook is [`runbooks/release-build.md`](runbooks/release-build.md).

## Tagging rule

Do not create or advertise a V1 release tag while any required row above is
`BLOCKED`, `UNKNOWN`, or supported only by component evidence. When a
gate changes, update this index in the same reviewed transaction that adds its
independently verifiable receipt; prose alone cannot change status.
