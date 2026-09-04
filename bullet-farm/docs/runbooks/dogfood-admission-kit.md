# Dogfood admission kit (operator)

Status: **PROPOSED — NOT EXECUTABLE.** Do not run any step below yet. An independent review on
2026-08-28 held this kit on eight findings; the corrections are folded in, but the loop it proposes does
not exist: `check dogfood`, `NOT_A_RELEASE_PROFILE`, the purpose-separated `DOGFOOD_RUN` operational record,
the dogfood enrollment, a dogfood-scoped policy binding, an admitted RFC 8785 policy producer/read-back, and
typed incident-inventory/W0 inputs to fresh Genesis are all unimplemented. It becomes executable
only after those exact typed predecessors land, their bounded hostile tests pass, and a fresh independent
safety review removes this `NOT EXECUTABLE` status.
Owner: the operator (these steps cannot be delegated to an agent)
Authority: [ADR 0015](../decisions/0015-dogfood-track.md) and OD-K in
[ADR 0013](../decisions/0013-operator-decision-register.md)
Supersedes: `.l7-bundle/OPERATOR-LIVE-ADMISSION-KIT.md` — see "What the old kit got wrong" below.

If its future gates are satisfied, this kit would admit only the **dogfood track**: a purpose-separated,
non-evidence `DOGFOOD_RUN` operational observation under `dogfood-local-v0`. It would clear no release gate,
close no OD-A…OD-J decision, mint no forge credential, and touch nothing on `127.0.0.1:8787`. Its eventual
steps would be limited to this host and the operator's UID.

## What is not true yet (read this before anything else)

The retired `.l7-bundle` kit said every mechanism existed and that a valid policy meant a live run was
"refused only by policy". That was false. So was some of this kit's first draft. The corrected facts:

**Where a run actually stops today.** Not at `RUNTIME_PROBE_UNAVAILABLE`. Production loads the provider
enrollment record *before* the key, lease, containment or probe steps
(`crates/application/src/live_conformance/steps.rs:81-95`, `enrollment.rs:219-259`), and no step in this
kit creates `<data-dir>/policy/enrollments/claude.json`. A run therefore stops at **`ENROLLMENT_MISSING`**.
Past that, production Claude refuses at **`PROBE_EXECUTION`** (`probe_steps.rs:307-343`), not at ADMISSION.

**The key this kit mints cannot be used by the command it names.** `bullet provider live-conformance`
exposes no issuer/key flags and hardcodes issuer `bullet-kernel`, key `launch-grant-alpha`
(`apps/bullet/src/provider.rs:41-54,153-168`), looked up exactly at `steps.rs:85-101`. A key minted as
`ben-host` / `dogfood-2026-08` will never validate for that command. Either mint with the hardcoded
identities or wait for a dogfood command that takes them as arguments.

**This policy is not scoped to dogfood.** `PolicySnapshotV1` has no profile or audience field for it
(`bullet-wire/src/policy.rs:286-343`) and `validate_live_admission` checks only a global boolean,
the generation, and the presence of a provider-runner key (`policy/live.rs:44-64`). Setting
`live_admission_enabled = true` therefore clears the POLICY step for **every** guarded live route, not
only a future dogfood command. A separately typed dogfood audience/operation binding that the general
live and release paths refuse is an **engineering predecessor of OD-K**, not a detail.

**Operator provenance here is social, not cryptographic.** Kernel policy loading checks regularity, size,
stable inode/length and canonical content — but not owner, mode, signature, or any OD-K witness
(`policy_snapshot/load.rs:62-103`); key loading checks mode 0600 and the current UID, which every agent on
this host shares. Nothing in Hub or Kernel source reads OD-K or `AGENT_CHAT.md`. So an agent-created policy
and key are **not** mechanically distinguishable from the operator's. The protection is that you, a person,
created these bytes and no agent was told to — that is custody by convention on this host. Do not describe
it as anything stronger until an operator trust root inaccessible to agents exists.

**A same-UID Claude turn must not inherit your cached credentials.** The enrollment record has no
service-identity or credential-handle field (`enrollment.rs:151-174`) and admission currently supplies
empty credential targets (`steps.rs:395-408`). Explicit credential projection and a filesystem boundary
limiting the child to the private clone are required before a real turn — the current egress work proves
network isolation only.

