# Release build containment boundary

Status: **public build refused; quarantined component code only; every release gate remains BLOCKED**
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-25

The public `bullet-family release build` command produces no files. It refuses
before argument parsing, source/tool inspection, child execution, or output
creation with `RELEASE_BUILD_CONTAINMENT_UNAVAILABLE`. A release build needs
private exact-OID reconstruction, a sealed toolchain, and a different-identity
build broker; same-UID pathname checks cannot provide that boundary.

The compiler-retained code under `src/release/build/` and its tests preserve a
one-target Linux x86_64 assembly component. It is quarantined implementation
material, not a callable producer, release archive, installer, signer, or
package matrix. First-GA `self-hosted-v1` requires one signed Ubuntu 24.04
x86_64/systemd archive; later `universal-v1` requires all five signed archives.

## Quarantined component shape

The internal component tests exercise this unsigned shape; no public command
currently emits it:

```
<out>/
  family.lock                                   copy of the hub lock, at the path the manifest schema binds
  release-build-manifest.json                   canonical JSON; binds every subject and digest, never its own
  SIGNING.txt                                   the exact operator commands for OD-E
  x86_64-unknown-linux-gnu/
    bullet-farm-<tag>-x86_64-unknown-linux-gnu.tar.zst        the archive
    bullet-farm-<tag>-x86_64-unknown-linux-gnu.cdx.json       CycloneDX 1.6 SBOM
    bullet-farm-<tag>-x86_64-unknown-linux-gnu.intoto.jsonl   unsigned in-toto provenance
    bullet-farm-<tag>-x86_64-unknown-linux-gnu.checksums.json BLAKE3 over every archive entry and bundle file
  .scratch/                                     build scratch: the Portal clone and the cargo/npm caches
```

The archive layout is exactly what `src/release/archive.rs` admits: one
`bullet-farm/` root, byte-sorted unique ASCII paths, regular files and
directories only, parents before children.

```
bullet-farm/
bullet-farm/LICENSE
bullet-farm/bin/{bullet,bullet-effects,bullet-family,bullet-farmd,bullet-gitd,bullet-mcpd,bullet-runner,bullet-verifier}
bullet-farm/share/family.lock
```

The quarantined path builds `bullet-farmd` with `--features embedded-portal`, so
the Portal bytes are inside the daemon binary rather than loose files.
`bullet-mcpd` is the read-only MCP projection adapter. The internal archive
parser requires the exact eight-binary set and tests direct `bin/` entries as
mode 0755 on Unix while retaining package data as 0644. This is component proof
only; the public extractor is disabled even on Linux.

## What it does not produce

- **No public output.** `release build` refuses before it can create `--out`.
- **No signature.** Signing remains operator decision OD-E. Component code may
  render signing instructions, but it owns no signing credential.
- **No profile-preserving `release-manifest.toml`.** ReleaseManifest v2 is the
  frozen universal-envelope component: it requires a schema-3 `family.lock`,
  all five byte-sorted targets, and separately signed archive, checksum,
  CycloneDX, SPDX, and provenance subjects for each target. It cannot represent
  the one-target `self-hosted-v1` package. The checked-in lock is schema 2 and
  four universal target archives are absent.
- **No SPDX producer.** ReleaseManifest v2 can structurally bind an SPDX subject;
  the quarantined one-target builder still emits CycloneDX only. SPDX generation
  and semantic admission remain open builder work.
- **No semantic release admission.** `release verify` checks the v2 structure,
  exact bytes, detached signatures, signer identity, and schema-3 lock binding.
  It does not interpret archive binaries, checksum JSON, either SBOM, or
  provenance semantics.
- **No SHA-256 checksums.** BLAKE3 is the family digest and the only algorithm
  the verifier, the family lock, and the Portal bundle manifest accept. No
  SHA-256 implementation is pinned in `Cargo.lock`; adding one would be an
  unpinned dependency, not evidence.

## Preserved internal assumptions

These are test assertions for the quarantined builder, not public-command
preconditions or release assurances:

1. all four member checkouts are ordinary clean clones;
2. the required Git, Rust, Node, and npm tools are present; and
3. the internal output path is absolute and absent.

They do not solve same-UID tool, repository, or output substitution. The future
broker must re-admit each exact subject inside its different-identity boundary.

## Public refusal

```bash
bullet-family release build \
  --target x86_64-unknown-linux-gnu \
  --out /absolute/absent/bundle
echo EXIT=$?
# RELEASE_BUILD_CONTAINMENT_UNAVAILABLE; nonzero; no output is created
```

Do not treat this refusal as a successful build receipt. It is the admitted
behavior until the build broker exists.

## Quarantined component stages

