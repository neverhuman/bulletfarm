# Health checkpoint after the production audit

Status: **Corrective packets accepted locally; production and release remain incomplete**

Last updated: 2026-09-09 11:10 UTC

This checkpoint follows the [deep audit](deep-audit-20260909.md),
[full-product plan](full-product-dogfood-plan.md) and [G1–G18 register](product-gaps.md).
The audit remains a frozen observation of its original subjects. Its 101-document
coverage has been followed by a 103-document inventory and review of subsequent
changes, including the two new audit/checkpoint documents.

Production source read-back at Kernel5858e843/Git6cf10e70 confirms that durable
command/outbox/claim infrastructure exists, while the worker still requires a
simulator receipt and settles UNKNOWN; the standalone Runner is sim-only and
the product verifier refuses signed-intent admission. Native Claude read-only
dispatch is a separate component. The expanded [development closeout](xbabe2-development-closeout.md)
orders actual production admission, native dispatch, durable stop/Candidate
completion, independent verifier and human integration; it does not grant a
harness-recording exception or promote a profile.

Portal29d02c348e66bd4da6bd4582a33e2932b3c72940 / treee6f8dc714761d9ca6a8c6f784ef29a944fd191fa completed the full standalone required lane on9September:183 unit tests,14 mocked browser tests, five bundle tests and all five fresh lane observations. Independent readback9c0343ca verified1,431 artifact hashes, immutable inputs and released process/lock custody; elapsed92.234732seconds. This adds no real-farmd, coverage, family, hosted or provider acceptance. Hub6ad38025 integrates the media CI routing after all three direct suites and independent review3c272047; complete mapped/fullHub proof remains pending.

## Accepted Kernel work

| Commit | Reviewed change | Executed evidence |
| --- | --- | --- |
| `79cae6e7084d1a651e001967f44cbacb9dd3f054` | Restore strict raw JSON parsing, consume the unchanged authoritative formal fixtures, and restore truthful generated/manual ownership metadata. | Three formal tests and one schema agreement test passed. Invalid comments, malformed JSON and raw string newlines are rejected; valid escaped newlines retain their meaning. |
| `02bf7c5ad0aea398a6b82b9d98ff60dbdde2a09c` | Reconcile the actual test inventory and active fixture instructions. | Enumeration found 1,121 total and 1,075 standalone identities, adding exactly the new parser test. The existing inventory checker passed. |

The complete Kernel `bash scripts/ci-local.sh required` then passed on
`02bf7c5ad0aea398a6b82b9d98ff60dbdde2a09c`, tree
`6fd94de4da0514be66dce7dc79360078225fa4cd`, in **504.998 seconds**.
Its 1,075 standalone and 34 contract tests passed, together with the mapped lint,
security and documentation checks. The three host-dependent egress and nine
family tests are outside this standalone campaign. Peak resident memory was
697,800 KiB. Source, selected tools and raw Cargo configuration stayed unchanged;
the checkout was clean and the proof lock was released.

The run supplied `CARGO_NET_GIT_FETCH_WITH_CLI=true` and offline Cargo explicitly.
An earlier inventory attempt failed its configuration guard and is retained.
The successful warm enumeration fixed the sole known configuration difference
with an explicit override; it was inventory evidence, not a Rust test run.

BF-A11's active consumer is repaired. Unused commented formal copies and the
declaration-only binding introduced by the earlier workaround remain for a
separate reviewed cleanup. This checkpoint supplies no blanket acceptance of
earlier commits, auditor scores or other member trees.

## Accepted private capture and rendering

| Commit | Reviewed change | Executed evidence |
| --- | --- | --- |
| `3f976ab43741c3b68abca96052b0ca8c2d98f563` | Preserve private captures, true exits, interruptions and cleanup results; distinguish a forked child from provider execution. | All 11 recorder tests passed, including signals, missing executable, EOF, limits, custody and synthetic campaign failures. |
| `3bb772c72302ccd75161584b76bf8880e7f7ef9f` | Preserve inputs and exact-color Portal masters; independently check GIF pixels, geometry and artifact identity with selected local tools. | All three renderer tests passed. Subsequent continuous capture exposed a GIF timing defect; earlier acceptance covers RGB/geometry and retained source timestamps. |
| `742a771d5786b81925867fb419f7677e74436094` | Sample Portal frames serially between eight landmarks, retain actual timing/gaps, and drain capture writes before closing the browser. | Six actual Chromium tests plus all 11 recorder regressions passed; focused lint, syntax and 17 selected/completed identities agreed. Synthetic pages and API responses were used. |

