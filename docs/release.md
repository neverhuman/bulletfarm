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
| V1 GA | Exact offline transaction; conformant Claude, Codex, Cursor, and Antigravity receipts; protected Jeryu and GitHub effect receipts; five signed archives | Install, operations, recovery, security, containment, and supply-chain gates |
| `linux-preview` | Ubuntu 24.04 x86_64/systemd, exact offline transaction, one Claude receipt, and local Jeryu integration | Non-release diagnostic slice only; it cannot authorize V1 or waive any omitted provider, GitHub, or package gate |
| `provider-*`, `github-adapter-v1`, `platform-*` | One exact certification slice | Diagnostic evidence reusable by canonical V1 only when its exact gate also appears in the 26-gate release decision |
| `team-v1` | Explicitly post-V1 | PostgreSQL/workload-mTLS partition, failover, freeze, and restore receipts |

Multi-tenant SaaS is outside this roadmap. A provider/model/adapter/profile is
eligible only under its own exact, unexpired certification; one provider's
receipt never certifies another provider or profile.

## Current hard blockers

| Gate | Status | Evidence needed to clear it |
| --- | --- | --- |
| Hub-only installation | `BLOCKED` | The checked-in alpha.4 lock is schema 2 and intentionally rejected. Descriptor-relative private staging, no-replace publication, fsync, bounded no-follow cleanup, sealed Linux execution of admitted Cargo/Node/Bash/npm/setup-mutation/family-lock/checkout Git subjects, a build-free default-refusing wrapper, and two-run fixture setup are committed. The wrapper's operator-selected external executable is not signed package admission; clone transport Git/helpers, non-Git traversal, and transient/between-child repository stability remain open. Publish a real schema-3 lock with authenticated Jeryu URL/slug and signed exact subjects, then replay the invariant using a signed prebuilt installer from tagged release bytes in a fresh home |
| Production Kernel transaction | `BLOCKED` | Atomic lease/command/event/outbox, snapshots, authenticated ingress, exact offline reconciliation, admitted verifier gates, bounded verifier transport, PASETO launch-grant admission, Linux egress isolation, six read-only operational projections including revision-one Context Lineage, and a policy-gated common provider path are committed. Current v1alpha1 policy refuses before provider spawn. The unauthenticated `HttpLeaseClient` must not be exposed; signed full-subject internal lease transport, durable authority epoch/budget reservation, source/Attempt reconstruction authority, production JSON-RPC/effect dispatch, remaining normalized cognitive truth, CAS/GC, admitted restore, and cross-plane crash receipts remain |
| Production BulletGit transaction | `BLOCKED` | Durable CAS/journal, generation-atomic apply, preservation-bound cleanup, honest post-delete UNKNOWN outcomes, and a complete provenance-bound local Candidate identity are committed; positive online authority/settlement, complete Integration proof, immutable shared-wire tag consumption, and reviewed tagged `jeryu-gitd` remain |
| Offline five-plane proof | `BLOCKED` | One signed `TRANSACTION_PROOF` covering authority, runner death/salvage, independent verification, ambiguous-effect reconciliation, protected integration, preservation, and truthful portal projection |
| Jeryu live effect | `BLOCKED` | Operator-restored authentication and read-back/reconciliation receipt; the running forge must not be modified to work around missing capability |
| GitHub live effect | `BLOCKED` | Configured GitHub App test repository and exact-subject integration/reconciliation receipt; V1 GA requires this effect proof even though GitHub is not source authority |
| Provider conformance | `BLOCKED` | The common policy→key→lease→admission→grant→egress→read-only-turn→canary→receipt path is committed for all four providers. Current v1alpha1 policy yields neutral refusal and zero spawn. Real Claude, Codex, Cursor, and Antigravity identity/profile admission, native behavior, settlement, teardown, and separately conformant live receipts remain mandatory for V1 GA |
| Security quality | `BLOCKED` | The latest deterministic Hub Jankurai report is 58 (raw 58), with 10 caps and 44 findings: 29 high/hard and 15 medium/soft. The score, caps, and hard findings block release. Hosted CI also lacks a portable checksum-pinned Jankurai artifact; do not replace that gap with a machine-local or skip-green lane. Release needs at least 90, zero caps/hard findings, and all required scans |
| Release supply chain | `BLOCKED` | A Linux component verifies an exact non-circular signed five-target manifest and every declared byte subject, then can safely materialize one exact signed archive at an absent destination. Portal now emits a deterministic clean-commit bundle manifest binding its lock, Git/Node/npm subjects, and emitted files. Neither component is a package builder, installer, activation/rollback mechanism, or release authority. Reproducible package production, Rust embedding, semantic binary/SBOM/provenance validation, package signatures from protected release keys, installer smoke, and tagged release receipts remain |
| Platform containment | `BLOCKED` | Linux production containment plus fail-closed proof on every other packaged platform until an equivalent native backend passes |