## Preconditions

**The commands below are proposed procedure sketches, not approved commands. Do not run or adapt any of
them until the typed dogfood scope, exact enrollment and credential projection, filesystem containment,
bounded budgets, purpose-separated operational record, admitted canonical policy producer, complete incident
inventory, exact mechanically revalidated four-repository W0 subject, and fresh independent safety review all
exist.** Present-tense descriptions of already implemented helper behavior do not make this
procedure executable.

- Linux, this host, your UID. `unshare nsenter slirp4netns nft curl jq stat grep sed cat kill` on `PATH`
  (all present here).
- The admitted exact W0-subject producer and consumer described in Step 4. Four independent `git rev-parse`
  outputs or a chat line cannot bind repository identity, trees, cleanliness, claim high-water, and one atomic
  drift refusal.
- Nothing to install. Do **not** create keys or policies anywhere inside a repository.

## Step 1 — private data directory

```bash
set -euo pipefail
umask 077
operator_home=/home/ubuntu
data_root="$operator_home/.bullet-data"
identity_format='%d:%i:%F:%a:%u'

# Refuse a substituted or write-open ancestor before the first mutation.
for ancestor in / /home "$operator_home"; do
  test ! -L "$ancestor"
  test -d "$ancestor"
  ancestor_mode="$(stat -Lc '%a' "$ancestor")"
  test "$((8#$ancestor_mode & 0022))" -eq 0
done
test "$(stat -Lc '%u' /)" -eq 0
test "$(stat -Lc '%u' /home)" -eq 0
test "$(stat -Lc '%u' "$operator_home")" -eq "$(id -u)"
ancestry_identity="$(stat -Lc "$identity_format" / /home "$operator_home")"

exec {operator_home_fd}< "$operator_home"
operator_home_fd_path="/proc/$$/fd/$operator_home_fd"
test "$(stat -Lc "$identity_format" "$operator_home_fd_path")" = \
  "$(stat -Lc "$identity_format" "$operator_home")"

data_root_via_home="$operator_home_fd_path/.bullet-data"
test ! -L "$data_root_via_home"
if test ! -e "$data_root_via_home"; then
  mkdir -m 700 -- "$data_root_via_home"
fi
test ! -L "$data_root_via_home"
test ! -L "$data_root"
test "$(stat -c '%F:%a:%u' "$data_root")" = "directory:700:$(id -u)"

exec {data_root_fd}< "$data_root_via_home"
data_root_fd_path="/proc/$$/fd/$data_root_fd"
test "$(stat -Lc "$identity_format" "$data_root_fd_path")" = \
  "$(stat -Lc "$identity_format" "$data_root_via_home")"
test "$(stat -Lc "$identity_format" "$data_root_fd_path")" = \
  "$(stat -Lc "$identity_format" "$data_root")"

for leaf in authority policy receipts incidents baselines; do
  leaf_via_data="$data_root_fd_path/$leaf"
  leaf_public="$data_root/$leaf"
  test ! -L "$leaf_via_data"
  if test ! -e "$leaf_via_data"; then
    mkdir -m 700 -- "$leaf_via_data"
  fi
  test ! -L "$leaf_via_data"
  test ! -L "$leaf_public"
  test "$(stat -c '%F:%a:%u' "$leaf_public")" = "directory:700:$(id -u)"
  test "$(stat -Lc "$identity_format" "$leaf_via_data")" = \
    "$(stat -Lc "$identity_format" "$leaf_public")"
done

# Rebind the complete ancestry after creation; movement is a refusal.
for ancestor in / /home "$operator_home"; do
  test ! -L "$ancestor"
done
test "$(stat -Lc "$identity_format" / /home "$operator_home")" = \
  "$ancestry_identity"
test ! -L "$data_root"
test "$(stat -Lc "$identity_format" "$operator_home_fd_path")" = \
  "$(stat -Lc "$identity_format" "$operator_home")"
test "$(stat -Lc "$identity_format" "$data_root_fd_path")" = \
  "$(stat -Lc "$identity_format" "$data_root")"
exec {data_root_fd}<&-
exec {operator_home_fd}<&-
```

