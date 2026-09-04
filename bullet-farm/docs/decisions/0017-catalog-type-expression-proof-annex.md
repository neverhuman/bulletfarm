# ADR 0017 normative proof and admission annex

Status: Accepted (DESIGNED; no implementation bytes)
Owner: Bullet Farm maintainers
Applies to: `0017-catalog-type-expression-vocabulary.md`, decision version `Accepted/W11`

## Normative boundary

This file is part of ADR 0017, not commentary. The parent binds its final SHA-256; a missing,
renamed, changed, or differently hashed annex is incomplete. This annex names parent/version but
never its hash, avoiding a cycle; both files must pass review before acceptance.

Everything remains **DESIGNED**: no catalog/generated/source/test/CI byte or evidence fact is created.

## Duplicate-aware public byte admission

One strict Rust/TypeScript algorithm accepts RFC 8259 whitespace/order; caps bytes at 33,554,432 and
depth at 128; requires nonempty strict UTF-8 without BOM, trailing token, duplicate member, unpaired
surrogate, float, exponent, `-0`, or unsafe integer; then applies every parent rule. Integers match
`0|-?[1-9][0-9]*`. Duplicate error names the escaped path; byte/UTF-8/JSON/depth uses `""`;
code is `DOCUMENT_SCHEMA_INVALID` and values are never echoed.

Rust emits, for every strict `T`, exactly these associated function-item signatures (notation, not
literal bodies):

```text
T::decode_bytes: fn(&[u8]) -> Result<T, ContractValidationErrorV1>
T::decode_str: fn(&str) -> Result<T, ContractValidationErrorV1>
```

`decode_str` only calls `decode_bytes(text.as_bytes())`. A private Serde visitor rejects duplicates
before private `UniqueJsonValueV1`; only non-exported `decode_unique_value` reaches collection.
Strict DTOs do not implement `Deserialize`, so `serde_json::from_*::<T>` cannot bypass admission;
legacy derives stay exact. Strict wrappers preserve invariants and serialize infallibly.

For declared `FooV1`, TypeScript exports exactly:

```typescript
export type ContractValidationErrorV1 = Readonly<{
  code: "DOCUMENT_SCHEMA_INVALID"; path: string;
}>;
export type ContractDecodeResultV1<T> =
  | Readonly<{ ok: true; value: T }>
  | Readonly<{ error: ContractValidationErrorV1; ok: false }>;
export function decodeFooV1Bytes(bytes: Uint8Array): ContractDecodeResultV1<FooV1>;
export function decodeFooV1Text(text: string): ContractDecodeResultV1<FooV1>;
```

Text rejects unpaired UTF-16, UTF-8 encodes, then calls bytes; bytes first copies the view. One generated
recursive tokenizer—not `JSON.parse`, casts, or exported object validation—implements duplicate/
numeric/depth rules. Success is a fresh branded readonly graph; every DTO/array/result/error is frozen
without alias. Private validation accepts only branded parser nodes. Hostiles cover nested/discarded
duplicates, formatting, UTF/surrogate/number/depth/path conflicts, mutation, and parser/source bypass.

## W11-S structural predecessor

Baseline `canonical_hostile.rs` is SHA-256 `a4d6eccad4f528810867ecb66c96874594ea516f8991f37db1e879effc74da42`, 2,843 LOC/eight tests. Before its eight-path exception, owner posts physical family-log EOF; paths are root plus `canonical_hostile/{support,metadata,lexical,module_graph,canonical_tests,inventory_tests,hostile_tests}.rs`.

Only whole-item/import moves, Rustfmt, root `include!`, and this split are allowed. Baseline test
`unique_decode_admits_formatting_but_never_ambiguous_members_or_numbers` moves lines 1688..1982
unchanged into expression `metadata.rs` returning `(family,sources,production,test_module_sites)`;
2077..2478 move into unit `module_graph.rs`; inventory retains 1655..1687/1983..2076 via expression includes. No helper/assertion/literal/order/guard/semantic change.

Remaining 2,017 lines have largest item 316 and proven five-bin maximum 411; blocks are 295/402 and retained inventory 127. Every final file is <500. Test list keeps exact eight names/zero benchmarks;
outcomes/guards/maps/counts/hashes match; only W11-ABC-i may then change inventory literals and its owned CI count/hash constants.

## Cap-safe implementation packets

Same-prefix <=4-path lanes may prepare/handoff concurrently but form one compile/test/commit head; each re-pins four sentinels and no lane borrows a path.

1. **W11-ABC-a:** `catalog.rs`, `catalog/validation.rs`, `catalog/validation/tests.rs`.
2. **W11-ABC-b:** `catalog/schema.rs`, `catalog/schema/strict.rs`, `catalog/schema/tests.rs`.
3. **W11-ABC-c:** `contract_bindings.rs`, `contract_bindings/{strict,strict_template,tests}.rs`; `template.rs` unchanged.
4. **W11-ABC-d:** `contract_tool.rs`, `tests/policy_registry.rs`, `tests/policy_v1alpha2.rs`.
5. **W11-ABC-i:** `ops/ci/lib.sh`, `canonical_hostile/inventory_tests.rs`; a/b/c/d/i commit atomically.
6. **W11-P0:** `.github/workflows/ci.yml`, `ci.toml`; preprovision only, with no premature test.
7. **W11-R-a:** `contract_tool.rs`, `contract_tool/w11_platform.rs`, `contract_tool/w11_platform/manifest.rs`.
8. **W11-R-g:** `contract_tool/w11_platform/tests.rs`, `canonical_hostile/inventory_tests.rs`; R-a/g atomic.
9. **W11-D-a:** `contract_tool.rs`, `contract_tool/w11_proof.rs`, `contract_tool/w11_oci.rs`.
10. **W11-D-g:** `contract_tool/w11_proof/tests.rs`, `canonical_hostile/inventory_tests.rs`; D-a/g atomic.
11. **W11-P1:** `scripts/ci-doctor.sh`, `ops/ci/doctor-test.sh`, `.github/workflows/ci.yml`.

