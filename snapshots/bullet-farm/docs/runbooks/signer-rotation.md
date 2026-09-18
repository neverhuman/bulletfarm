# Signer rotation — launch-grant key today, release-signing key not provisioned

Status: **launch-grant key lifecycle available offline; release signing not provisioned; no rotation receipt exists**  
Owner: Bullet Farm maintainers  
Last reviewed: 2026-08-25  
Component receipt baseline (minimum; replay current-head lanes before use): bullet-kernel `d388733`+ (`bullet authority keygen`, launch-grant issuer; read at `0109a90`),
bullet-farm `bf5c642`+ (policy v1alpha2 rule) and `143f8b9`+ (`release receipt-verify`), ADRs 0005, 0010,
0011, 0012

Two signers exist in the design and only one exists as bytes. This runbook is split accordingly. Neither
half has been exercised as a rotation on this host; the commands and semantics below were verified against
source, not against a rotation receipt.

## 1. Launch-grant authority key (exists)

### 1.1 What `keygen` does

```bash
cd ../bullet-kernel
cargo run --locked -p bullet -- authority keygen --data-dir /abs/bullet-data \
  [--issuer bullet-kernel] [--key-id launch-grant-alpha]
```

`apps/bullet/src/authority.rs` + `crates/harness-core/src/launch_grant/keyfile.rs`:

- writes `<data-dir>/authority/launch-grant.key` (directory forced to `0700`, file created with
  `create_new` and mode `0600`, 64 raw bytes); **never overwrites** — an existing file, a relative
  `--data-dir`, or a non-Unix host is `LAUNCH_GRANT_INVALID`;
- prints `key_file`, `public_key_hex`, and a ready-to-paste `IssuerKeyV1` with
  `key_purpose = authority-signing`, `algorithm = paseto-v4.public`, `audiences = ["provider-runner"]`,
  `activates_at_unix_ms = now`, `expires_at_unix_ms = now + KEY_VALIDITY_MS` (365 days),
  `revoked_at_unix_ms = null`, `retain_until_unix_ms = expires + KEY_RETENTION_GRACE_MS` (24 h);
- prints on stderr that the key must be ratified into a new policy generation and that v1alpha1 keeps live
  admission disabled regardless.

`load_signing_key` (used by `mint-launch-grant` and the live-conformance path) refuses a symlink, a mode other
than exactly `0600`, a file not owned by the current user, or a length other than 64 bytes, and
`mint-launch-grant` refuses `LAUNCH_GRANT_KEY_UNKNOWN` when the file's public key differs from the
policy-admitted key for the same `issuer`/`key_id`.

### 1.2 Policy semantics the rotation relies on (ADR 0012; `crates/bullet-wire/src/policy/live.rs`)

- `live_admission_enabled = true` is legal only under `v1alpha2`, `policy_generation >= 2`, with at least one
  `issuer_keys` entry that is `authority-signing` / `paseto-v4.public`, lists `provider-runner`, has no
  `revoked_at_unix_ms`, and overlaps the policy window; otherwise `LIVE_ADMISSION_REQUIRES_RUNNER_KEY`
  (or `LIVE_ADMISSION_REQUIRES_GENERATION`).
- `validate_at(now)` uses activation-inclusive, expiry/revocation-exclusive instants; a policy outside its
  window is `POLICY_NOT_ACTIVE`.
- Every grant expires within 15 s (`--ttl-ms` is clamped to the lease and to 15000) and its `grant_nonce`
  is single-use. Revoking the key therefore invalidates the future, not the past: at most 15 s of already
  minted grants remain valid, and none can be replayed.