Everything the track owns lives here: `authority/`, `policy/`, `tools/`, `forges/`, `runs/`,
`receipts/`, `spend/`, `incidents/`, `baselines/`. The exact `authority/` output directory must exist,
be non-symlinked, owner-only, and read back through the retained data-root descriptor before key generation
starts. Step 1 refuses the complete `/` → `/home` → `/home/ubuntu` → `.bullet-data` ancestry before
its first mutation and rebinds every created leaf to that descriptor. Never point a daemon at
`./target/demo` — that is inside the shared cargo target
and any `cargo clean` destroys the ledger.

## Step 2 — authority key (you hold the private half)

```bash
set -euo pipefail
umask 077
export LC_ALL=C
operator_home=/home/ubuntu
data_root="$operator_home/.bullet-data"
authority_dir="$data_root/authority"
keygen_output="$authority_dir/keygen.out"
identity_format='%d:%i:%F:%a:%u'

# Re-authenticate and retain every directory in the private ancestry.
for ancestor in / /home "$operator_home"; do
  test ! -L "$ancestor"
  test -d "$ancestor"
  ancestor_mode="$(stat -Lc '%a' "$ancestor")"
  test "$((8#$ancestor_mode & 0022))" -eq 0
done
test "$(stat -Lc '%u' /)" -eq 0
test "$(stat -Lc '%u' /home)" -eq 0
test "$(stat -Lc '%u' "$operator_home")" -eq "$(id -u)"
ancestry_identity="$(stat -Lc "$identity_format" / /home "$operator_home")"

exec {operator_home_fd}< "$operator_home"
operator_home_fd_path="/proc/$$/fd/$operator_home_fd"
test "$(stat -Lc "$identity_format" "$operator_home_fd_path")" = \
  "$(stat -Lc "$identity_format" "$operator_home")"

data_root_via_home="$operator_home_fd_path/.bullet-data"
test ! -L "$data_root_via_home"
test ! -L "$data_root"
test "$(stat -c '%F:%a:%u' "$data_root")" = "directory:700:$(id -u)"
exec {data_root_fd}< "$data_root_via_home"
data_root_fd_path="/proc/$$/fd/$data_root_fd"
test "$(stat -Lc "$identity_format" "$data_root_fd_path")" = \
  "$(stat -Lc "$identity_format" "$data_root")"

authority_dir_via_data="$data_root_fd_path/authority"
test ! -L "$authority_dir_via_data"
test ! -L "$authority_dir"
test "$(stat -c '%F:%a:%u' "$authority_dir")" = "directory:700:$(id -u)"
exec {authority_dir_fd}< "$authority_dir_via_data"
authority_dir_fd_path="/proc/$$/fd/$authority_dir_fd"
test "$(stat -Lc "$identity_format" "$authority_dir_fd_path")" = \
  "$(stat -Lc "$identity_format" "$authority_dir")"

# Bash noclobber opens a new output with O_CREAT|O_EXCL. With O_EXCL, even a
# dangling final symlink is refused; umask 077 makes the new file mode 0600.
keygen_output_via_authority="$authority_dir_fd_path/keygen.out"
test ! -L "$keygen_output_via_authority"
test ! -e "$keygen_output_via_authority"
set -o noclobber
exec {keygen_output_fd}> "$keygen_output_via_authority"
set +o noclobber
keygen_output_fd_path="/proc/$$/fd/$keygen_output_fd"
keygen_output_identity="$(stat -Lc '%d:%i' "$keygen_output_fd_path")"
test ! -L "$keygen_output"
test "$(stat -Lc '%d:%i' "$keygen_output_via_authority")" = \
  "$keygen_output_identity"
test "$(stat -c '%d:%i' "$keygen_output")" = "$keygen_output_identity"
test -f "$keygen_output_fd_path"
test "$(stat -Lc '%a:%u:%h:%s' "$keygen_output_fd_path")" = \
  "600:$(id -u):1:0"

# Movement after admission is a refusal before the one-use producer starts.
for ancestor in / /home "$operator_home"; do
  test ! -L "$ancestor"
done
test "$(stat -Lc "$identity_format" / /home "$operator_home")" = \
  "$ancestry_identity"
test ! -L "$data_root"
test ! -L "$authority_dir"
test "$(stat -Lc "$identity_format" "$operator_home_fd_path")" = \
  "$(stat -Lc "$identity_format" "$operator_home")"
test "$(stat -Lc "$identity_format" "$data_root_fd_path")" = \
  "$(stat -Lc "$identity_format" "$data_root")"
test "$(stat -Lc "$identity_format" "$authority_dir_fd_path")" = \
  "$(stat -Lc "$identity_format" "$authority_dir")"

cd /home/ubuntu/bullet/bullet-kernel
cargo run --locked -p bullet -- authority keygen \
  --data-dir "$data_root" \
  --issuer ben-host --key-id dogfood-2026-08 \
  1>&"$keygen_output_fd"

# Flush the exact descriptor target, bind it once more, then close the writer.
sync -- "$keygen_output_fd_path"
sync -- "$authority_dir_fd_path"
test "$(stat -Lc '%d:%i' "$keygen_output_fd_path")" = \
  "$keygen_output_identity"
test ! -L "$keygen_output"
test "$(stat -c '%d:%i' "$keygen_output")" = "$keygen_output_identity"
test "$(stat -c '%F:%a:%u:%h' "$keygen_output")" = \
  "regular file:600:$(id -u):1"
test "$(stat -Lc '%s' "$keygen_output_fd_path")" -gt 0
exec {keygen_output_fd}>&-

# Independently reopen through the retained authority descriptor and read all bytes.
test ! -L "$keygen_output_via_authority"
test ! -L "$keygen_output"
exec {keygen_read_fd}< "$keygen_output_via_authority"
keygen_read_fd_path="/proc/$$/fd/$keygen_read_fd"
test "$(stat -Lc '%d:%i' "$keygen_read_fd_path")" = \
  "$keygen_output_identity"
test "$(stat -c '%d:%i' "$keygen_output")" = "$keygen_output_identity"
test "$(stat -Lc '%F:%a:%u:%h' "$keygen_read_fd_path")" = \
  "regular file:600:$(id -u):1"
keygen_output_size="$(stat -Lc '%s' "$keygen_read_fd_path")"
keygen_readback="$(cat <&"$keygen_read_fd")"
test "$(printf '%s\n' "$keygen_readback" | wc -c)" -eq "$keygen_output_size"

mapfile -t keygen_lines <<<"$keygen_readback"
test "${#keygen_lines[@]}" -ge 4
test "${keygen_lines[0]}" = \
  "key_file: /home/ubuntu/.bullet-data/authority/launch-grant.key"
public_key_line_pattern='^public_key_hex: ([0-9a-f]{64})$'
[[ "${keygen_lines[1]}" =~ $public_key_line_pattern ]]
printed_public_key="${BASH_REMATCH[1]}"
test "${keygen_lines[2]}" = "issuer_key_v1: {"
last_line_index=$((${#keygen_lines[@]} - 1))
test "${keygen_lines[$last_line_index]}" = "}"

issuer_json="$(
  printf '%s\n' "$keygen_readback" | sed -n '3,$p' | \
    sed '1s/^issuer_key_v1: //'
)"
for issuer_time_field in \
  activates_at_unix_ms expires_at_unix_ms retain_until_unix_ms
do
  issuer_time_pattern="^[[:space:]]*\"$issuer_time_field\"[[:space:]]*:"\
'[[:space:]]*(0|[1-9][0-9]*),?[[:space:]]*$'
  test "$(printf '%s\n' "$issuer_json" | grep -Ec "$issuer_time_pattern")" -eq 1
done
printf '%s\n' "$issuer_json" | jq --stream -se '
  ([.[] | select(length == 2) | .[0][0]] | sort) == [
    "activates_at_unix_ms", "algorithm", "audiences", "expires_at_unix_ms",
    "issuer", "key_id", "key_purpose", "public_key",
    "retain_until_unix_ms", "revoked_at_unix_ms", "schema_version"
  ]' >/dev/null
printf '%s\n' "$issuer_json" | jq -se --arg printed "$printed_public_key" '
  length == 1
  and (.[0] |
    type == "object"
    and (keys == [
      "activates_at_unix_ms", "algorithm", "audiences", "expires_at_unix_ms",
      "issuer", "key_id", "key_purpose", "public_key",
      "retain_until_unix_ms", "revoked_at_unix_ms", "schema_version"
    ])
    and .schema_version == "v1alpha1"
    and .issuer == "ben-host"
    and .key_id == "dogfood-2026-08"
    and .key_purpose == "authority-signing"
    and .algorithm == "paseto-v4.public"
    and .audiences == ["provider-runner"]
    and .public_key == $printed
    and (.activates_at_unix_ms | type == "number")
    and (.expires_at_unix_ms | type == "number")
    and (.retain_until_unix_ms | type == "number")
    and .activates_at_unix_ms == (.activates_at_unix_ms | floor)
    and .expires_at_unix_ms == (.expires_at_unix_ms | floor)
    and .retain_until_unix_ms == (.retain_until_unix_ms | floor)
    and .activates_at_unix_ms >= 0
    and .activates_at_unix_ms <= 9007199254740991
    and .expires_at_unix_ms > .activates_at_unix_ms
    and .expires_at_unix_ms <= 9007199254740991
    and .retain_until_unix_ms >= .expires_at_unix_ms
    and .retain_until_unix_ms <= 9007199254740991
    and .revoked_at_unix_ms == null
  )' >/dev/null

# Keep the read descriptor until strict decode and every final binding succeed.
test "$(stat -Lc '%d:%i' "$keygen_read_fd_path")" = \
  "$keygen_output_identity"
test "$(stat -c '%d:%i' "$keygen_output")" = "$keygen_output_identity"
test "$(stat -c '%F:%a:%u:%h:%s' "$keygen_output")" = \
  "regular file:600:$(id -u):1:$keygen_output_size"
for ancestor in / /home "$operator_home"; do
  test ! -L "$ancestor"
done
test "$(stat -Lc "$identity_format" / /home "$operator_home")" = \
  "$ancestry_identity"
test ! -L "$data_root"
test ! -L "$authority_dir"
test "$(stat -Lc "$identity_format" "$data_root_fd_path")" = \
  "$(stat -Lc "$identity_format" "$data_root")"
test "$(stat -Lc "$identity_format" "$authority_dir_fd_path")" = \
  "$(stat -Lc "$identity_format" "$authority_dir")"
exec {keygen_read_fd}<&-
exec {authority_dir_fd}<&-
exec {data_root_fd}<&-
exec {operator_home_fd}<&-
```