Paths default under wire `src/` or `tests/`; `ops/ci/lib.sh` is repository-relative. ABC atomically changes the public schema wrapper to its
fallible production resolver and ABC-d migrates all three callers; no panic, placeholder,
or bypass exists. Current 351-LOC `contract_tool.rs` only declares/wires/calls; R adds platform and D
extends that resolved/fallible path for Rust/TS/profile/OCI. Every file remains <500; ABC also lands
variants/metadata, exhaustive renderers, two legacy literal `None`s, and ABC-i derives/re-pins final
Bullet Wire/total test counts and identity SHA-256 values after every ABC test name is fixed.

## Exact OCI custody table

W11-P0 places one strict top-level `[w11_javascript_oci_v1]` table before the first job with the following keys only. Angle-bracketed values become reviewed committed `ci.toml` non-placeholders, never environment/CLI overrides.

```toml
[w11_javascript_oci_v1]
schema_version = "1"
image_index_digest = "sha256:83f487e0a63425e5b4d146fb5e5be574bcbe1b7b843d3ebafdd95eaf7767a7e5"
image_manifest_ref = "docker.io/library/node@sha256:4d676821dff059fd00d277ee4261ef34ea712317fed0737c03941481b5760c96"
image_config_digest = "sha256:6e6261159fd399ebe5a3d556b7d89da9c85c873f3f270918aad6c8107da8b411"
sealed_projection_root = "<canonical root-owned absolute path>"
key_policy_bundle_sha256 = "<64-lower-hex>"
platform_runtime_manifest_sha256 = "<64-lower-hex>"
platform_admission_envelope_sha256 = "<64-lower-hex>"
platform_admission_issuer = "<1..128 ASCII identity>"
platform_admission_key_id = "<1..128 ASCII key ID>"
platform_admission_public_key_lower_hex = "<64-lower-hex nonzero Ed25519 key>"
platform_admission_key_active_from_unix_ms = 0 # replace with positive safe integer
platform_admission_key_expires_at_unix_ms = 0 # replace; active < expires
platform_admission_key_revoked_at_unix_ms = 0 # replace; active <= revoked <= expires
platform = "linux/amd64"
node_version = "v22.23.2"
npm_version = "10.9.8"
typescript_name = "typescript"
typescript_version = "5.9.3"
typescript_url = "https://registry.npmjs.org/typescript/-/typescript-5.9.3.tgz"
typescript_size = 4377468
typescript_sha512 = "8e5d6f6733c38a72ebf5e52ddc9feded5e8580d130f508ef04f772b33f4a7d00c3e357d0ac2d98e2f290762694a454f86d795bd511e12e9a7cc2d9ba3394e04b"
typescript_sri = "sha512-jl1vZzPDinLr9eUt3J/t7V6FgNEw9QjvBPdysz9KfQDD41fQrC2Y4vKQdiaUpFT4bXlb1RHhLpp8wtm6M5TgSw=="
child_environment_sha256 = "11dfff13666b598d1717379844439d3668f68c20c197f5bac8292ac66d201468"
runtime_profile_envelope_sha256 = "<64-lower-hex>"
runtime_profile_issuer = "<1..128 ASCII identity>"
runtime_profile_key_id = "<1..128 ASCII key ID>"
runtime_profile_public_key_lower_hex = "<64-lower-hex nonzero Ed25519 key>"
runtime_profile_key_active_from_unix_ms = 0 # replace with positive safe integer
runtime_profile_key_expires_at_unix_ms = 0 # replace; active < expires
runtime_profile_key_revoked_at_unix_ms = 0 # replace; active <= revoked <= expires; no revocation means expires
memory_bytes = 1073741824
memory_swap_bytes = 1073741824
pids = 64
cpus_millis = 1000
work_tmpfs_bytes = 268435456
child_output_bytes = 1048576
inspect_deadline_ms = 10000
execute_deadline_ms = 120000
cleanup_deadline_ms = 10000
```

Literal angle brackets or zero lifecycle placeholders are `W11_OCI_CONFIG_INVALID`. Both lifecycle
triples are JSON-safe positive integers no greater than 9,007,199,254,740,991 and obey the shown
inequalities; each public key is exactly 32 bytes with a nonzero byte, and the two keys differ. Test
keys exist only in temporary fixtures. The committed platform key/lifecycle/envelope digest is the
root for the separately signed predecessor below; changing any trust value requires source review.
W11 cannot mint, rotate, self-sign, default, download, or accept a caller key.

