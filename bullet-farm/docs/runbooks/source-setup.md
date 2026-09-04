# Source setup and installation boundary

Status: **existing-family contributor proof available; trusted public installation blocked**
Owner: Bullet Farm maintainers  
Last reviewed: 2026-08-26

The public discovery index is
[`https://github.com/neverhuman/bulletfarm`](https://github.com/neverhuman/bulletfarm)
(not `neverhuman/bullet-farm`). A hub clone from that URL is not a trusted
install and does not create the other three family members.

This runbook distinguishes three surfaces that must not be conflated.

| Surface | Current authority |
| --- | --- |
| Existing canonical family | contributor development and local proof |
| `scripts/setup.sh` | build-free external `bullet-family` selection; no signed package admission |
| Signed prebuilt installer | required for public release; not published |

## Existing canonical family

The family root contains four ordinary independent clones named by
`repos.manifest.toml`. Never create Git worktrees. Before running family proof:

```bash
cargo run --locked --quiet --bin bullet-family -- doctor --json; echo EXIT=$?
just fast
```

Run these from the Hub checkout. `doctor` reports its verdict in its exit status
as well as its JSON: `0` READY, `3` BLOCKED — the family's "diagnosed, not
usable" code, the same one `check` and `coord` use. Under the checked-in
schema-2 lock the honest answer today is `EXIT=3`, so do not chain `doctor` with
`&&` and do not read exit 0 as proof of a healthy family. `doctor` is a
diagnostic; it does not repair
dirty/missing subjects or grant install authority. The checked-in lock is schema
2, so `checkout verify` currently returns `UNSUPPORTED_SCHEMA` by design. Run it
only after an authenticated schema-3 lock exists, or use that refusal as a
negative diagnostic rather than expecting a clean-family verdict.

For local development fusion, use `bullet-family fuse --source local`. The
ignored `.fusion` output may contain local path overrides; committed manifests
must never contain sibling `path = "../..."` dependencies.

## Source bootstrap wrapper

`scripts/setup.sh` refuses unless `BULLET_SETUP_ADMITTED_BIN` names an external,
canonical regular executable and Cargo/Node/npm are supplied as explicit
absolute paths. Its closed argument surface accepts no wrapper arguments or
exactly one `--offline`; unknown, repeated, or additional arguments refuse
before selecting or executing the external bootstrap. It then invokes:

```text
bullet-family setup --root <family-root> --source jeryu \
  --cargo-bin <absolute-cargo> --node-bin <absolute-node> \
  --npm-cli <absolute-npm-cli> [--offline]
```

It is not a curl-pipe installer or release trust root. The wrapper performs
path and file-shape checks, but does not authenticate its selected executable
as a signed package subject; the Rust boundary separately admits and seals the
dependency tools. The checked-in alpha lock is schema 2, so a hub-only clone
currently returns `UNSUPPORTED_SCHEMA` with regeneration guidance before
creating member directories or running dependency tools. `--offline` narrows
dependency/network behavior; it cannot supply missing signed source authority.

Before the Rust CLI can run, wrapper refusals use `setup: CODE: reason` and exit
4. Automation may branch on `SETUP_ARGUMENT_INVALID`,
`SETUP_BOOTSTRAP_UNAVAILABLE`,
`SETUP_BOOTSTRAP_INVALID`, `SETUP_TOOL_PATH_INVALID`,
`SETUP_HUB_UNAVAILABLE`, or `FAMILY_ROOT_NOT_FOUND`; it must never branch on the
prose reason. These codes do not promote the wrapper into an authenticated
installer. Run `doctor --json` and follow [`setup-recovery.md`](setup-recovery.md)
after recording the exact code and subject.

The source wrapper and its public `just setup` recipe select `/bin/bash`
directly, so a poisoned or absent `PATH` cannot substitute their launcher.
That is a Linux-V1 bootstrap property, not a portable installer claim; the
platform refusal and signed-package requirements below remain authoritative.
Pass the optional wrapper flag through Just as `just -- setup --offline`; the
separator belongs to Just and is not forwarded to the wrapper.

## Future trusted installation

No positive signed/local-Jeryu install command exists in this alpha. Loopback
source-shape admission is not signed installation evidence.

Do not publish a public install command until all inputs exist:

1. a signed prebuilt `bullet-family` for the target platform;
2. an allowed-signers policy and verified non-circular release manifest;
3. a signed schema-3 family lock with authenticated Jeryu URL/slug, tag,
   commit/tree, dependency-lock, generated-artifact, and checksum subjects;
4. the signed package set selected by the requested profile—one Ubuntu 24.04
   x86_64/systemd archive for first-GA `self-hosted-v1`, and five archives only
   for later `universal-v1`—with SBOM, provenance, checksums, and signatures;
5. installer smoke receipts from clean hosts selected by that profile; and
6. a different-UID publication broker plus a minimal pathless root helper that
   installs only exact signed entries into retained root-owned generations and
   reconciles an ambiguous activation as `UNKNOWN`.

The existing Linux `bullet-family release verify` command only verifies an
already materialized ReleaseManifest v2 bundle. It structurally validates and
re-reads the separately signed archive, checksum, CycloneDX, SPDX, and provenance
subjects for all five targets; that frozen universal-envelope verifier is
incompatible with and cannot admit the one-target `self-hosted-v1` package. It
does not interpret subject semantics. Public
`release build` refuses before validation or mutation with
`RELEASE_BUILD_CONTAINMENT_UNAVAILABLE`. Public `release extract` verifies its
input, then refuses before publication with
`RELEASE_PUBLICATION_CONTAINMENT_UNAVAILABLE`. Neither command downloads,
builds, installs, activates, rolls back, or provisions signers.

## Setup transaction rules

The Rust setup mechanism must continue to:

- validate every fallible input before no-replace publication;
- retain the admitted family-root descriptor and recheck pathname identity
  around path-based external commands;
- use descriptor-relative 0700 staging, no-replace member/manifest publication,
  and fsync boundaries;
- create ordinary canonical clones at exact OIDs;
- reject dirty, symlinked, non-empty, or conflicting destinations;
- run `cargo --locked` and `npm ci` through bounded, sealed descriptor subjects
  on Linux;
- generate contracts into temporary directories and reject drift;
- publish the outer completion manifest last; and
- be idempotent: two runs in a fresh home end with exact clean OIDs and no
  tracked changes.

After a crash or refusal, run `doctor --json` before any retry. With an admitted
schema-3 lock, also run `checkout verify`; with the current schema-2 lock, record
its expected `UNSUPPORTED_SCHEMA` refusal instead. Do not delete, overwrite, or
move partially published directories merely to force success. Preserve them for
diagnosis. Recovery has four honest classes: prior state; a recoverable exact
partial with no outer manifest, whose already-published members the same
authenticated setup verifies and reuses without replacement; complete next; and
an indeterminate/orphan state that must remain preserved. A pre-manifest error
does not imply prior state because exact members publish one at a time before the
outer manifest. See [`setup-recovery.md`](setup-recovery.md) for the executable
classification drill.

The Rust setup boundary now snapshots admitted Cargo, Node, Bash, npm, setup
mutation Git, family-lock verification Git, and checkout-verification Git bytes
into sealed read-only memfds on Linux and executes those descriptor subjects; a
swap after verification cannot execute replacement bytes. This does not
authenticate the wrapper-selected external `bullet-family` as a signed release
artifact. Clone-transport Git/helpers, non-Git traversal, and transient or
between-child repository mutation remain outside that boundary.
A trusted public path therefore still starts from a signed prebuilt and must pin
or isolate Git. Bounded cleanup intentionally
preserves an orphan if identity, depth, or entry limits prevent a safe removal.
An error reported after the outer manifest was published is indeterminate until
the same exact setup and verification commands reconcile the durable family.

The positive two-run setup fixture is component proof: it uses `LocalTransport`
and a test-only exact validator. It does not exercise production `JeryuTransport`
or `SetupValidator` end to end, and therefore cannot satisfy installer acceptance.

## Platform boundary

Linux is the initial production runner. Packages for macOS and Windows must
refuse real mutation until equivalent native containment has release evidence.
An archive existing for a platform does not authorize execution there.
