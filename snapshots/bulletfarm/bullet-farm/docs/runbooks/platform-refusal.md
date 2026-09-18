# Platform refusal — what each binary does off the supported runner

Status: **Linux GNU is the only runner that mutates; every other platform fails closed before mutation**  
Owner: Bullet Farm maintainers  
Last reviewed: 2026-08-26
Component receipt baselines (minimum; replay current-head lanes before use): bullet-farm `d762f86`, bullet-kernel `0109a90` (harness-egress, launch-grant key custody, farmd
worker token), bullet-git `236f4ef` (workspace generations/preservation)

First-GA `self-hosted-v1` selects one Ubuntu 24.04 x86_64/systemd archive and a
Linux production-containment receipt. Independent platform profiles later add
Linux arm64, macOS x86_64/arm64, and Windows x64; `universal-v1` composes those
five targets. The existing `REQUIRED_TARGETS` verifier in `src/release/schema.rs`
is a frozen five-target universal-envelope component: it is incompatible with
and cannot admit the first-GA one-target package. An archive existing for a
platform never authorizes execution there. The `cfg` gates below select on
OS/libc, not on architecture; no aarch64 Linux receipt exists either.

Every refusal below is produced before authority-bearing mutation. Some read-only commands first admit and read
their subjects or run a bounded signature verifier; that work is not installation or publication. Each
platform-specific refusal was confirmed by reading the named source; none can be observed on this Linux GNU host,
so no exit was recorded for them here. The hub codes map to exit `2` through the default arm of
`CoordError::exit_code` (`src/coord/mod.rs`); `UNSUPPORTED_SCHEMA` is exit `4`.

## 1. Hub (`bullet-family`)

| Command | Supported | Elsewhere | Reason code | Where |
| --- | --- | --- | --- | --- |
| `setup` | Linux GNU | refuses before `AdmittedRoot::open` and before reading the lock | `UNSUPPORTED_PLATFORM_CONTAINMENT` — "setup cannot mutate on this platform until process-tree termination and atomic no-replace publication have equivalent release evidence" | `src/setup.rs` `require_setup_containment` (the first call in `run`) |
| `setup` staging/publication | Unix | descriptor-relative staging, manifest publication, and `finish` all refuse | `UNSUPPORTED_PLATFORM_CONTAINMENT` | `src/setup/transaction.rs` non-Unix module |
| `checkout verify` exact working-tree bytes/modes | Unix | refuses | `UNSUPPORTED_PLATFORM_CONTAINMENT` — "exact working-tree byte/mode verification is unavailable on this platform" | `src/checkout/git.rs` `verify_exact_worktree` |
| `fuse` publication | Linux GNU | no-replace/exchange publication refuses | `UNSUPPORTED_PLATFORM_CONTAINMENT` — "atomic no-replace/exchange fusion publication is not verified on this platform" | `src/fuse/publish.rs` `require_platform` |
| `release build` | nowhere through the public CLI | always refuses before argument parsing, validation, or mutation | `RELEASE_BUILD_CONTAINMENT_UNAVAILABLE` — production needs private exact-OID reconstruction, a sealed toolchain, and a different-identity build broker | `src/release.rs` `run`, `release_build_containment_unavailable` |
| `release verify` | Linux | archive/SBOM/provenance bytes cannot be sealed into an immutable snapshot | `RELEASE_VERIFICATION_PLATFORM_UNSUPPORTED` | `src/release/verify.rs` `immutable_snapshot` |
| `release extract` | nowhere through the public CLI | on Linux, an exact valid bundle reaches the unconditional publication refusal; off Linux, structural verification can refuse first; no destination is admitted or changed | `RELEASE_PUBLICATION_CONTAINMENT_UNAVAILABLE` after successful verification, otherwise the earlier typed verification error — production publication needs a different-UID or privileged containment backend | `src/release.rs` `run` |

The internal `src/release/archive.rs` scanner and `archive/publish.rs` no-replace publisher remain quarantined
components. Their platform refusal codes and hostile-archive tests are component evidence only: the public CLI
does not call them, so they cannot produce an installation or platform-containment receipt.

Observed on this host (Linux GNU, so the platform gates pass and the next check speaks):

```text
bullet-family release build
  -> RELEASE_BUILD_CONTAINMENT_UNAVAILABLE (exit 2; before argument parsing or mutation)
bullet-family release verify --bundle /nonexistent/bundle --allowed-signers /nonexistent/allowed_signers
  -> COORD_IO_FAILED (exit 2)
bullet-family release
  -> USAGE: usage: bullet-family release verify --bundle ABSOLUTE_PATH --allowed-signers ABSOLUTE_PATH (exit 2)
bullet-family setup --root /home/…/bullet --source jeryu --cargo-bin … --node-bin … --npm-cli … --offline
  -> UNSUPPORTED_SCHEMA: family.lock schema 2 is not installable; … (exit 4, nothing staged)
```