The sole environment names six canonical files under the normalized table root:
`BULLET_W11_OCI_RUNTIME=/static/bin/oci-client`, `BULLET_W11_TYPESCRIPT_TGZ=/dynamic/typescript.tgz`, `BULLET_W11_OCI_RUNTIME_PROFILE=/static/trust/runtime-profile.json`,
`BULLET_W11_KEY_POLICY_BUNDLE=/static/trust/key-policy-bundle.json`, `BULLET_W11_PLATFORM_ADMISSION=/static/trust/platform-admission.json`, and `BULLET_W11_PLATFORM_MANIFEST=/static/trust/platform-manifest.json`.
It chooses no root/trust/purpose/endpoint/image/toolchain/outcome; missing, duplicate, relative, linked, mutable, noncanonical, or mismatched input refuses.

## Signed platform-runtime admission predecessor

`W11PlatformRuntimeAdmissionEnvelopeV1` is distinct 1..=65,536-byte canonical JSON with exact logical fields `{schema_version,issuer,key_id,paseto}` and RFC 8785 order. Version is `"v1alpha1"`; raw digest, issuer, key, and key window equal the table. PASETO v4.public has fixed footer
`bullet-farm.w11-platform-runtime-admission-footer.v1alpha1`, assertion
`bullet-farm.w11-platform-runtime-admission.v1alpha1`, purpose `w11-platform-runtime-admission-signing`,
and domain `w11.platform-runtime-admission-claims.v1alpha1`. Crate-private `PurposeSeparatedPasetoVerificationKey` accepts only the table key; no generic/public verifier, alternate key source, or caller purpose exists. Claims are exact `{schema_version,signing_purpose,
claims_domain,admission_id,issuer,key_id,signed_at_unix_ms,valid_from_unix_ms,expires_at_unix_ms,
manifest_sha256}`. The <=1,048,576-byte canonical raw manifest hashes to claim/table and is exactly:

```text
W11PlatformRuntimeManifestV1 = {
 schema_version:"v1alpha1", admission_id:WraId, sealed_root:AbsPath,
 source_subjects:SourceSubjects, keys:KeyRole[5..260], carriers:CarrierPath[4],
 ancestors:PathIdentity[1..512],
 files:RuntimeFile[16..4096], elf_roots:ElfRoot[10..64], processes:ProcessClosure[4..16],
 kernel:KernelClosure, socket:SocketClosure, projection:ProjectionPolicy,
 rust_boundary:RustBoundaryPolicy, toolchain:RustToolchainSubject
}
SourceSubjects = {key_policy_bundle_sha256:Sha256}
KeyRole = {purpose:Identity,role:KeyRoleName,issuer:Identity,key_id:Identity,public_key_lower_hex:Sha256,
 custodian_principal:Identity,custodian_service_uid:SafeU64,custody_subject_sha256:Sha256,
 active_unix_ms:SafeU64,expires_unix_ms:SafeU64,revoked_unix_ms:SafeU64}
KeyRoleName = "policy_issuer"|"release_signer"|"platform_attestor"|"runtime_profiler"|"projection_broker"|"rust_proof_broker"
PathIdentity = {path:AbsPath,dev:SafeU64,ino:SafeU64,mount_id:SafeU64,uid:SafeU64,
 gid:SafeU64,mode:SafeU64,nlink:SafeU64}
CarrierPath = {role:"key_policy_bundle"|"platform_admission"|"platform_manifest"|"runtime_profile",identity:PathIdentity}
RuntimeFile = {role:RuntimeRole,identity:PathIdentity,size:SafeU64,sha256:Sha256,
 fs_verity_sha256:Sha256}
ElfRoot = {path:AbsPath,interpreter:AbsPath|null,needed:AbsPath[]}
ProcessClosure = {role:ProcessRole,pid:SafeU64,start_ticks:SafeU64,uid:SafeU64,gid:SafeU64,
 exe:PathIdentity,cmdline_sha256:Sha256,environ_sha256:Sha256,config_paths:AbsPath[]}
ProcessRole = "daemon"|"containerd"|"projection_broker"|"rust_proof_broker"
KernelClosure = {boot_id:Identity,release:Ascii256,build_id:Identity,notes_sha256:Sha256,
 image_path:AbsPath,config_path:AbsPath,module_paths:AbsPath[],measured_boot_sha256:Sha256}
SocketClosure = {identity:PathIdentity,peer_role:"daemon",peer_pid:SafeU64,peer_start_ticks:SafeU64,
 peer_uid:SafeU64,peer_gid:SafeU64,peer_exe:PathIdentity,daemon_id:Ascii128}
ProjectionPolicy = {static_root:AbsPath,dynamic_root:AbsPath,broker_socket:PathIdentity,
 broker_process_role:"projection_broker",
 signer_issuer:Identity,signer_key_id:Identity,signer_public_key_lower_hex:Sha256,
 max_receipt_ms:120000}
RustBoundaryPolicy = {broker_socket:PathIdentity,broker_process_role:"rust_proof_broker",
 profile_path:AbsPath,profile_sha256:Sha256,signer_issuer:Identity,signer_key_id:Identity,
 signer_public_key_lower_hex:Sha256,network:"none",max_processes:64,deadline_ms:120000}
RustToolchainSubject = {cargo:PathIdentity,rustc:PathIdentity,linker:PathIdentity,
 cargo_sha256:Sha256,rustc_sha256:Sha256,linker_sha256:Sha256,
 cargo_version:"1.95.0",cargo_commit:Ascii64,rustc_version:"1.95.0",rustc_commit:Ascii64,
 cargo_home:PathIdentity,cargo_home_tree_sha256:Sha256,target:"x86_64-unknown-linux-gnu"}
```