Missing credentials produce a neutral, unregistered live lane only when that
lane is not required for the requested profile. Missing required tools,
adapters, receipts, or signatures fail the release.

The canonical V1 command is the unprofiled
`bullet-family check release --json`; it contains exactly 26 gates and all are
currently `BLOCKED`. `bullet-family check release --report` renders that same
decision. Named profiles are fail-closed diagnostic slices only;
`--profile linux-preview` deliberately omits frozen V1 requirements and can
never authorize a release. No static placeholder or generic receipt envelope
is counted as passing evidence. The portable generated
copy is [`assurance/release-truth.generated.md`](assurance/release-truth.generated.md);
it is a drift-checked projection, not a receipt.

Exactly one gate, `release.rust-msrv-1-95`, has a semantic receipt-admission
path. Its absence still produces the blocker above; the other 25 gates remain
unconditionally blocked. Production discovery reads only the fixed
`/etc/bullet-farm/release-msrv-1-95-admission.toml` descriptor. That
root-owned descriptor selects a separately root-owned policy, an external
evidence directory, and distinct source-tag, build-attestor, and trusted-time
roots. Environment variables and repository files cannot redirect it. Every
operator input and ancestor is admitted with no-follow descriptor reads,
root ownership, non-writable mode, single-link identity, and bounded stable
bytes. The three roots must contain distinct actual Ed25519 keys, not merely
different pathnames or principals.

The admitted evidence directory uses fixed receipt, detached-signature, and
trusted-time filenames. Its canonical typed payload binds the schema-3 lock,
current clean signed Hub/member commits and trees, every dependency-lock
digest, exact admitted Rust 1.95 rustc/cargo bytes, and the fixed locked/offline
build and test argv for all three Rust workspaces. Every build and test
observation must be nonempty with exit zero, zero failures, and zero skips.
The independently signed time observation binds the receipt digest, nonce,
policy, gate, expiry, future-skew limit, and freshness window. A generic result
digest, a self-selected signer file, or a receipt from another subject cannot
clear the gate. Even when this one gate is receipted, the family release remains
blocked until the remaining 25 gates have their own kind-specific authority.

## Current component receipt snapshot

