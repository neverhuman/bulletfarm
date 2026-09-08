# Exact-source public publication

Status: component implementation; hosted CI and release certification unproved.
This is part of W0/WP-22 and G1/G8/G12. The existing gap register remains the
completion board.

`bullet-publish` captures four clean ordinary canonical checkouts, their exact
commit/tree/object format, and root templates from the captured Hub commit.
Linux is the admitted publication platform. Existing absolute manifests remain
valid; portable schema `1.3.0` uses exact basenames and `split_root = "."`
against an explicitly admitted root. Private source origins are preserved.

The versioned `publication.json` contains no aggregate SHA. External prepared,
publication, and CI records bind that SHA. Root files come only from selected
`publication/root/` templates; direct aggregate edits or extra root files refuse.

## Prepare and review

First prove and commit reviewed changes on local review branches in canonical
checkouts. Set explicit absolute paths and the authoritative current public main
OID. Request identifiers admit letters, digits, and hyphens.

```bash
family_root=/absolute/canonical/family
publication_store="$family_root/bullet-farm/.git/bullet-publication"
publication_bin=/absolute/proved/bullet-publish
publication_request=reviewed-packet-identifier
publication_base=exact-current-public-main-oid
"$publication_bin" inspect "$family_root"
"$publication_bin" prepare "$family_root" "$publication_store" \
  "$publication_request" "$publication_base"
```

The exclusive private store is a bare object database inside Hub metadata,
without a worktree or alternative member checkout. Durable intent precedes
effects. Identical requests reconstruct the same commit; changed inputs conflict.
Git writes explicitly fsync. Interrupted requests reconcile retained local refs.
Inspect the request and prepared commit before public effects.

## History admission and publication

Set `BULLET_PUBLICATION_GITLEAKS` to the absolute gitleaks 8.21.2 executable with
SHA256 `50b742abd7daad8bbddb6301f3017efb680632d9a5b3b4d8f137b3aac250e359`.
It executes sealed retained bytes. All reachable objects from four sources and
the aggregate are exposed in an unpushed synthetic root as complete text
additions, including deleted blobs, binary bytes, and commit/tree metadata.
Missing objects, findings, malformed output, timeout, more than 100,000 objects,
an object above 8 MiB, or more than 512 MiB total refuse. Private attempts retain
the inventory and execution reports. These finite scanner checks cannot promise
that every possible secret is detectable.

The default-extended policy excludes the exact public BLAKE3 fixture commitment
in `credential_projection_digest`, independently traced to `projection_digest()`
and historical Hub commits `9a5def68d367a27bf4f3d452bbbaf45982648b00` and
`5683c49298cfa7c393285e81497f80a4c71673a8`. No entire test path or arbitrary
digest is excluded. An independently reviewed exact match also excludes the public
Portal storage label `bullet-farm.csrf.v1` in historical `CSRF_STORAGE_KEY`
assignments; the separately returned CSRF token remains a storage value. The
scanner conformance test uses the committed policy, accepts that exact label,
and detects a different value in the same field. Repository ignore files and
inline allow directives are disabled.

Run history admission before any publication credentials are needed:

```bash
"$publication_bin" scan "$publication_store" "$publication_request"
```

Credentialed operations also require `BULLET_PUBLICATION_GH`: the absolute
GitHub CLI 2.62.0 Linux x86_64 executable with SHA256
`d2330508768dbbaa4c474353c77367e1690b1fe08c81497f787e40f9f53564d4`.
Its upstream release archive SHA256 is
`41c8b0698ad3003cb5c44bde672a1ffd5f818595abd80162fbf8cc999418446a`;
the retained executable runs with private configuration and a cleared environment.

Publication requires `BULLET_PUBLICATION_TOKEN`: an expiring GitHub App
installation token scoped to `neverhuman/bulletfarm` alone with Contents,
Workflows, and Pull requests write.
Operator tokens are refused. Ordinary PR jobs receive no publication,
subscription, or signing credentials. Retry with the original request:

```bash
"$publication_bin" push "$publication_store" "$publication_request"
"$publication_bin" pr "$publication_store" "$publication_request"
```

The tool atomically publishes lightweight source tags at
`refs/tags/bullet-source/v1/<member>/<commit>` and the review branch at
`refs/heads/publication/<request>`, using expected-old values and remote read-back.
Conflicting refs refuse. Response loss is reconciled from remote truth.
Synthetic scan refs never leave the store. Receipts record `integrated: false`.
The tool never writes main. Its pre-push main comparison is not a server-side
compare-and-swap on main.

The PR command retains its exact creation intent before a single attempted POST.
After response loss or process death it reconciles through reads of the original
request; absent proof of the first outcome refuses another POST. It binds the
Bot author, repository, source head, request body, and authoritative read-back.
Closed or merged PRs retain their original receipt; reconciliation still requires
the retained source and review refs.

An actual App-authored PR, independent human approval, tested merge subject, and
final main rerun remain required. Protect main after complete required checks operate:
PR review, stale-review dismissal, required checks, and no force-push/deletion.
Token rotation does not change request identity.

## Hosted proof scope

The discoverable bootstrap checks out the actual event SHA without persisted
credentials, scans source before building, provisions private Cargo configuration
and targets, runs publication tests, and reconstructs ordinary checkouts from
immutable source refs. Symlinks, hidden dirt, missing objects, paths, source refs,
template drift, and unequal trees refuse. `GITHUB_SHA` stays unchanged. The Rust
observation binds actual event/workflow/run/attempt, member subjects, and exact
artifact hashes; it is unsigned diagnostic evidence.

The template also runs the reconstructed Hub's actual `source-scan` lane with
a checksum-pinned scanner and a cleared child environment, stages its diagnostic,
and executes the exact source-bound semantic validator. Completion binds command
success, aggregate and member subjects, event/workflow/run/attempt, supporting
tool observations and artifact digests. The existing Rust validation envelope
retains `execution_evidence=false`; a separate unsigned completion record binds
the actual command. Supporting tool hashes describe a sampled interval, with
`tool_closure_admitted=false`.

`Publication bootstrap required` always runs, downloads the exact bootstrap and
member artifacts for that run/attempt, and rejects every predecessor result
except success. Missing completion outputs, stale subjects, unexpected files,
invalid bodies or changed digests refuse. This covers publication bootstrap and
one actual Hub source scan. The eight existing member workflows define 53 jobs
and 55 expanded invocations; complete activation, repeated real family proof,
portable Jankurai, admitted tools and workers, scheduled security/native jobs,
operational campaigns, final acceptance and protected integration remain unproved.
Local fixtures do not establish public hosted execution or release eligibility.

## Focused checks

Use an admitted private `CARGO_TARGET_DIR` and the exact scanner subject. Build
`bullet-publish` from the current Hub with `cargo build --locked --bin
bullet-publish --message-format=json`, and set `publication_bin` to the exact
executable path returned in Cargo's JSON output before running the wrappers:

```bash
cargo test --locked -p bullet-family --lib publication:: -- --test-threads=1
cargo test --locked -p bullet-family --lib family_lock
cargo test --locked -p bullet-family --lib coord::git::wave0
cargo test --locked -p bullet-family --test family_lock --test family_lock_git
BULLET_PUBLICATION_TEST_BIN="$publication_bin" bash publication/ci-tests.sh
```

The full Hub partition and independently reviewed identity pins remain required.
These checks admit no coordinator generation, provider call, incident recovery,
protected integration, or release gate.