The source-pinned bundle is <=1,048,576-byte canonical JSON with exact recursively closed record
`{schema_version:"v1alpha1",policy_snapshot:PolicySnapshotV1,release_signer_policy:
ReleaseSignerPolicyV1,accepted_keys:KeyRole[1..256],test_key_sha256:Sha256[1..256]}`; raw SHA-256 equals
table/manifest, and both embedded policies use their current generated schemas and validators.
All records reject unknowns; nullable fields are required. Source `revoked_at:null` normalizes to
`revoked_unix_ms=expires`; `effective_end=min(expires,revoked_unix_ms)`, and every KeyRole obeys
`0 < active < expires` plus `active <= revoked <= expires`. `AbsPath` is normalized absolute UTF-8,
1..4096 bytes, without control, `.`/`..`, symlink, magic-link, or empty component. Arrays are raw-key
sorted/unique. `RuntimeRole` is exactly `oci_client|cargo|rustc|linker|loader|shared_object|daemon|
containerd|shim|runc|runtime_config|seccomp|kernel_image|kernel_config|kernel_module|projection_broker|
rust_proof_broker|rust_boundary_profile|cargo_cache`. `WraId` is `wra_` plus the existing framed-BLAKE3 hash of JCS(manifest without
`admission_id`) under `w11.platform-runtime-admission-id.v1alpha1`.

The client is `linux/amd64` static ELF; each other ELF root's interpreter and recursive `DT_NEEDED`
closure appears once in `files`. Mandatory roles are exactly `oci_client,cargo,rustc,linker,daemon,
containerd,shim,runc,runtime_config,seccomp,kernel_image,kernel_config,projection_broker,
rust_proof_broker,rust_boundary_profile,cargo_cache`;
`loader/shared_object` exist exactly when referenced and `kernel_module` equals the loaded set.
`processes` has exactly persistent daemon/containerd/projection-broker/rust-proof-broker facts; during use, every shim/runc descendant
and cgroup member must execute a signed sealed identity. The attestor binds their configuration,
kernel build/notes/measured boot, socket peer, and tool bytes/commits. W11 parses ELF and re-hashes/
re-stats every file, `/proc` fact, kernel fact, socket peer/executable, daemon ID, and containment
before create, during shim use, and after cleanup; missing/change is `W11_OCI_RUNTIME_UNADMITTED`.

Normalize every PolicySnapshot issuer key with role `policy_issuer` and ReleaseSignerPolicy key with
role `release_signer` to `(purpose,role,issuer,key_id,decoded-32-byte-Ed25519,lifecycle)`; undecodable,
non-Ed25519, duplicate `(purpose,issuer,key_id)`, or reused material refuses. `accepted_keys` is the
one-to-one source union plus only custody fields, lexicographically sorted by raw `(purpose,issuer,
key_id)` bytes; source scans refuse another key loader. `keys` is exactly it union the four distinct
platform/profile/projection/rust-execution rows, same sort, and disjoint from raw-material SHA-256
`test_key_sha256`, the source-reviewed complete raw-key digest set for all test-only signing fixtures;
the verifier computes the union and no runtime caller supplies either set. Any bundle, union,
denylist, or custody mismatch is `W11_OCI_RUNTIME_UNADMITTED`.
Exactly one entry has each purpose `w11-platform-runtime-admission-signing`,
`w11-oci-runtime-profile-signing`, `w11-sealed-projection-signing`, and
`w11-rust-proof-execution-signing`; their roles are respectively `platform_attestor`,
`runtime_profiler`, `projection_broker`, and `rust_proof_broker`. Each equals its table/manifest
carrier and binds a distinct principal, service UID, custody subject, lifecycle, and revocation;
their principals/services/material differ from each other and every accepted policy key. Thus admission,
not issuer prose, precedes profile and projection verification.
For each of those four purposes, the exact relation is `active <= signed_or_issued_at <= verifier
OS-now < claim_expires_at <= min(effective_end, signed_or_issued_at + purpose_cap)`, where the cap is
2,678,400,000 ms for platform/profile and 120,000 ms for projection/Rust. Before activation and at
effective end refuse; exact activation admits only when every other condition matches.

Each carrier is its named environment leaf; its identity/ancestors are signed and table digests bind
its bytes without a cycle. Every RuntimeFile and its root-to-leaf ancestors equal signed device/inode/mount/owner/mode/nlink
facts, is root-owned, group/world-nonwritable, single-link, fs-verity sealed, and lies on the signed
read-only sealed mount. W11 holds opened identities through use. Dynamic inputs use the signed
projection broker over its root-owned socket: request is one u32be-bounded canonical header
`{admission_id,nonce,files:[{name,size,sha256}]}` followed by raw-name-sorted u64be-bounded bytes;
`request_sha256=SHA-256(u32be(JCS-header length)||JCS(header)||bytes)`. Nonce is one-use 64-lower-hex.
Response is a <=32,768-byte canonical `{schema_version:"v1alpha1",issuer,key_id,paseto}` envelope using footer
`bullet-farm.w11-sealed-projection-footer.v1alpha1`, assertion
`bullet-farm.w11-sealed-projection.v1alpha1`, purpose `w11-sealed-projection-signing`, and domain
`w11.sealed-projection-claims.v1alpha1`. Exact claims are `{schema_version,signing_purpose,
claims_domain,admission_id,nonce,request_sha256,root,ancestors,files,issued_at_unix_ms,
expires_at_unix_ms}`; file rows are `{name,identity,size,sha256,fs_verity_sha256}`. The manifest key
verifies it; IDs, request, sorted identities, and the uniform lifecycle equation must match. Broker
holds the opened read-only lease until removal; W11 holds descriptors. Static/dynamic roots bind
bundle/profile/admission/manifest/tools and four inputs. Before/during/after each projection or Rust
broker exchange, `SO_PEERCRED` PID/UID/GID equals its referenced ProcessClosure; `/proc` start/exe,
signed RuntimeFile identity/bytes, socket ancestors/identity, and KeyRole custodian UID also equal.
Drift is `W11_OCI_RUNTIME_UNADMITTED` or, for Rust, `W11_RUST_BOUNDARY_UNADMITTED`.