The Kernel loader mirrors this rule since Kernel `0d848f6` (`crates/application/src/policy_snapshot/live.rs`;
ADR 0012's prose still describes the pre-mirror state and belongs to another lane). Under the committed
generation-1 policy the Kernel still refuses every grant with `POLICY_LIVE_ADMISSION_DISABLED`, so until
OD-A is ratified a rotation changes nothing observable.

### 1.3 Rotation procedure (what exists; untested as a whole)

There is no `authority rotate` command. Rotation is composed of the pieces above:

1. **Mint the successor.** The key path is fixed per data directory, and `keygen` never overwrites, so the
   successor is generated with a new `--key-id` in a **new, absolute data directory**:
   `authority keygen --data-dir /abs/bullet-data-2 --key-id launch-grant-beta`. Keep the printed
   `IssuerKeyV1`.
2. **Write policy generation N+1** (v1alpha2, `policy_generation` incremented) whose `issuer_keys` lists the
   new key and keeps the old key entry with `revoked_at_unix_ms` set to the cut-over instant (revocation is
   exclusive: the old key is dead from that millisecond). Store it outside the repositories, as in
   [`live-conformance.md`](live-conformance.md) §2.
3. **Ratify** in `AGENT_CHAT.md` at the family root, following ADR 0013 OD-A's line format with the new
   generation and `key_id`. No agent edits the policy.
4. **Point the Kernel at the new data directory and policy** (`BULLET_POLICY_PATH`, `--data-dir`). Because
   the ledger also lives under `<data-dir>`, this is a fresh ledger; that is acceptable only while no
   production ledger exists. A same-ledger rotation requires the old key file to be moved out of
   `<data-dir>/authority/` into operator custody first (a plain filesystem act the CLI does not perform),
   then `keygen` in place with the new `--key-id`. Either way the old key file is retained until
   `retain_until_unix_ms` (ADR 0005: retention at least one maximum token lifetime past expiry) and then
   destroyed by the operator.
5. **Prove** by re-running the live-conformance lane: minting with the old `issuer`/`key_id` must fail at
   `LIVE_ADMISSION_REQUIRES_RUNNER_KEY` or `LAUNCH_GRANT_KEY_UNKNOWN`; minting with the new key must reach
   `PONG` (or the neutral policy refusal while no ratified policy is loaded).

Rollback is the same as [`live-conformance.md`](live-conformance.md) §5: delete or replace the ratified
policy; everything returns to `POLICY_LIVE_ADMISSION_DISABLED`.

## 2. Release-signing key (not provisioned)

What exists as bytes:

- `release/allowed_signers` — one OpenSSH allowed-signers principal (`bot@jekko.ai`, namespace `git`), the
  Ed25519 identity `release verify --allowed-signers` uses for bundle signatures;
- `bullet-family release receipt-verify --receipt ABS --signature ABS --policy ABS` — verifies a canonical
  TOML release receipt against an explicit absolute-path signer policy that scopes each Ed25519 signer to
  exact receipt kinds and a validity interval, fixed OpenSSH namespace `bullet-farm-release-receipt-v1`
  (`src/release/receipt.rs`; ADR 0010). Environment variables, default search paths, and a policy bundled
  with the receipt are never trusted;
- `ReleaseManifest.release_signing_identity` in `src/release/schema.rs`.

What does not exist: a protected release signing key, its custody record, a signed release manifest, signed
archives, provenance, or any real receipt. The receipt verifier "has only ever checked fixtures"
(`release.receipt-contracts`). Without a key there is nothing to rotate; writing a rotation procedure now
would be documentation ahead of evidence.

The invariants any future procedure must satisfy are already fixed: authority and release keys have distinct
purposes and encodings; keys bind allowed audiences/kinds and carry activation, expiry, optional revocation,
and retention at least one maximum token lifetime past expiry (ADR 0005); the signer policy is an explicit
external input (ADR 0010). Custody is [ADR 0013](../decisions/0013-operator-decision-register.md) OD-E; the
gates it unblocks are `release.signatures`, `release.receipt-contracts`, and `release.provenance`.

## 3. Evidence class

Everything above is COMPONENT_PROOF (Kernel keygen/mint tests, hub receipt-verify tests) or unexecuted
procedure. No signer rotation has been performed and receipted on any host.
