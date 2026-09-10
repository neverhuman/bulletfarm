# Retained native-bridge recordings

These September 9 recordings document a Claude bridge attempt and a Portal
view of a bridge ledger. The requested production coding showcase remains open.
The [development closeout plan](../../docs/assurance/xbabe2-development-closeout.md)
defines the remaining transaction, provider and recording gates.

## What the retained attempt establishes

The [terminal transcript](candidate/transcript.txt) shows Claude 2.1.266,
a Kernel source snapshot, a proposal creating only `PONG.txt`, Candidate
preservation, and a cleanup refusal after the lease was released. The inspected
local run returned `RC=1`. Its receipt reports 47.997 seconds and 529,592 micro-USD;
those are local observations, not independently verified vendor billing. Every
eligibility flag is false.

This `transaction_offline` bridge seeds its own ledger and uses fixture verifier
and forge components. It does not establish the authenticated
`POST /api/v1/commands` production transaction, independent verification, human
integration, or a useful implementation task. The recorded terminal does not
independently establish all its narrated containment claims or model identity.
The inspected run had no retained raw provider capture or completed component
proof receipt.

The [Portal recording](portal-real.gif) shows SQLite-backed projections, but its
binding to this exact attempt is not established. Reading those projections does
not establish production coding admission. Its Merge Rail shows no Candidate;
the preserved artifacts still need an exact binding to the ledger projection.

## Artifact properties

| Artifact | Geometry | Bytes | Frames / duration |
| --- | --- | --- | --- |
| [Terminal, larger](candidate/dogfood-candidate-hi.gif) | 2049 × 1323 | 5,537,463 | 108 / 126.8 s |
| [Terminal, historical “1080” filename](candidate/dogfood-candidate-1080.gif) | 1903 × 1228 | 5,070,420 | 108 / 126.8 s |
| [Portal](portal-real.gif) | 1920 × 1200 | 1,199,065 | 9 / 20.8 s |

The terminal byte counts are read from the retained artifacts. All three GIFs
are below 50,000,000 bytes; none has the required 1920 × 1080 geometry.
The [original terminal event stream](candidate/session.cast) has 124 output
events and ends at 131.267519 seconds. It is not an original RGB frame master.

Palette counts measured after GIF encoding cannot prove that the original
colors survived quantization. Building FFV1 from decoded GIF frames and comparing
it with that GIF also cannot establish preservation of the original capture.
No original RGB master accompanies this terminal packet. The browser capture
was quantized and its stated comparison still reports changed pixels; a claim
of lossless original-frame preservation is unsupported.

## Tooling and remaining work

The [PTY recorder](../../scripts/lib/demo-gif-pty-record.py) and
[renderer](../../scripts/lib/demo-gif-render.py) are tracked in the Hub.
The Portal contains `ops/media/portal-capture.mjs`. Capture and rendering code
must remain in the source family and its primary aggregate.

The local wrappers `run-real-e2e.sh` and `session.sh` still need repairs before
serving as a reproducible recording command: propagate the child failure, bind
one exact run and proof directory, remove session-specific scratch defaults,
retain the native capture result, and report observed process custody. Selecting
the latest run and latest proof directory independently is insufficient.

Production recordings require a completed authenticated transaction through the
ledger, native Runner, containment, BulletGit, durable finalization, independent
verifier, and human integration. Qualify Codex, Claude and Cursor independently,
then Antigravity; show meaningful implementation and mixed-provider handoffs.
Keep the coordinator Operating HOLD until its reviewed admission checkpoint.

Capture the styled TUI and web at exactly 1920 × 1080, retain original PNG or
FFV1 masters before quantization, bind frames to the same run and source
subjects, and inspect decoded frames for sharpness, contrast and clipping.
Generate each GIF below 50,000,000 bytes and measure its difference from the
original frames. Preserve failures alongside successful evidence. These retained
files remain diagnostic observations and clear no release gate.