## Signed runtime-profile carrier

The profile is 1..=32,768 duplicate-free canonical JSON with exact logical declaration fields `{schema_version,issuer,key_id,paseto}` and RFC 8785 wire order. Values are `"v1alpha1"`, table issuer/key ID, and a cap-bounded `v4.public.*`; raw SHA-256 equals table. PASETO uses crate-private `PurposeSeparatedPasetoVerificationKey` with only its nonzero 32-byte Ed25519 key and fixed footer
`bullet-farm.w11-oci-runtime-profile-footer.v1alpha1`, and fixed implicit assertion bytes
`bullet-farm.w11-oci-runtime-profile.v1alpha1`. No generic/public verifier or alternate purpose is
generated.

Authenticated payload uses the following exact logical declaration order, RFC 8785 wire-key order,
and recursive unknown-field rejection:

```text
W11OciRuntimeProfileClaimsV1 = {
  schema_version:"v1alpha1", signing_purpose:"w11-oci-runtime-profile-signing",
  claims_domain:"w11.oci-runtime-profile-claims.v1alpha1", profile_id:WrpId,
  issuer:Identity, key_id:Identity, signed_at_unix_ms:SafeU64,
  valid_from_unix_ms:SafeU64, expires_at_unix_ms:SafeU64, subject:W11OciRuntimeSubjectV1
}
W11OciRuntimeSubjectV1 = {
  platform_admission_id:WraId, platform_manifest_sha256:Sha256,
  client_sha256:Sha256, client_version:Ascii64, client_api_version:ApiVersion,
  daemon_endpoint:"unix:///var/run/docker.sock", daemon_id:Ascii128,
  server_version:Ascii64, server_api_version:ApiVersion, engine_commit:Ascii128,
  containerd_version:Ascii64, containerd_commit:Ascii128,
  runc_version:Ascii64, runc_commit:Ascii128, os:"linux", architecture:"amd64",
  kernel_version:Ascii256, cgroup_version:"2", cgroup_driver:Ascii64,
  security_options:Ascii128[1..16], seccomp_profile:"builtin",
  no_new_privileges_supported:true, image_manifest_digest:Sha256Tag,
  image_config_digest:Sha256Tag
}
```

`SafeU64` is a JSON integer in `0..=9007199254740991`; `Sha256` is 64 lower hex; `Sha256Tag` is
`sha256:` plus it; `ApiVersion` is `^[0-9]+\.[0-9]+$`; `AsciiN` is 1..N bytes, each in
`0x21..=0x7e`; `Identity` is 1..128 bytes in `[A-Za-z0-9._:/-]`, exactly matching the reused key
carrier. Security options are raw-byte sorted/unique. `WrpId` is
`wrp_` plus lower hex of existing
`hash_framed_bytes("w11.oci-runtime-profile-id.v1alpha1", JCS(subject))` (the existing
`bullet-wire.v1\0`, u64-little-endian length framing). Recompute before use.

Authentication precedes subject semantics. First verify the source-pinned bundle bytes, policies,
complete union, and denylist; authenticate the platform envelope with the table key; verify manifest/
ID/live closure and bundle digest; then select the distinct profile key from its signed complete
index and authenticate the profile. Envelope/claim/table identities, keys, and
admission/manifest subjects must agree. Platform/profile additionally require
`valid_from <= signed_at < expires`, OS-now in `[valid_from,expires)`, and the uniform key/lifetime
equation above; time comes only from the verifier. Clock error, inactive/revoked key, wrong
purpose/domain/footer/assertion/key/digest/ID, noncanonical or duplicate bytes, and any semantic or
live read-back mismatch are `W11_OCI_RUNTIME_UNADMITTED`. Cross-purpose W8/W9/W10 tokens refuse.
OS time is component freshness only; release still requires signed trusted time and platform receipt.

## OCI execution and read-back