`8248c45c61fee68921761900e06210f2e00632e6`, tree
`c462ff9fbf88f2dcfefc31c6ba3237938202be72`, subsequently repairs native
timestamp measurement and GIF encoding. All three existing renderer identities,
focused Clippy and syntax checks passed. Independent review verified 957 retained
artifacts; the total remains 20 distinct media test identities.

The corresponding integration trees are `82d081386c5e77d9a441768072db4e7a76104b6e`,
`300cdee789e5b33758df8bb8ab824d70b3d25554` and
`369044a78c7a0d1bb82e8e204e65c3474a32310a`. Independent reviews verified the
respective 811, 641 and 390 retained artifact entries. These are **20 distinct
focused test identities**, with the 11 recorder regressions repeated during
continuous capture verification. They are not a complete Hub or family run.

The latest synthetic browser tour retained 17 original 1920 × 1080 PNGs over
6.062 seconds, with an observed 2.752 frames/second and actual gaps. Requested
4 Hz is a ceiling, not a guaranteed frame rate. The first Rust browser attempt
passed five cases and failed one Chromium screenshot operation. That failure
remains retained; a reviewed fixture paint-readiness precondition preceded the
successful six-case run. Screenshot failures still abort capture.

A separate actual render and independent check of those 17 PNGs passed in
6.524 and 3.420 seconds. All 112 artifacts and 40 successful child-command records
were reviewed. Original RGB and geometry match the lossless master; GIF pixels
are explicitly `QUANTIZED`. Original input JSON remains exact.

**The encoded timing defect is repaired and independently verified locally.**
The original 112-artifact roundtrip above remains historical RGB/geometry evidence:
it flattened variable gaps to 35–36 centiseconds. The corrected renderer preserves
native timestamp offsets, checks source-derived master PTS, and validates actual
GIF graphic-control delays and the final display hold. Coherently rehashed but
retimed masters, shifted origins, flattened GIFs and altered final holds refuse.

A new render/check of the same 17 original PNGs passed in 5.996/3.367 seconds.
The master is byte-identical to the earlier lossless master. Maximum measured
source timestamp error is 1.225 microseconds for the master and 4.883 milliseconds
for the GIF, whose absolute timestamps use centisecond rounding. Its final
34-centisecond hold repeats the last encoded gap; this is an explicit display
policy, not an observed source duration. Encoded intervals shorter than two centiseconds, or colliding after rounding,
refuse instead of dropping frames. These bounds do not claim temporal
losslessness. Original inputs, failed attempts and corrected outputs remain separate.

No real Portal application, subscription, coding transaction or public export
is admitted by these synthetic browser tests. The [capture runbook](../demo-gif/README.md)
records the actual interfaces and limits. Native CLI illustration prompts request
explanations without tools or edits; genuine coding demonstrations still require
the production transaction and provider campaign. Observed tool files and host
fonts do not establish a complete portable runtime/font closure.

## Inventory, documentation and review corrections

`49b95c23764514efeb166584c55aae7521569f21`, tree
`cecc4ef6be0fe0546ca090d5d32e314fe2c251a9`, changes only four inventory literals.
Actual Nextest enumeration found **819 total identities: 600 Hub and 219 wire**.
All previous 799 remain, with exactly the 11 recorder, three renderer and six
browser additions; no removals, ignored tests or partition overlap were found.
The unchanged inventory checker passed in 34.421 seconds. Enumeration does not
mean that all 819 tests were executed.

`db91f0d1cc87ef582634462ada031d88375953a6` admits only Boolean
`net.git-fetch-with-cli` configuration while retaining compiler/dependency/alias
controls. The full existing compiler-control canary passed in 48.747 seconds,
including 16 new valid/invalid configuration cases. The global Cargo configuration
was preserved. The earlier `just docs` refusal on `43d601c` remains recorded.