It writes the private key under `<data-dir>/authority/` and prints `key_file`, `public_key_hex`, and a
complete `issuer_key_v1` record already carrying `key_purpose: authority-signing`,
`algorithm: paseto-v4.public` and `audiences: ["provider-runner"]` — exactly what ADR 0012 requires.
The block rebinds the full private ancestry, creates `keygen.out` once beneath the retained `authority/`
descriptor, and keeps that same mode-0600 sink descriptor as keygen's stdout for the entire one-use producer;
no pathname-reopening `tee` participates. It fsyncs and closes the writer, independently reopens through the
retained directory, and accepts the full read-back only when descriptor/path identity, owner/mode/link/size,
the closed issuer field set, complete single-object decode, and printed/embedded public-key equality all hold.
Keep `keygen.out`; only after these checks may the future canonical policy producer consume it.
If any command fails, stop and preserve the key plus output exactly as found; never rerun keygen against the
same data directory or replace either file.

## Step 3 — generation-2 policy, outside every repository

Derive it from the committed offline policy so every conservatism field is inherited rather than retyped.
There is deliberately no shell producer here: plain `jq` output is pretty JSON with a trailing newline, while
Kernel admits only byte-exact RFC 8785 encoding. Installing that output would fail `NON_CANONICAL_POLICY`.
Before this step can become executable, an independently accepted producer must strictly parse the base policy
and the complete keygen read-back, perform exactly these four semantic mutations, serialize with the shared RFC
8785 implementation, create the mode-0600 output without overwrite, reopen it, and require equality with a second
canonical encoding before returning its digest:

1. change only the top-level `schema_version` to `v1alpha2`;
2. set `policy_generation` to `2`;
3. set `sandbox_policy.live_admission_enabled` to `true`; and
4. append the one exact admitted `issuer_key_v1` record.

Every nested policy `schema_version`, including `sandbox_policy.schema_version`, remains `v1alpha1`. Review the
typed semantic diff and canonical read-back before admission — this file is your authority, not the producer's.
In particular,
`route_policy.evolutionary_authority` stays `false`, `maximum_lease_ttl_seconds` stays ≤ 15,
`arbitrary_shell_gates` stays `false`, and `unknown_quota_is_headroom` stays `false`; any of those
flipping makes the policy `UNSAFE_POLICY` and it will be refused — correctly.

Do **not** copy `crates/bullet-wire/tests/fixtures/policy-v1alpha2-live-enabled.json`. Its keys are
fixture-labelled, nobody holds their private halves, and enrollment refuses them by label.

The loader reads `BULLET_POLICY_PATH` (absolute) or `<data-dir>/policy/policy.json`.

## Step 4 — coordinator: retire the frozen generation, start a fresh one

The 2026-08-26 incident left `events.jsonl` frozen at mode 0400 with no `CURRENT`. Its sanctioned
recovery needs a reviewer who is not you. Only after the typed predecessors and fresh safety review pass
could a later operator ratification of OD-K permit retaining it as **incident evidence** and starting a
fresh Genesis.

