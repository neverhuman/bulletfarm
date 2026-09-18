# Real recordings: `record-tui.sh` and `readme-real-check.sh`

Two stages guard the media in this repository, and they admit different things.

**Stage one — `scripts/readme-check.sh`** admits only synthetic VHS media:
exactly 1200×675, `r_frame_rate` 12/1, at most 3 MiB, and a byte-identical VHS
re-render. Every frame is reproducible from a tape, so the gate can demand the
bytes back. A recording of a real session can never satisfy it, and lowering
stage one so it could would destroy the property that makes stage one useful.

**Stage two — `scripts/readme-real-check.sh`** exists for the other case: a real
capture, whose bytes are not reproducible, published at exactly 1920×1080 with a
manifest that says what it is, what produced it, which run it belongs to, and —
just as importantly — what it does not claim.

`scripts/media/record-tui.sh` is the recorder that produces media stage two
accepts. It replaces the local wrappers `media/dogfood/run-real-e2e.sh` and
`media/dogfood/session.sh`, which `media/dogfood/README.md` lists as needing
repair before they could serve as a reproducible recording command.

Neither script establishes product completion, provider qualification, a
completed transaction, independent verification, or release authority.

---

## `record-tui.sh`

```
record-tui.sh --out-dir DIR --narration SCRIPT --bullet-bin PATH --farmd-bin PATH
              [--cols N] [--rows N] [--run-id ID] [--max-seconds N]
record-tui.sh --self-test --out-dir DIR [--cols N] [--rows N] [--run-id ID]
```