The complete mapped **`just docs` passed in 217.835 seconds** on
`4cf7a98794517145299d2ca674e9c0f23cc9bd99`, tree
`9c447224678de954eaa5b8324d38c8108be50456`, plus the recorded continuous-capture
source overlay. It ran rustdoc, the existing zero-doctest suites, 90-file link
checks, dogfood/release-truth checks, strict BLOCKED doctor readback and two
independent component-media reconstructions. All 870 source, ten Cargo-config,
15 selected-tool and 177 sysroot-file guards stayed unchanged; the proof lock
was released. This was mapped documentation proof on a checkout containing
retained untracked media, not clean complete Hub/family or hosted CI acceptance.

`6298124` and `a5d81f7` withdraw the newly introduced fast wrapper and competing
closeout page after independent review. The wrapper could count one checkout
four times, hid Cargo-home inputs, and removed private-target selection; the
page supplied invalid commands and weaker production/CI exit criteria. Original
commit `e0286b9` and all preimages remain preserved. The existing plan and lane
commands remain authoritative.

`4cf7a987` corrects historical media documentation. The unadmitted live-media
hook and false authentication claim were restored to the existing component
boundary after preserving their bytes. The live checker still refuses when its
required collection is absent and still reconstructs a valid collection when
present. `bc1176d038dccedb614b0c4ca517ab906f9449c9` adds explicit private tool-receipt
selection against a reviewed source digest; `962dadf` documents that interface.
All 16 rendering and 15 normalization fixture groups passed. The actual canonical
receipt consumer also admitted the real private input, 26 selected executables
and 107 runtime files in 1.587 seconds; after relocation it returned identical
metadata in 1.242 seconds with the old canonical receipt absent. This exercised
receipt selection, without image execution, media generation or provider calls.

All 57 previously untracked files (13,479,239 bytes) now have verified private
copies and retained originals in durable custody. Fifteen separate batches of at
most four paths used same-filesystem, no-replace rename, with unchanged tracked
source, index and HEAD. No bytes or Git objects were deleted. The historical
theme remains explicitly authored historical configuration; it was not labeled
generated or hidden with an exclusion. Source-pin selection keeps the current
receipt usable outside the checkout. Seven filesystem tests and independent
copy/procedure reviews preceded relocation. Interrupted operations require
reconciliation; this one-off transfer is not a production crash campaign.
The later clean complete Hub pass below covers mapped lint; complete proof of
subsequent changes and the four-member family remains open.

## Complete Hub proof and subsequent integration

Hub `258c8467be46855a6bdadc410e24deb96f097c40`, tree
`2cfe419ae37a26f7c6fbcee1fd89ddb82f2bb674`, passed the complete required lane:
**819 distinct tests (600 Hub, 219 wire)**, all five mapped lanes, both pinned
models and custody controls. No selected test failed, skipped or retried.
GNU time was 22:41.98; maximum child RSS was 1,792,340 KiB. Its new private target
used 4,171,760,014 logical bytes. These measure this local run, not the memory
needed by parallel hosted workers. All 814 source, ten configuration, 25 tool,
177 sysroot-library and 478 browser-file subjects stayed unchanged; the checkout
was clean and the lock released. Independent review accepted the exact subject.

The next family run completed the stage 1–5 commands, then refused concurrent
Kernel source drift at the stage 5 boundary. Stages 6–7 never started.
Its 1,037.97-second GNU measurement, 400 produced
and 349 prior artifacts, and failure review remain retained. The earlier family
failure from an unsuitable temporary-directory ancestry/socket length remains
separate. Neither run establishes complete family acceptance or proves the
subsequently changed Kernel. Retry only after all four subjects and inputs freeze.

| Commit | Reviewed change | Executed evidence and limit |
| --- | --- | --- |
| `eb6c9ccdf82347f16c792d9bce6409bbc7b6814c` | Preserve the exact source timeout for each of 53 jobs / 55 matrix invocations; keep the authentic v1 publication template unchanged. | 36 publication tests, three mapped Clippy commands and 50 actual wrapper cases passed with source/tool/config guards. Initial compile and unmapped-lint failures remain retained. The 52 unavailable member jobs and five final refusals remain unavailable. |
| `912dddf210e32dd48213e44a0c5b07615acfcf18` | Select `neverhuman/bulletfarm` for new requests and reconcile root templates while preserving historical identities. | All 50 actual wrapper cases passed again against the changed configuration/templates. This is local reconstruction proof, not remote publication. |
| `8b6fb098e797bb26c73ceda0c8eb439a197dec57` | Correct aggregate clone/publication instructions and remove hardcoded green/audit badges. | Independent source review and focused links passed; complete mapped documentation proof of the final integrated subject remains required. |