Every call executes the manifest's direct fs-verity-sealed static client (never a shim) with only
`HOME=/`, `LC_ALL=C`, empty `PATH`, and `DOCKER_CONFIG=/dev/null`, and uses
`--host=unix:///var/run/docker.sock`. Before create, exact bounded `version --format
{{json .}}`, `info --format {{json .}}`, and `image inspect <manifest-ref>` observations must match
every authenticated subject and table field. The projection receipt/held lease binds all mounted
inputs. Container name is `bullet-w11-` plus the first 24 lower hex of SHA-256 over JCS array
`[profile_id,projection_receipt_sha256,tgz_sha256,generated_ts_sha256,hostile_ts_sha256,
env_guard_sha256]`. Requested labels are exactly
`bullet.w11.profile-id`, `bullet.w11.typescript-sha256`, `bullet.w11.generated-sha256`, and
`bullet.w11.hostile-sha256`, plus `bullet.w11.env-guard-sha256` and
`bullet.w11.projection-receipt-sha256`, with corresponding full values; image-config labels remain
exact and no other caller label is allowed. An existing name is inspected/reconciled and removed or
fails—never renamed/retried blind.

`container create` uses the manifest ref, exact labels/name, one read-only `rprivate` `/inputs`
bind, `/work:rw,nosuid,nodev,noexec,size=268435456,mode=0700,uid=65532,gid=65532` tmpfs, and exactly:

```text
--pull=never --platform=linux/amd64 --network=none --read-only --cap-drop=ALL
--security-opt=no-new-privileges=true --security-opt=seccomp=builtin --user=65532:65532
--pids-limit=64 --cpus=1 --memory=1073741824 --memory-swap=1073741824
--ulimit=nofile=256:256 --cgroupns=private --ipc=private --shm-size=16777216
--restart=no --stop-timeout=1 --entrypoint=/bin/sh
```

Pre-start `container inspect` requires every HostConfig/Config/Mount value, sealed dynamic source,
labels/name,
`Privileged=false`, no added capability/device/port/DNS/extra-host/host namespace, exact config ID,
entrypoint, and environment. Only image-bound `PATH/NODE_VERSION/YARN_VERSION` plus fixed
`HOME=/work/home`, `LANG/LC_ALL=C.UTF-8`, `TZ=UTC`, empty `NODE_OPTIONS/NODE_PATH`,
`NODE_DISABLE_COMPILE_CACHE=1`, `npm_config_cache=/work/npm-cache`,
`npm_config_offline=true`, `npm_config_userconfig=/dev/null`,
`npm_config_globalconfig=/dev/null`, `npm_config_update_notifier=false`,
`npm_config_audit=false`, and `npm_config_fund=false` are present; no other variable is present.

`container start --attach` runs fixed `/bin/sh -ceu`. Every npm/Node child actually starts through
`/usr/bin/env -i` with exactly the 15 pairs whose JCS
object is 385 bytes and SHA-256 equals table `child_environment_sha256`: `HOME=/work/home`,
`LANG=C.UTF-8`, `LC_ALL=C.UTF-8`, `PATH=/nonexistent`, `TZ=UTC`, empty `NODE_OPTIONS/NODE_PATH`,
`NODE_DISABLE_COMPILE_CACHE=1`, and the seven npm variables above. Pinned `env-guard.js` compares
JCS(`process.env`) before loading its absolute target and returns the hash in the final result;
Docker/shell-only `HOSTNAME/PWD/SHLVL` cannot enter a child. Exact install argv is
`<ENV> /usr/local/bin/node /inputs/env-guard.js
/usr/local/lib/node_modules/npm/bin/npm-cli.js install /inputs/typescript.tgz --offline
--ignore-scripts --no-audit --no-fund --package-lock=false
--workspaces=false --no-bin-links --install-links=false --loglevel=error --prefix=/work/ts`, then
the exact guarded TypeScript commands below. Post-exit inspect requires exit 0, not OOM-killed, empty engine
error, unchanged profile/client/input/image/config, and matching containment. Every child has
bounded frames and a monotonic deadline: inspect/kill/remove 10,000 ms, attach 120,000 ms, each
stream 1,048,576 bytes.
Cleanup always kills then force-removes the exact name and kills the attached client; stopped/not-
found is success only after inspect proves the exact completed/absent state. Response loss stays
unknown until name/labels/state reconcile. No pull, registry, PATH/npx/cache/vendor fallback exists.

Stable failures are `W11_OCI_CONFIG_INVALID`, `W11_OCI_RUNTIME_UNADMITTED`,
`W11_OCI_IMAGE_MISSING`, `W11_OCI_IMAGE_MISMATCH`, `W11_TYPESCRIPT_ARTIFACT_INVALID`,
`W11_OCI_CONTAINMENT_INVALID`, `W11_OCI_EXECUTION_TIMEOUT`, `W11_OCI_OUTPUT_LIMIT`,
`W11_OCI_EXECUTION_FAILED`, and `W11_TYPESCRIPT_TREE_MISMATCH`; none skips or passes.

## Standalone Rust and TypeScript proof subjects

The temporary Rust manifest is exactly the following LF-final 422 bytes, SHA-256
`f8d64a474a4e13e63400cab09037966b93ddcfc3c7ca7548cdc36672158ed733`:

```toml
[package]
name = "bullet-w11-generated-proof"
version = "0.0.0"
edition = "2024"
rust-version = "1.95"
publish = false

[dependencies]
blake3 = { version = "=1.8.7", default-features = false, features = ["pure", "std"] }
serde = { version = "=1.0.229", features = ["derive"] }
serde_jcs = "=0.2.0"
serde_json = "=1.0.151"
unicode-normalization = "=0.1.25"

[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
all = "deny"
```