These reviewed commits are component evidence, not release or live evidence:

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
| Policy-gated provider conformance | Kernel `ba485d5`, nightly real-mode wrapper `b4735da` | Common fail-closed orchestration and sealed step receipts; required 439/439 with 3 intentional live skips, egress 3/3, and four-provider neutral refusal with zero spawn. Only Claude has a deep positive fake-process proof; no provider has a live receipt |
| Operational farmd projections | Kernel `529bad1`, `7cdf850`; Portal `3033b67` | Six new atomic watermark-bound surfaces with generated/strictly validated clients, including revision-one Context Lineage; Portal 104/104 unit, 10/10 mocked browser, and 2/2 real farmd. Six designed surfaces remain explicit UNKNOWN, successor/compression lineage and the generated AJV root are absent, and there is no packaged runtime |
| Initial Context Capsule authority | Kernel `7cdf850` | Immutable revision-one identity/package membership is normalized and atomically bound to graph materialization, lease, fence, and Attempt; cross-graph replay refuses. Successor/compression lineage, task/role/fusion, quota/budget, and routing scheduler truth remain open |
| Setup transaction | Hub `94b6549`, `7efe2f3`, `3039878`, `e8f0180`, `34f3326`, `093a0e2` | Descriptor-relative source/component fixture plus sealed Linux Cargo/Node/Bash/npm, mutation-Git, family-lock verification-Git, and checkout verification-Git subjects and a build-free default-refusing wrapper; signed admission of its external executable, clone transport Git/helpers, transient/between-child object/ref/index/config/file stability, non-Git traversal, allowed-signers admission, production Jeryu/validator replay, public schema-3 authority, and prebuilt installer remain open |
| MSRV receipt admission | Hub `d762f86` | One fixed root-owned policy/descriptor/evidence path can admit exact independently signed Rust 1.95 semantic evidence; no real admission exists, and this mechanism cannot clear any of the other 25 release gates |
| Release-truth report | Hub `0cc7eec` | Deterministic 26-row operator projection with explicit mechanical/evidence/review/deployment/survival separation and decision exit 3; 0/26 receipts remain and the report itself cannot satisfy a gate |
| Bundle verifier/extractor | Hub `352f963`, `ba09056` | Exact-byte verification and safe absent-destination materialization only; no package production, semantic admission, activation/rollback, or signing authority |
| Signed receipt verifier | Hub `143f8b9` | Canonical receipt/policy and exact OpenSSH signer/namespace/interval verification; no external policy, trusted time/revocation/custody, semantic adjudication, registry/replay, or real receipt |
| BulletGit subject/recovery contract | BulletGit `274fd6d`, `f551736`, `4c508e4` | Exact local freeze/recovery, strict wire-shaped subjects, and provenance-complete Candidate/Content identities; no immutable shared-wire tag, Kernel caller convergence, online authority/settlement, Integration proof, or production Jeryu service |
| BulletGit cleanup/CI contract | BulletGit `2d22c28`, `5dac98e` | Synced tombstone before cleanup success and fail-closed UNKNOWN after ambiguous deletion; no positive online authority/Jeryu proof and Jankurai remains below release floor |
| Generated/browser runtime truth | Kernel `35b6484`; Portal `181cd00`; Hub `601cb82` | Generated consumed DTO validation plus exact correlated UNKNOWN/SSE proof against Vite preview + real farmd; not Rust-embedded or package-served, and Candidate/Evidence/Effect DTOs remain open |
| Portal bundle manifest | Portal `3033b67`; ignored generated root `blake3:3dd9ad08d729247b9889e6e68ee150c6aeca3e47306c7992d4cde509ab999596` | Clean commit/tree, lock, exact Git/Node, whole npm tree, and emitted bundle identity; `bundle:generate` and `bundle:check` exit 0. The manifest lives under ignored `dist/` and is not release evidence. No signed environment, Rust embedding, archive, activation, or installer authority |

## Local pre-release gates

Run from the public hub in the canonical ordinary-clone family:

```bash
just fast
just contract
just check-family
just family-contract
just security
just audit
bullet-family check release --json
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
detached signatures, and exact Ed25519 signer status. After that complete verification, the separate
component extractor can materialize one signed target into an absent canonical destination:

```bash
bullet-family release extract \
  --bundle /absolute/path/to/bundle \
  --allowed-signers /absolute/path/to/allowed_signers \
  --target x86_64-unknown-linux-gnu \
  --destination /absolute/absent/path
```

Extraction is currently Linux GNU only. It re-hashes the selected signed archive into an immutable descriptor,
admits a bounded regular-file/directory-only `tar.zst` or canonical stored ZIP layout, stages and fsyncs the complete
tree beside the destination, and publishes with descriptor-relative no-replace rename. It rejects non-ASCII names
(therefore every Unicode normalization ambiguity), case collisions, traversal, platform-special names, links and
special files, ZIP64/multi-disk/data-descriptor ambiguity, oversize/ratio abuse, and any existing destination. A
post-rename parent-fsync failure returns `RELEASE_PUBLICATION_UNKNOWN`; the destination is the complete next tree.
Intermediate bundle-directory replacement remains outside the verifier's path-pinning guarantee, but substituted
archive bytes cannot reach a parser unless their exact size and signed BLAKE3 digest match.

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
and exact-family checks complete before publication; injected transaction boundaries recover to
prior or complete next state. Setup rejects unsupported platforms before mutation. Active same-UID
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

V1 GA requires archives for Linux x86_64/aarch64, macOS x86_64/arm64, and
Windows x64. The built Portal is embedded in the Rust distribution. Linux is
the initial production runner. Other packages must refuse real mutation unless
their native containment backend has equivalent release evidence. The
`linux-preview` diagnostic covers only Ubuntu x86_64 and cannot waive the other
four archives.

Every certified archive is bound to the same hub tag and family lock and
carries both SBOM formats, checksums, signatures, and provenance. The final
manifest binds the hub tag without embedding its own digest.

## Tagging rule

Do not create or advertise a V1 release tag while any required row above is
`BLOCKED`, `UNKNOWN`, or supported only by component/synthetic evidence. When a
gate changes, update this index in the same reviewed transaction that adds its
independently verifiable receipt; prose alone cannot change status.