External Claude provider edits were uncommitted at the earlier checkpoint;
subsequent preservation commits do not supply independent acceptance.
Independent review found post-spawn failures classified as pre-spawn, lost known
usage, missing durable failure records after collisions/write errors, diagnostic
text preventing recording, and valid transcripts reaching success despite process
failure or timeout. Later native protocol observations are valuable inputs, but
need exact source/receipt review and paired unsafe-case tests. They do not admit
the provider or lift this root's coordinator Operating HOLD.

Subsequent reviewed local packets are:

| Commit | Change | Focused evidence and remaining limit |
| --- | --- | --- |
| `5b11bfb3000a44e04ea6969a67f1dc5dc5939556` | Validate actual BulletGit source-scan completion, including ordinary executable validator files. | 36 publication tests, three mapped Clippy commands, build and 50 wrappers passed. Earlier format/mode failures remain retained. |
| `599b2ae8a6e0c958da658307a43ae1525210d6b0` | Transfer exact verifier/scanner subjects between isolated jobs. | All 50 wrapper cases plus 23 transfer controls, Bash and ShellCheck passed; exact verifier/scanner bytes and run/attempt binding are checked independently of the downloaded manifest; complete executable/runtime closure is not established by this packet. |
| `974f5de20032ba619e4495791946e44d42e94508` | Connect the generated Git source-scan job to bootstrap tool transfer and final report validation. | Fresh 36 publication tests, mapped lint/build and 50+23 controls passed. The other 51 unavailable member profiles and five final refusal gates remain; no hosted run occurred. |
| `9f53fe6af3d03e7f3a632e44f5fee07239947e3a` | Preserve Portal command identity across corrupt storage, reload and stale POST/GET responses. | Typecheck, 43 focused and all 183 unit tests passed; all 32 coverage floors and 8,384 input guards agreed. APIs were mocked; no actual browser/provider claim. |
| `29d02c348e66bd4da6bd4582a33e2932b3c72940` | Ratchet fast/coverage/meta checks to the actual 183 identities. | Existing report and metadata gates passed; 8,390 guarded inputs agreed. Complete standalone required subsequently passed; see review9c0343ca above. Family and hosted acceptance remain separate. |

The original 165 Portal identities are retained alongside 18 new regression
cases. External supporting merges that omitted workflows are not accepted as
complete source publication. The repaired canonical history retains workflows.
All 21 inventoried recording/media source and test files are tracked and carried
by unfiltered aggregate tree import; their presence on public main still requires
publication of the accepted source tree. Real four-provider recordings, the
strict 1080p GIF size/fidelity checks, and language migrations remain work.
No typed gap or product-profile status is promoted by these local packets.

## Outstanding delivery and production work

At the **11:52 UTC** read-back, GitHub `neverhuman/bulletfarm` main remained
`f8ce28e6b0583160519e5898250904f63eee753d`: zero workflows, zero runs, zero
rulesets and unprotected main. Supporting Hub PR 2 merged as `55e86b2` and
Kernel PR 2 as `6070a05`; these reconstructed subjects do not inherit proofs
bound to different canonical commits. The earlier exact audit/Kernel branch
pushes were rejected because the available OAuth token lacks `workflow` scope.
Their branches remained absent. Source/workflow history was preserved; the
repository-scoped publication App and complete aggregate CI remain required.
Fresh SSH main-ref reads succeeded for all four supporting repositories; this
establishes a transport option for preserving full review history, not permission
to discard remote-only history or evidence of a push.

