# Private native capture and rendering

This pipeline records native CLI observations and Portal screenshots on the
operator's machine. It does not establish production Bullet dispatch, account
qualification, a completed coding task or release evidence. The
[deep audit](../assurance/deep-audit-20260909.md) records those missing connections.
Historical captures remain unqualified observations, preserved in private
custody and retained source history. The current pipeline never overwrites or
automatically publishes them.

## Capture

`just demo-gif-record` invokes the installed Claude, Codex and Cursor CLIs and a
local `bullet-farmd` plus Portal preview. The fixed prompt asks for a short
explanation without tools or file edits. This is an illustration, not the
three-provider coding campaign. Run it only when those native invocations are
within the operator's current account and harness authorization. The coordinator
Operating HOLD and production admission requirements remain in effect.

The host needs `claude`, `codex`, `cursor-agent`, `curl`, `jq`, `node`, `npm` and
`python3` on PATH; the Portal must already have `dist` and Playwright installed.
`BULLET_FARMD_BIN` selects the local daemon. Nothing is downloaded by these
scripts. Installed tool paths and an existing login alone do not qualify an
account or its complete runtime dependency closure.

Set `BULLET_DEMO_GIF_CACHE` to an absolute, canonical directory outside the
family checkout, owned by the operator with mode `0700`. Each invocation creates
a fresh `run-*` directory and fresh daemon data. It retains every attempted CLI
capture, exit status, raw PTY bytes, transcript and result record. Ordinary
failure may use a separate fallback attempt; interruption aborts the campaign.
`native_capture_child_started` means a capture child was forked; successful
provider execution remains `UNVERIFIED`.

The Portal tour samples 1920 × 1080 PNGs serially after initial DOM content is
loaded and between eight correlated landmarks. It requests a 4 Hz ceiling and
retains actual capture/write timing, missed cadence, frame gaps and checkpoints;
it does not interpolate frames. Bounds are 64 frames, 15 seconds, 32 MiB of PNGs
and 8 MiB per PNG. Capture stops and retains its failure when a limit or screenshot
operation fails. It drains any active capture/write before closing the browser.

The bootstrap input is masked. Correlated HTTP and command observations are
retained without storing response credentials. Loopback listener identity and
product completion remain `UNVERIFIED`. Actual Chromium tests use synthetic pages
and responses; production Portal/account acceptance is separate.

The renderer validates native master timestamps against the recorded capture
and encodes GIF absolute timestamps with centisecond rounding. It checks the
actual GIF frame delays independently; encoded intervals shorter than two
centiseconds, or colliding after rounding, refuse. The final display hold repeats the last encoded
gap, or uses 100 milliseconds for a single frame. That hold is a display policy,
not an additional observed capture duration. The [health checkpoint](../assurance/health-checkpoint-20260909.md)
retains the original timing failure and the independently verified correction.

## Render one selected capture

Use the `portal` directory containing `observation.json`, or one successful CLI
attempt directory containing `session.cast.result.json`. Select explicit,
absolute, canonical local Python and FFmpeg executable files and their SHA-256
hashes. Symlink paths are refused; resolve the selected executable before
reviewing its hash. Terminal rendering additionally requires a canonical `agg`
executable file and its hash.
Select the hash of `scripts/lib/demo-gif-render.py` as the implementation subject.
These hashes identify selected files; they do not admit a portable tool closure.

The following variables are operator-selected absolute paths and reviewed
SHA-256 values. `PRIVATE_PARENT` must already be a private `0700` directory
outside the family; each output directory must be new.

```bash
just demo-gif-render "$PYTHON" "$PYTHON_SHA256" "$RENDERER_SHA256" \
  --kind portal --input "$CAPTURE" --input-sha256 "$CAPTURE_SHA256" \
  --output "$PRIVATE_PARENT/render-001" \
  --ffmpeg "$FFMPEG" --ffmpeg-sha256 "$FFMPEG_SHA256"
```

For a Portal input, `CAPTURE_SHA256` hashes `observation.json`. For a terminal
input it hashes `session.cast.result.json`; use `--kind terminal` and add
`--agg "$AGG" --agg-sha256 "$AGG_SHA256"`.

| Output | What is actually preserved or checked |
| --- | --- |
| `source/` | Exact capture input files, including original timestamps and private raw bytes. |
| `master.nut` (Portal) | FFV1 master whose decoded RGB hashes match every original PNG. |
| `derivative.gif` | Portal: original geometry, no fades, scaling, interpolation or dithering; decoded pixels labeled `EXACT` or `QUANTIZED`. Terminal: measured font/theme rendering of the retained cast, with no original-pixel or no-dither guarantee. |
| `manifest.json` | Source/tool hashes, actual commands, decoded pixels and timing, and explicit qualification limits. |

GIF has a limited color palette and centisecond timing. Disabling dithering
does not make conversion lossless. Add `--strict-lossless` to refuse a Portal
GIF whose decoded pixels differ; the original PNGs and exact-color master are
retained even on refusal. Original timestamp JSON is preserved separately from
rebased master timestamps and GIF timing. The terminal renderer uses a bright
`github-light` theme, but a cast has no original RGB master to compare against;
strict lossless mode therefore refuses terminal input. Observed host fonts are
recorded without claiming a complete font closure; strict font closure currently
refuses. Bounds and failures are retained in private output, never promoted to
successful media.

## Independently check the selected generation

Set `MANIFEST_SHA256` to the reviewed hash of the completed render manifest.
Use the same selected implementation and tool subjects. For terminal output,
use `--kind terminal` and include the same `agg` arguments.

```bash
just demo-gif-check "$PYTHON" "$PYTHON_SHA256" "$RENDERER_SHA256" \
  --kind portal --input "$PRIVATE_PARENT/render-001" \
  --input-sha256 "$MANIFEST_SHA256" --output "$PRIVATE_PARENT/check-001" \
  --ffmpeg "$FFMPEG" --ffmpeg-sha256 "$FFMPEG_SHA256"
```

The checker re-reads hashes and decodes pixels and timing. A successful check
proves that limited artifact relationship. It does not certify redaction,
accessibility, authentication or public export. Review private captures before
selecting anything to share: raw PTY files can contain credentials or private
content. The scripts do not copy output into the repository.

The separate `readme-live-*` commands normalize explicitly supplied data and
rebuild historical offline derivatives. They do not invoke providers. Required
CI must use synthetic or reviewed credential-free inputs; real subscription
campaigns need their separately admitted execution environment and receipts.