Tests preserve these intended internal stages without exposing them publicly:

1. admit the target, the toolchain, every member subject, and the lock;
2. clone the committed Portal subject into `<out>/.scratch/bullet-portal`, then
   `npm ci --ignore-scripts`, `npm run build`, `npm run bundle:generate`, and
   `npm run bundle:check` there — never in the tracked checkout and never into a
   tracked `dist`;
3. re-read every emitted Portal file against that bundle manifest before the
   bytes reach the Rust build script;
4. `cargo build --locked --release` for all eight binaries, into a scratch target
   directory so no member checkout is written to;
5. write the deterministic `tar.zst` (uid/gid 0, mtime 0, ustar, one Zstandard
   frame);
6. write the CycloneDX SBOM and exercise its component validator;
7. write the unsigned in-toto provenance statement;
8. write the checksum manifest, then **re-open, re-parse, and re-hash** every
   subject it names;
9. write the non-circular build manifest and re-read it;
10. re-extract the archive through the internal archive component into
    `<out>/.scratch/extracted` and compare every byte and Unix mode to the
    checksum manifest.

## SBOM admission

Every component must carry a name, a version, a package URL, and a license.

- Cargo components come from `cargo metadata --locked` for all three Rust
  workspaces. Each is admitted against the committed `deny.toml` allow-list of
  *every* workspace whose locked graph contains it. SPDX `OR`/`AND`/`WITH`,
  parentheses, and the legacy `A/B` slash form are all evaluated.
- npm components come from the Portal `package-lock.json` bound by the bundle
  manifest. Shipped (non-`dev`) components are admitted against the union of the
  reviewed Rust allow-lists. Build-only components must declare a license but are
  marked `scope: excluded` and are not gated, because the family has no committed
  npm license policy file. **Adding one is an open supply-chain item.**

A component with no license fails the build. It is never a warning.

## Refusals

| Code | Cause |
| --- | --- |
| `RELEASE_BUILD_CONTAINMENT_UNAVAILABLE` | every public `release build` invocation; the different-identity broker does not exist |

The following codes remain internal component/test behavior. The public command
cannot currently reach them:

| Code | Cause |
| --- | --- |
| `UNSUPPORTED_RELEASE_TARGET` | any target other than `x86_64-unknown-linux-gnu`; the message names the other four archives the V1 contract requires |
| `DIRTY_SOURCE` | any member with tracked, untracked, or index changes |
| `RELEASE_TOOLCHAIN_MISSING` | `git`, `cargo`, `rustc`, `node`, or `npm` absent or not executable |
| `RELEASE_OUTPUT_EXISTS` | `--out`, a bundle file, or the Portal scratch clone already exists |
| `RELEASE_PORTAL_BUNDLE_INVALID` | the Portal bundle manifest disagrees with its own files, its lock, or the admitted Git subject; also carries a failed `npm` step's typed output |
| `RELEASE_SBOM_LICENSE_REFUSED` | a component declares no license, or one outside the governing `deny.toml` allow-list |
| `RELEASE_CHECKSUM_MISMATCH` | a byte subject changed between writing and re-reading it, or an extracted entry differs |
| `INVALID_RELEASE_BUILD_MANIFEST` | the build manifest would bind its own digest |
| `RELEASE_BUILD_FAILED` | a locked release compile failed; the child's exact exit status and bounded output are reported |

## Read-only verification and publication refusal

`bullet-family release verify` remains a read-only component for an independently
assembled ReleaseManifest v2 bundle. It verifies exact signed byte subjects but
does not produce, install, activate, or semantically admit them.

After successful verification, public extraction still stops before filesystem
publication:

```bash
bullet-family release extract \
  --bundle /absolute/preassembled-v2-bundle \
  --allowed-signers /absolute/path/to/allowed_signers \
  --target x86_64-unknown-linux-gnu \
  --destination /absolute/absent/destination
# RELEASE_PUBLICATION_CONTAINMENT_UNAVAILABLE; nonzero; destination remains absent
```

Production publication needs a different-UID broker and a minimal pathless root
helper that independently copies exact signed entries into a root-owned
generation, activates only the expected current generation, journals the
outcome, preserves the previous generation, and reconciles an ambiguous switch
as `UNKNOWN`. Internal archive tests are not a substitute.

## Known CLI limit

- There is no `bullet-family --version`; the binary answers `USAGE`.

## Gate status

The refusal and quarantined component tests clear nothing.
`release.package-matrix`, `release.checksums`,
`release.sbom`, `release.manifest-non-circular`, and `release.provenance` remain
`BLOCKED`; no public target was built and the signed five-archive contract is
absent. See [`../release.md`](../release.md).