The signed-fixture integration test additionally observes that public `release extract` verifies the exact
preassembled bundle and then returns `RELEASE_PUBLICATION_CONTAINMENT_UNAVAILABLE` while the requested destination
remains absent. That is a fail-closed component observation, not a platform or installer receipt.

The order inside `setup::run` is fixed: platform containment → admitted root → hub location → lock schema →
toolchain admission → environment → transaction. A macOS or Windows host therefore never reaches the schema
refusal; a Linux GNU host never reaches mutation while the lock is schema 2.

## 2. Kernel (`bullet`, `bullet-farmd`, `bullet-harness-egress`)

| Surface | Supported | Elsewhere | Reason code | Where |
| --- | --- | --- | --- | --- |
| Provider egress isolation (`bullet-harness-egress`) | Linux with `unshare`, `nsenter`, `slirp4netns`, `nft`, `curl`, `cat`, `kill` and unprivileged user+net namespaces | `Tooling::discover()` refuses, naming the first absent tool; nothing is namespaced or spawned | `EGRESS_TOOL_MISSING` | `crates/harness-egress/src/tools.rs`, `error.rs` |
| `ops/ci/egress.sh` lane | same | exits `78` (neutral) when a tool or unprivileged namespaces are missing; it never reports green without running the probes | — | `ops/ci/egress.sh` |
| `bullet provider live-conformance` | no production adapter can dispatch today | exit `78` covers exactly two designed refusals: checked-in v1alpha1 returns `POLICY_LIVE_ADMISSION_DISABLED` at `POLICY`; structurally valid v1alpha2 returns `RUNTIME_PROBE_UNAVAILABLE` at `ADMISSION`. Both occur before operator-key read, graph/Mission, lease, or nonce writes, egress preparation, or spawn; the CLI may already have opened `ledger.sqlite`. A missing egress tool remains a future failing `EGRESS_PREPARE`, never a neutral substitute for a real probe | `POLICY_*`, `RUNTIME_PROBE_UNAVAILABLE` | `apps/bullet/src/provider.rs` (`NEUTRAL_REFUSAL = 78`); [`live-conformance.md`](live-conformance.md) §3 |
| `bullet authority keygen` / key load | Unix | refuses: "operator key custody is certified only on Unix" | `LAUNCH_GRANT_INVALID` | `crates/harness-core/src/launch_grant/keyfile.rs` |
| `bullet-farmd --worker-token-file` | Unix | daemon exits with failure: "worker token files are unavailable without descriptor-safe admission on this platform" | process exit failure | `apps/bullet-farmd/src/main.rs` |

The Kernel's provider containment (ADR 0011) is Linux namespaces + nftables. There is no macOS or Windows
backend; [`../release.md`](../release.md) requires those packages to refuse real mutation until an equivalent
native backend has release evidence.

## 3. BulletGit (`bullet-git-workspace`)

| Surface | Supported | Elsewhere | Reason code | Where |
| --- | --- | --- | --- | --- |
| Preservation seal (private 0600 seal, OS randomness, destination identity, symlink identity) | Unix | refuses | `PRESERVATION_UNSUPPORTED` — "this platform lacks an audited private-seal / random-seal / destination identity backend" | `crates/bullet-git-workspace/src/preservation_io.rs`, `repository_preservation.rs` |
| Generation switch and tree copy (replace-and-sync pointer, no-follow symlink copy) | Unix | refuses | `GENERATION_UNSUPPORTED` | `crates/bullet-git-workspace/src/tree_copy.rs` |

Because preservation must succeed before cleanup, an off-Unix host can neither preserve nor clean a workspace:
it fails closed with the workspace intact.

## 4. What an operator does with a refusal

1. Record the exact reason code and the binary subject. A refusal is the intended outcome on an unsupported
   host; it is not a bug report and it is not evidence that the host is unsafe or safe.
2. Do not work around it by patching `cfg` gates, copying archives across platforms, or running the Linux
   binary under an emulation layer and calling the result a receipt. `release.platform-containment` needs
   (a) the Linux production containment receipt and (b) fail-closed mutation refusal receipts on the other
   targets selected by that profile. `self-hosted-v1` selects only Linux
   x86_64; later platform profiles require their own tagged-byte refusals, and
   `universal-v1` requires the complete five-target composition.
3. On Linux GNU with a passing platform gate, the next refusal is usually the schema-2 lock
   ([`schema-removal.md`](schema-removal.md)) or missing egress tooling; follow those runbooks.

No macOS or Windows refusal has been reproduced from tagged bytes; nothing in this runbook is a
`RELEASE_PROOF` receipt.