| Argument | Meaning |
| --- | --- |
| `--out-dir` | Absolute, must not exist yet, created `0700`. It must sit under `$HOME`, never inside the source family (`/home/ubuntu/bullet`, derived from the script's own location) and never under `/tmp`. The tracked renderer refuses to write inside the family, and the capture and render private-path checks refuse world-writable ancestors. |
| `--narration` | Absolute path to an executable script run **inside** the PTY. It writes `run.json` (below). |
| `--cols` / `--rows` | Terminal grid, default 225×54 (see [Geometry](#geometry)). Capped at 240×100 because that is the grid the tracked renderer and the stage-two gate admit. |
| `--bullet-bin` / `--farmd-bin` | The exact binaries the recorded session runs. Their sha256s go in the manifest. Required for a real recording; forbidden for `--self-test`, which runs neither. |
| `--run-id` | Optional. Also names the published artifacts; otherwise the name is `tui-<UTC stamp>`. |
| `--max-seconds` | PTY capture-loop limit, 1..600 (default 120). |
| `--self-test` | Record a harmless synthetic narration (~3 s) instead, render it, and prove the whole path end to end. |

Refusals are typed, one line, on stderr: `RECORD_TUI_<CODE>: <the value rejected>`.

### What it does, in order

1. Validates every argument, then creates the private output tree.
2. Records the narration through `scripts/lib/demo-gif-pty-record.py`
   (`openpty` + `TIOCSWINSZ` + `execvpe`, asciicast output).
3. **Propagates the narration's exit status.** A small in-PTY launcher runs the
   narration, writes its exact status to `capture/producer.exit`, and exits with
   it; the recorder reports it and `record-tui.sh` exits with the same number
   without rendering or publishing anything. `media/dogfood/session.sh`
   discarded that status.
4. Reads and validates `run.json`.
5. Renders through `scripts/demo-gif-render.sh` with the three positional
   arguments (`PYTHON PYTHON_SHA256 IMPLEMENTATION_SHA256`) computed from the
   selected interpreter and the tracked implementation, plus the selected
   `agg` and `ffmpeg` with their sha256s.
6. Centres the render on a 1920×1080 canvas (see [Geometry](#geometry)).
7. Writes `<name>.manifest.json`.
8. Runs `scripts/readme-real-check.sh` over the result and only then reports
   success.

### Output tree

```
<out-dir>/
  <name>.gif                      published, exactly 1920x1080
  <name>.manifest.json            bullet.real-media.v1
  <name>.cast                     the retained master (copy of capture/session.cast)
  run.json                        written by the narration
  narration-exit.txt              the narration's exit status
  capture/                        exactly what the tracked renderer admits:
    session.cast                  asciicast v2 event stream
    session.cast.raw              the child's exact terminal byte stream
    transcript.txt                ANSI-stripped, display-redacted
    session.cast.result.json      native capture result (bullet.native-capture.v1)
    producer.exit                 the narration's exit status
  render/                         the tracked render generation, left untouched
    derivative.gif  manifest.json  source/  logs/
  run/                            the ONE run directory, exported to the narration
  proof/                          the ONE proof directory, exported to the narration
  work/                           launcher, generated narration, render log, scratch
```

`render/` is never modified after the renderer writes it, so
`scripts/demo-gif-check.sh` can still re-verify that generation later. Scratch
files go in `work/`.

A PTY merges stdout and stderr into one stream by construction, so
`session.cast.raw` is the child's combined output exactly as the terminal saw
it; there is no honest way for a terminal recording to separate the two, and
this script does not pretend to. `producer.exit` carries the status.

### Environment handed to the narration

The narration is given **one** run directory and **one** proof directory. It
must use exactly those. Nothing anywhere selects "the latest" of anything —
`media/dogfood/session.sh` chose its run directory and its proof directory with
two independent `ls -dt | head -1` calls, which is how a recording ends up
narrating a different run than the one it just watched.

| Variable | Value |
| --- | --- |
| `BULLET_RECORD_OUT_DIR` | the output directory |
| `BULLET_RECORD_RUN_DIR` | the one bound run directory |
| `BULLET_RECORD_PROOF_DIR` | the one bound proof directory |
| `BULLET_RECORD_RUN_JSON` | where to write `run.json` |
| `BULLET_RECORD_RUN_ID` | the recording's name |
| `BULLET_RECORD_COLS` / `_ROWS` | the PTY grid |
| `BULLET_RECORD_BULLET_BIN` / `_FARMD_BIN` | the binaries named on the command line |
| `BULLET_RECORD_CAPTURE_DIR` | used by the launcher; the narration must not write there |

The narration must not write into `capture/`: the tracked renderer refuses any
capture directory whose inventory is not exactly the five files above.

`BULLET_RECORD_FFMPEG` and `BULLET_RECORD_AGG` override tool selection.
`scripts/lib/demo-gif-render.py` hashes each tool by reading it whole under a
64 MiB cap, so a large static `ffmpeg` build cannot be admitted at all; the
recorder therefore picks the first `ffmpeg` on `PATH` small enough for the
tracked renderer to read, and refuses with `RECORD_TUI_TOOL_UNAVAILABLE` when
there is none.

### Credentials

The recorder never reads, prints or copies a credential. It exports no secret to
the narration. The manifest's `account` is the literal string `REDACTED`, and
the gate refuses a manifest containing an email address anywhere.

The redaction screen also refuses `/home/`, `/Users/`, `/tmp/`, `Authorization:`,
`Bearer `, private-key headers, `sk-`, `ghp_`/`gho_`/`ghu_`/`ghs_`/`ghr_`,
`github_pat_`, `AKIA`/`ASIA`, `xox[baprs]-` and any email **inside the cast**,
plus `boot_`/`csrf_`/`wrk_` tokens and any 64-hex value the manifest does not
declare. A real narration must therefore print redacted paths and must not echo
a nonce. `gitleaks` then scans private copies of the manifest and the cast, so a
`.gitleaks.toml` or `.gitleaksignore` planted in the media directory cannot
influence the result.

---

## The `run.json` contract

The narration writes this file to `$BULLET_RECORD_RUN_JSON` before it exits. It
is the only channel through which run identity, receipt and provider facts reach
the manifest — `record-tui.sh` never invents them.

```json
{
  "schema": "bullet.record-run.v1",
  "run_id": "<the value of BULLET_RECORD_RUN_ID>",
  "command_id": "cmd_<64 hex>",
  "attempt_id": "atm_<64 hex>",
  "candidate_id": "can_<64 hex>",
  "receipt": { "algorithm": "blake3", "digest": "<64 hex>" },
  "provider": { "name": "claude", "runtime_version": "2.1.266" },
  "claims": ["local-observation", "no-gate-cleared", "real-recording"]
}
```

* `candidate_id` is `null` when the run preserved no Candidate.
* `receipt.algorithm` is `blake3` or `sha256`; `receipt.digest` is the receipt
  digest of the run being recorded.
* `provider.runtime_version` is the **enrolled** runtime version, not whatever
  happens to be on `PATH`.
* `claims` is checked against the allowlist below. `--self-test` must declare
  exactly `["self-test"]`; a real recording must not use that claim at all.
* Anything malformed is refused as `RECORD_TUI_RUN_JSON: <the value rejected>`
  before the render starts.

---

## The manifest: `bullet.real-media.v1`

`readme-real-check.sh` enforces the key set exactly — a missing field and an
unexpected field are both refused.

| Field | Rule |
| --- | --- |
| `schema` | `bullet.real-media.v1` |
| `kind` | `terminal` or `portal` |
| `name` | matches the file basenames |
| `recorded_at` | RFC 3339 UTC, `YYYY-MM-DDThh:mm:ssZ`, and a real date |
| `host` | short hostname |
| `recorder` | `{path: "scripts/…", sha256}` — the recording script, hashed |
| `render_tools` | `agg`, `ffmpeg`, `python`, each `{sha256, version}` (`agg` may be `null` for `portal`) |
| `binaries` | `{bullet: {sha256}, "bullet-farmd": {sha256}}`, or `null` **iff** the only claim is `self-test` |
| `members` | all four repositories — `bullet-farm`, `bullet-git`, `bullet-kernel`, `bullet-portal` — each `{oid: <40 hex>, dirty: <bool>}` |
| `run` | `{command_id: "cmd_…", attempt_id: "atm_…", candidate_id: "can_…" or null}` |
| `receipt` | `{algorithm: blake3\|sha256, digest: <64 hex>}` |
| `provider` | `{name, runtime_version}` |
| `account` | the literal `REDACTED`; no email may appear anywhere in the manifest |
| `master` | `terminal`: `{cast_sha256}`. `portal`: `{frames_framemd5_sha256, ffv1_sha256}` |
| `gif` | `{sha256, bytes}`, both re-derived from the file |
| `canvas` | see [Geometry](#geometry) |
| `capture_sha256` | `session.cast.result.json` (terminal) or `observation.json` (portal) |
| `render_receipt_sha256` | the `manifest.json` the tracked renderer wrote |
| `terminal` | `{cols, rows, font_family, font_size, line_height, theme, renderer}` |
| `portal` | `{frames}`, cross-checked against the GIF frame count |
| `claims` | allowlist, below |
| `release_eligible` | must be `false` |

Checks that are re-derived rather than trusted: GIF geometry via `ffprobe`
**and** the GIF header; `gif.sha256`/`gif.bytes`; size strictly < 50,000,000 bytes; native
GIF delays (each ≥ 2 cs) against decoded PTS (strictly increasing, same count);
total duration in (0, 900] s; the retained master's sha256; the cast's UTF-8,
event shape, monotone timing and grid against `terminal.cols`/`rows`; the canvas
geometry against the GIF's own image descriptors; and the redaction and secret
scans.

Because the exact font resolution is not pinned here, `terminal.font_family` is
recorded as `agg-default-chain`; the observed font inventory — every font file
with its sha256 — lives in the render receipt bound by `render_receipt_sha256`.

---

## Geometry

`agg` lays out a terminal image as

```
width  = round((cols + 2) * font_size * advance_ratio)
height = round((rows + 1) * font_size * line_height)
```

— two extra advance widths of horizontal padding and one extra line box of
vertical padding. Measured on this host at `--renderer fontdue`:
`advance_ratio ≈ 0.602051` (DejaVu Sans Mono, 1233/2048), reproduced exactly at
80×24, 81×24, 120×34, 160×40 and 228×56, and the height formula reproduced at
rows 20, 30, 40, 50, 54, 60 and 78.

`scripts/lib/demo-gif-render.py` invokes `agg` **without** `--font-size`,
`--line-height`, `--font-family` or `--fps-cap`, so a recording does not get to
choose them: they are `agg` 1.5.0's defaults, font size 14 and line height 1.4.
That fixes the vertical cell box at 19.6 px, and

> `(rows + 1) * 7 * font_size / 5` must land in [1079.5, 1080.5) for the render
> to be 1080 px tall, i.e. `(rows + 1) * font_size` must be an integer in
> [771.07, 771.79). There is none.

so **no integer grid renders exactly 1080 px tall at line height 1.4, at any
font size.** (The width is no kinder: it would need `(cols + 2) * font_size` to
be exactly 3189 = 3 × 1063, which needs more than 240 columns.) The existing
recording in `media/dogfood/candidate/` is 2049×1323 for this reason, and the
one named `…-1080.gif` is 1903×1228.

The recorder therefore renders the largest grid that **fits** and then centres it
on the target canvas:

* default 225×54 → `round(227 × 8.428711) = 1913` by `round(55 × 19.6) = 1078`
* centred at `+3+1` on 1920×1080

The centring is **not** a re-encode. `canvas.method` is
`GIF_LOGICAL_SCREEN_EXPANSION`: the GIF's logical screen size, its background
colour index, the first global colour table entry and each frame's left/top
offset are rewritten in place. No pixel, palette entry or frame delay is
touched, which the recorder proves by requiring every image descriptor to carry
its own local colour table before it repaints the global entry — otherwise the
rewrite is refused. The border colour is sampled from the render's own top-left
pixel, which is `agg`'s padding, so the border is the terminal background.

`canvas` records the whole operation and the gate re-derives it:

```json
"canvas": {
  "method": "GIF_LOGICAL_SCREEN_EXPANSION",
  "native_width": 1913, "native_height": 1078,
  "offset_x": 3, "offset_y": 1,
  "background_rgb": "eceff4"
}
```

* `offset_x`/`offset_y` must centre `native_*` in 1920×1080.
* `method` is `NONE` **iff** the native render was already 1920×1080, and
  `background_rgb` is `null` **iff** `method` is `NONE`.
* The GIF's **first image descriptor** must be exactly the declared native
  rectangle, and every other frame must lie inside it — so a manifest cannot
  claim a native geometry the file does not have (`REAL_MEDIA_CANVAS`).
* `round((rows + 1) × font_size × line_height)` must equal `native_height`, and
  `native_width / ((cols + 2) × font_size)` must be a plausible monospace
  advance (`REAL_MEDIA_GEOMETRY_MATH`).

---

## Honesty rules

### Claims allowlist

Exactly these strings, and nothing else:

`self-test`, `real-recording`, `real-provider-turn`, `local-observation`,
`transaction-offline-bridge`, `candidate-preserved`, `known-defects-shown`,
`no-gate-cleared`.

* `local-observation` and `no-gate-cleared` are **mandatory** on every recording
  that is not a self-test.
* `self-test` may not appear alongside any other claim.
* No claim may contain `TRANSACTION_PROOF`, `independent`, `LIVE_PROOF`,
  `release` or `self-hosted-v1`, in any case. A recording is a local
  observation; it is not proof, it is not independent, and it clears no gate.
* `release_eligible` is `false` and the gate refuses any other value.

### What a recording may say

It may say that these bytes were captured on this host at this time, from this
run identity, with this provider runtime and these member commits, and that the
capture, the render receipt and the master are retained and hash to the recorded
values. It may show a defect.

### What a recording may not say

It may not say that a production authenticated transaction completed, that any
verification was independent, that a provider is qualified, that a gate is
cleared, that anything is release-eligible, or that the shown numbers are
vendor-verified billing. It may not present an offline bridge as the production
path, and it may not claim lossless preservation of original terminal RGB: a
terminal capture has no original RGB master, only an exact event stream, and the
render receipt says so.

### Caption template

Use this next to any published recording, filled in from the manifest:

> Local observation recorded on `<host>` at `<recorded_at>`, run `<command_id>`
> / attempt `<attempt_id>`, provider `<provider.name> <provider.runtime_version>`
> on an enrolled runtime. 1920×1080, `<frames>` frames, `<duration>` s,
> `<bytes>` bytes; the asciicast master and the render receipt are retained.
> Claims: `<claims>`. This clears no gate, was not independently verified, and
> is not release evidence.

---

## Proving both scripts

```sh
bash scripts/readme-real-check.sh --self-test
bash scripts/media/record-tui.sh --self-test --out-dir "$HOME/bullet-media-selftest-$(date -u +%Y%m%dT%H%M%SZ)"
shellcheck -x scripts/readme-real-check.sh scripts/media/record-tui.sh
```

`readme-real-check.sh --self-test` builds a synthetic 1920×1080 two-frame GIF
with `ffmpeg`, plus a valid manifest and cast, in a private temporary directory;
proves it passes; then mutates one thing at a time — wrong geometry, oversize,
missing field, unexpected field, wrong schema, wrong kind, name mismatch,
non-RFC-3339 timestamp, malformed run identity, unsupported receipt digest,
non-numeric runtime version, an email as the account label, a missing member
repository, a non-OID member commit, `release_eligible: true`, a forbidden
claim, an unknown claim, missing mandatory claims, a mixed self-test claim,
mis-declared binaries, mis-declared canvas geometry, a grid that does not fit
the canvas, an implausible cell advance, GIF hash drift, master hash drift, a
missing master, a symlinked GIF, a cast/manifest grid mismatch, `boot_` and
`csrf_` tokens, an undeclared 64-hex nonce, an email and a home path in the
cast, a `gitleaks`-only finding, and a zero frame delay — and proves that each
one produces its own typed refusal.

`record-tui.sh --self-test` records a ~3 s synthetic narration that calls no
provider and spends nothing, renders it through the tracked renderer, centres it
on 1920×1080, writes the manifest with `claims: ["self-test"]` and
`binaries: null`, and passes it through `readme-real-check.sh`.

To watch the gate refuse a real file, point it at the retained recording in
`media/dogfood/candidate/` with a hand-written manifest: it is 2049×1323, so a
manifest claiming the required geometry is refused with
`REAL_MEDIA_GEOMETRY: 2049x1323`, and a manifest declaring the true native size
is refused with `REAL_MEDIA_CANVAS: canvas.native_width=2049`. That recording
cannot pass stage two and re-recording it is the only fix.