W11-D holds exact root `Cargo.lock` SHA-256 `e26db8c20b9b24f17193ad9b9adcbabdbb082a3ae655a5014a6f6497a3363fd9`, manifest, generated bytes plus `\n#[cfg(test)] mod hostile;\n` as `src/lib.rs`, and complete literal `src/hostile.rs` with colocated SHA tests.
The broker receives sealed Cargo/rustc/linker/cargo-cache descriptors, then held root-owned,
group/world-nonwritable, single-link, fs-verity regular-file FDs for exact `Cargo.toml,Cargo.lock,
src/lib.rs,src/hostile.rs` in raw-name order via `SCM_RIGHTS`; no caller directory/other descriptor survives.
`SourceFileRow={path:"Cargo.toml"|"Cargo.lock"|"src/lib.rs"|"src/hostile.rs",mode:SafeU64,size:SafeU64,sha256:Sha256}`; request field
`source_manifest_sha256=SHA-256(JCS(raw-path-sorted rows))`. The <=32,768-byte u32be-framed request is
`{admission_id,profile_id,projection_receipt_sha256,source_manifest_sha256,cwd:"/work/project",environment,argv}`. Its strict profile is exact
`{schema_version:"v1alpha1",rootfs:PathIdentity,rootfs_tree_sha256:Sha256,network:"none",namespaces:["user","mount","pid","network","ipc"],
empty_mounts:["/run","/var/run","/tmp"],proc:"private",devices:["null","zero","random","urandom"],read_only_tools:["/tools/cargo","/tools/rustc","/tools/linker"],
writable_root:"/work",network_syscalls:"deny_all",max_processes:64,deadline_ms:120000}`;
raw digest equals signed policy/RuntimeFile. Broker creates those private namespaces/cgroup, read-only admitted root/private mounts, and live-matching mounts; loopback is down with no address/route.
Seccomp-notify refuses/counts `socket,socketpair,connect,bind,listen,accept,accept4,sendto,sendmsg,sendmmsg,recvfrom,recvmsg,recvmmsg,shutdown`; attempt/residual process fails and deadline kills the cgroup.
W11 live-checks peer/start, profile/rootfs/tree, namespaces, cgroup, mounts, seccomp, network, process tree, and cleanup against signed facts.
Rootfs contains only directories and manifest RuntimeFiles; links/specials refuse. Its digest is SHA-256(JCS(raw-path-sorted exact `{path:AbsPath,kind:"dir"|"file",mode:SafeU64,size:SafeU64,sha256:null|Sha256}` rows)); directories require size zero/null hash; private proc/dev mounts are excluded and live-checked.
Broker privately copies only those FDs, verifies/re-hashes them throughout, and runs lock generation
as the sole writable-source phase. After exact lock verification it makes all four files and their
mount read-only; same-shape raw-sorted test-pre/test-post manifests bind the generated lock, match,
and are re-hashed after test, so no caller/test process can replace a path.

Inside, `<R>=/work/project`, `<T>=/work/target`, `<CARGO>=/tools/cargo`, `<RUSTC>=/tools/rustc`, and `<LINKER>=/tools/linker`; CWD is exactly `<R>`.
Fresh mode-0700 `/work/{project,home,target,tmp}` and private `/work/cargo-home` match receipt/tree digests before/after. Held parent descriptors prove absence before exec and after exit of exactly
`/work/project/.cargo/config`, `/work/project/.cargo/config.toml`, `/work/.cargo/config`, `/work/.cargo/config.toml`, `/.cargo/config`, `/.cargo/config.toml`,
`/work/cargo-home/config`, and `/work/cargo-home/config.toml`. Root is read-only, request has no `--config`, and exact environment is only `HOME=/work/home`, `TMPDIR=/work/tmp`, `PATH=`,
`CARGO_HOME=/work/cargo-home`, `CARGO_TARGET_DIR=<T>`, `RUSTC=<RUSTC>`, `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=<LINKER>`, `LC_ALL=C`, `TZ=UTC`,
`CARGO_NET_OFFLINE=true`, `CARGO_TERM_COLOR=never`, and `RUSTUP_AUTO_INSTALL=0`; direct argv is:

```text
<CARGO> generate-lockfile --quiet --offline --manifest-path <R>/Cargo.toml
<CARGO> test --quiet --locked --offline --manifest-path <R>/Cargo.toml --target x86_64-unknown-linux-gnu --target-dir <T> --lib -- --test-threads=1
```