The installed Jankurai remains **1.6.11**. Subsequent remote discovery found the
published 1.7.0 Linux archive and its checksum/provenance assets, plus all three
selected dependency tags. An initial tag absence was caused by a local URL
rewrite and is not evidence of remote absence. The archive was downloaded and
hashed without installation or execution. Subsequent cryptographic verification
with the selected GitHub CLI checked the GitHub OIDC/workflow identity, source,
run/attempt and archive subject, plus wrong-workflow/archive/repository negatives.
That verifies artifact origin, not the missing selected fixes, runtime behavior,
portable auditor admission or any Bullet score.

Source readback establishes a remaining admission defect: selected Core
`8505079e47597225e1f2bf65f57d41b0d050bfe7` has the common-base versions of
`audit/mod.rs` and `commands/witness.rs`, and lacks `audit/outcome.rs` and
`audit/release_proof.rs` from reviewed Core `2e0c395`. These four implementation
paths prove that those reviewed fixes are absent from the selected source;
ancestry divergence alone was not used to make that finding. A newer release
banner and successful release build do not establish the required policy,
conformance and process-exit agreement. Score-90/zero-cap/zero-hard acceptance
remains open until the source selections and auditor qualification are repaired.

Finish health and complete CI, then admit the preserved coordinator generation
and supervised 22→23 upgrade before the durable account/reservation/run/dispatch
transaction. Connect Runner supervision, termination reconciliation and atomic
Candidate finalization; reconcile browser state after lost responses and restart.
Qualify Codex, Claude and Cursor independently on xbabe2, then execute the twelve
real tasks, mixed-provider collaboration and seven-day survival observation.
The remaining custody, lifecycle, platform, evolution, forge, distributed and
post-V1 obligations stay in the full plan. All G1–G18 remain `DESIGNED`; all
18 product and two diagnostic profiles remain `BLOCKED` in the unchanged typed
inventory. The coordinator Operating HOLD remains effective. The
[xbabe2 development closeout](xbabe2-development-closeout.md) details the existing
plan's work order and requires capture/render/check source in the main aggregate.

## Retained evidence

The local September 8 health implementation directory on xbabe2 retains original
source packets, raw logs, failed attempts, independent reviews and integration
receipts, including:

- `formal-json-repair-r1/`, `formal-json-metadata-r1/` and `kernel-required-r5/`.
  Kernel required log SHA-256: `90cbb3ff540616c4b080821545e6c6e22ad09f9e515d9e7f3f6e5ac8991b47f4`.
- `jankurai-admission-delta-r2/` and the deep-audit directory's document coverage,
  GitHub readbacks and publication refusals.
- Deep-audit `recording-repair-r1/`, `hq-rendering-repair-r1/`, `continuous-portal-r1/`,
  `continuous-render-integration-r1/`, `continuous-timing-diagnosis-r1/` and
  `renderer-timing-repair-r1/`. Timing review SHA-256:
  `8b6d993f07a039b0166fec6d9378c704ecb47deb946508284ee4fcd82aa50806`.
- Deep-audit `cargo-transport-config-r1/`, `hub-media-inventory-r1/`, `media-routing-r1/`,
  external-closeout review/restoration packets, and both `mapped-docs-r1/` and
  `mapped-docs-r2/`. Successful docs log SHA-256:
  `aab8eb7f8c56d554c5f5316c46db0ba3b874bef808dbb7d1a2467ead936e2fc1`.

- `jankurai-admission-remote-r1/`, and deep-audit `private-live-receipt-r1/`,
  `private-receipt-actual-consumer-r1/` and `untracked-media-custody-transfer-r1/`.
  The durable private transfer retains both verified payloads and original files.

Hosted runs, packages and product-profile receipts require their own executed,
exact-subject admission. Local observations do not substitute for them.

Additional retained receipts include `hub-required-r1/` (required log SHA-256
`abe398c959a7c3c31147ec1b4a1c94753714ec0081bf658216a30038fc252937`),
family retry log `4750847541dcf1ffb183b58b4d87447db2c9df5f0d677e0cf6e59d909b1af5dd`,
timeout review `270ec9fb087b227a3437a3194d783b765276771730010af961943b7f3b47c142`,
destination review `d489da27235464bb57e050988d4968f8072510d274f5fa93a46af113d8bbf560`,
and Jankurai attestation review
`a496fd885eb8f7e8631afb65fb9da5a0a91977c266d8d2243c7c04c5001c8554`.