**Precondition, not a suggestion.** Do not retire anything until W0 is actually clean: four committed
heads, no active claim in the log, and an independent review of that state. The retirement is ordered
*after* W0, not before it — an earlier draft of this kit had the `mv` before its own condition.

**Two typed bindings are mandatory and do not exist.** First, an admitted descriptor-bound producer must write
a create-once RFC 8785 incident inventory under `/home/ubuntu/.bullet-data/incidents/`. It must cover every
byte-sorted relative path beneath the retained coordinator directory (including directories), its file type,
owner, mode, link count, size, and every regular-file digest; bind the source directory identity and destination
name; persist a domain-separated inventory digest; and verify the same complete tree after the no-clobber move.
Two console `find` listings plus three loose hashes are neither durable nor complete and must not authorize
retirement.

Second, a typed create-once W0 subject under `/home/ubuntu/.bullet-data/baselines/` must bind exactly four named
members — Hub, Kernel, BulletGit, and Portal — with each canonical repository identity, commit OID, tree OID,
clean index/worktree/untracked state, the zero-active-claim ledger high-water, and an independent-review subject.
`GenesisInput`/`GenesisManifestBody` currently bind one bootstrap commit and a path list
(`src/coord/model/api.rs:55-64`, `generation/manifest/types.rs:82-94`), not those four subjects. Before any
retirement or init, the mutation command must require both canonical record paths and digests, retain directory
authority, re-read every inventory and W0 field in the same locked transition, and refuse any path, record,
claim, commit, tree, index, worktree, untracked-file, or destination drift. It must persist the W0 digest beside
the Genesis authority (or extend the typed manifest) so later replay can verify it.

No accepted producer or consuming `coord init` interface implements these requirements, so no retirement or
Genesis command is shown. Do not approximate them with `find`, `sha256sum`, `mv`, the current one-commit
`coord init`, or a chat witness. The frozen `events.jsonl` remains untouched at mode 0400; DF-R7a/R7b remain
open and owed.

## Step 5 — ratification