The generated lock must be 5,733 bytes, 219 LOC, 26 package tables including the root, no path/git source, and SHA-256 `ed3d2155178bcdfc4acb3859bee841a38add3b11e5b9d2c8cba870fde7bd08db`.
Absent-path drift or config `[env]`, alias, wrapper, rustflags, runner, source replacement, or credential is `W11_RUST_BOUNDARY_UNADMITTED`. Broker returns 1..=32,768 canonical bytes in exact
envelope `{schema_version:"v1alpha1",issuer,key_id,paseto}` using purpose `w11-rust-proof-execution-signing`, footer `bullet-farm.w11-rust-proof-execution-footer.v1alpha1`,
assertion `bullet-farm.w11-rust-proof-execution.v1alpha1`, and domain `w11.rust-proof-execution-claims.v1alpha1`. Exact claims are `{schema_version,signing_purpose,
claims_domain,request_sha256:Sha256,descriptor_identity_sha256:Sha256,source_manifest_sha256:Sha256,
test_pre_source_manifest_sha256:Sha256,test_post_source_manifest_sha256:Sha256,broker_pid:SafeU64,broker_start_ticks:SafeU64,user_ns:SafeU64,mount_ns:SafeU64,
pid_ns:SafeU64,net_ns:SafeU64,ipc_ns:SafeU64,cgroup_sha256:Sha256,profile_sha256:Sha256,config_absence_sha256:Sha256,attempted_network_syscalls:0,
stdout_sha256:Sha256,stderr_sha256:Sha256,exit_code:0,cleanup_complete:true,issued_at_unix_ms:SafeU64,expires_at_unix_ms:SafeU64}`. Request hash is `SHA-256(u32be(JCS length)||JCS(request))`;
descriptor/absence hashes cover exact ordered identities/eight paths; input hash equals request rows,
test-pre/post hashes equal their private rows and each other. Issuer/key and the uniform lifecycle equation match, and each stream is <=`child_output_bytes`.
Wrong envelope/purpose/key/field/hash/time or incomplete cleanup is `W11_RUST_BOUNDARY_UNADMITTED`. Missing cache/tool/target, warning, ignored/zero/skipped test, changed source/lock, network attempt,
nonempty stderr, or nonzero exit is `W11_RUST_EXECUTION_FAILED`. The platform manifest binds Cargo 1.95.0 commit
`f2d3ce0bd7f24a49f8f72d9000448f8838c4e850` and rustc 1.95.0 commit
`59807616e1fa2540724bfbac14d7976d7e4a3860c`; substitution refuses.

OCI `/inputs` contains only exact `typescript.tgz`, `generated.ts`, `hostile.ts`, and pinned
`env-guard.js`, all named in the projection receipt. npm installs
twice cleanly. In-image read-back rejects link/special/non-beneath/non-UTF-8 entries and computes
`SHA-256(bytes("bullet.w11.typescript-tree.v1")||0x00||CONCAT_RAW_SORTED(u32be(path_bytes.len)||path_bytes||u64be(size)||raw_file_sha256))`.
Both installs contain exactly 132 regular files and root
`97a67fbd0bfcac69ebce426aab78259fe0dcaad818f0e3b9cd75ba610a2480c2`.

Exact commands are:

```text
<ENV> /usr/local/bin/node /inputs/env-guard.js /work/ts/node_modules/typescript/lib/tsc.js --pretty false --strict --noEmit --target ES2022 --module commonjs --moduleResolution node --lib ES2022 --skipLibCheck false /inputs/generated.ts /inputs/hostile.ts
<ENV> /usr/local/bin/node /inputs/env-guard.js /work/ts/node_modules/typescript/lib/tsc.js --pretty false --strict --target ES2022 --module commonjs --moduleResolution node --lib ES2022 --skipLibCheck false --outDir /work/out /inputs/generated.ts /inputs/hostile.ts
<ENV> /usr/local/bin/node /inputs/env-guard.js /work/out/hostile.js
```

`<ENV>` is literal `/usr/bin/env -i` plus the exact pairs above. `hostile.ts` and `env-guard.js` are
complete W11-D source literals with pinned SHAs. The final program emits exactly one JCS line
`{child_environment_sha256:<exact>,hostile_cases:<positive pinned count>,schema_version:"v1alpha1",status:"PASS"}` and
nothing on stderr; the harness derives no outcome from caller text. Tree/input/image/profile,
admission/projection/tool hashes and held identities are re-read after emit and execution. Two
independent installs/runs must have identical count/root.

## Hostile and proof closure

The table-driven synthetic catalog covers every parent spelling/scalar/shape/union and unused
declaration. Required failures include every stable class and pairwise/same-class precedence;
unknown meta fields/classes/versions; name/reserved/transform/cross-kind collisions; every absent,
extra, zero, reversed, maximum, and over-maximum bound/target; integer/text/code/enum/ID boundaries;
nullable omission/null; array/set/key order oppositions; union discriminator/tag/branch closure;
self/two-node/mixed record-union/legacy-new cycles; production coverage refusal of the private
fixture; and source scans proving exactly one resolver and no public/environment/CLI/test bypass.

Proof runs the complete `bullet-wire` unit/integration suite, `policy_registry`,
`release_registry_contract`, locked contract check, library Clippy `-D warnings`, Rustfmt/scope
diff, committed generated zero-drift, all four parent sentinels, every split/inventory/LOC guard,
and post-D W11-P1 doctor hostiles for key-policy bundle/union/denylist and verifier OS-now just
before/at key activation, key revocation, key expiry, claim issue, and claim expiry for every
signer purpose. Just before issue refuses, at issue admits when all else matches, just before claim
expiry admits, and at claim expiry refuses. Proof also covers
complete ELF/runtime/kernel/socket/path read-back, Rust profile/rootfs/config/network/tools/cache,
child environment, image mismatch, containment, response loss, cleanup, output, and deadline failure. Any missing tool/case, zero test,
skip, flaky/unknown result, warning, undeclared path, file at or above 500 LOC, or changed catalog,
generated, LC, policy, release, constraint, sentinel, or template byte is a hard failure.

Maximum W11 evidence remains **COMPONENT_ONLY** after separate implementation and review. Platform
runtime admission proves component tool custody, not release containment; release needs independently
admitted platform containment, trusted time, signer lifecycle/revocation, registry, and semantic
GateReceipt. Acceptance leaves the pair **DESIGNED**; it remains so until every packet lands.