After every predecessor and safety review passes, the operator may append the **single canonical OD-K
witness template** from [ADR 0013 § OD-K](../decisions/0013-operator-decision-register.md#od-k-dogfood-track-admission-internal-use-only).
Use that template verbatim and fill every dogfood-scope, policy, key, enrollment, executable/protocol/model/profile,
service-identity, credential-handle, invocation/spend, validity/revocation, rollback, frozen-generation inventory,
Genesis, exact W0, eligibility, provider-write, and Jeryu field. Do not maintain a weaker duplicate here.

The line is a social coordination witness only. On this same-UID host, an operator-looking line, policy, or
key is not mechanically attributable to the operator. Runtime authority must come from the future typed
scope, admitted exact enrollment and credential projection, and bounded command inputs; the witness cannot
supply or replace any of them.

If you also want the policy generation recorded in OD-A's own format, append a second line tagged
`POLICY-GENERATION-2` and mark it **dogfood-only; OD-A remains OPEN** — a dogfood policy does not enroll
a provider for `release.provider.claude`.

## Step 6 — verify, including what must still refuse

Several required forms do not exist yet, and the refusal an earlier draft promised was the wrong one. Corrected:

| Check | Available today? |
| --- | --- |
| `coord status`, current `coord init` | `status` and a one-Hub-commit `init` exist only in the schema-2 coordinator tree. The W0 head must retain safe read-only status, but the present `init` cannot consume the complete incident inventory or exact four-repository W0 subject and therefore is **not** the command this kit requires. |
| `transaction --json`, `check release` | Yes |
| `provider live-conformance --provider claude` | Runs, but **stops at `ENROLLMENT_MISSING`**, not at the probe — this kit creates no enrollment record, and production reads it before key, lease, containment or probe (`steps.rs:81-95`). With an enrollment present it would then stop at `PROBE_EXECUTION` with `RUNTIME_PROBE_UNAVAILABLE` (`probe_steps.rs:307-343`). Neither refusal proves the key validated: the command hardcodes issuer `bullet-kernel` / key `launch-grant-alpha`, so a `ben-host` key is never looked up. |
| `check dogfood --json` | **No — not implemented.** `dogfood` appears nowhere in `src/`. The interim board is `python3 scripts/dogfood-board.py --json`, which always exits 0 and so can report but never fail. |
| `check release --profile dogfood-local-v0` | **No — the profile does not exist.** Today this returns `UNKNOWN_RELEASE_PROFILE`, not the typed `NOT_A_RELEASE_PROFILE` ADR 0015 describes. |

```bash
bullet-family --root /home/ubuntu/bullet coord status --json --all   # expect exit 0, a Genesis generation
cd /home/ubuntu/bullet/bullet-kernel
cargo run --locked -p bullet -- transaction --json          # expect TRANSACTION_PROOF_UNAVAILABLE
cd /home/ubuntu/bullet/bullet-farm
cargo run --locked --bin bullet-family -- check release \
  --profile self-hosted-v1 \
  --receipts /home/ubuntu/.bullet-data/receipts \
  --json                                                        # expect exit 3: BLOCKED; 27 selected, 0 PASS
```

The absolute registry is the private directory precreated in Step 1; substituting a relative, ambient, or
different registry does not perform this check. The last two commands are the point of the exercise: whatever
else changed, **nothing was promoted**. The exact invocation must select 27 gates for `self-hosted-v1`, report
zero `PASS`, return status `BLOCKED`, and exit 3. Any other profile, count, result, or exit code is drift: stop
and treat it as an incident.

## Revocation and rollback

**Never edit or delete the ratified policy in place.** The digest of that exact file is what OD-K and the
Genesis manifest bind; changing a field inside it invalidates the binding you are trying to preserve, and
a recursive delete of `<data-dir>/policy` destroys the enrollment and audit inputs a later review needs.
An earlier draft of this kit told you to do both. It was wrong.

Roll forward instead, monotonically:

1. Author a **successor** policy generation (`policy_generation` strictly greater than the ratified one)
   at a new path, with `sandbox_policy.live_admission_enabled: false`, and with the dogfood key carrying a
   `revoked_at_unix_ms` in the successor only.
2. Install it atomically beside the old one — never over it — and keep every prior policy, enrollment,
   run and operational-record byte in place.
3. Point the loader at the successor, restart, and **verify the typed refusal**: the run must now stop at
   the policy step. Unsetting `BULLET_POLICY_PATH` is not a rollback; the default
   `<data-dir>/policy/policy.json` is still read.
4. Append a `— operator — … — REVOKED:` line naming the superseded generation, the successor generation,
   both digests, and the observed refusal.

The frozen coordinator generation is never touched by any of this.
