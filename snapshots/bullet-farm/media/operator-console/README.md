# Retained xbabe2 console diagnostics

These GIFs retain local UI observations. Both manifests declare provider `none`
(version `0.0.0`), `candidate_id: null` and `release_eligible: false`. They do not
show real provider coding, an independently verified Candidate or approved
integration. Operating HOLD remains.

| Capture | Recorded time (UTC) | Source and capture limits |
| --- | --- | --- |
| [TUI](operator-tui.gif) · [manifest](operator-tui.manifest.json) | 2026-09-11 04:42:30 | Kernel `53f6d96c` marked dirty; Portal `d00c88ae`. Native raster 1913×1078 from a 225×54 terminal, padded to a 1920×1080 canvas. |
| [Portal](operator-portal.gif) · [manifest](operator-portal.manifest.json) | 2026-09-10 20:17:01 | Portal `00b13bfa` marked dirty; Kernel `1d477119`. Four frames at 1920×1080. |

These are separate captures with different source and binary subjects. They
cannot establish the same operator, farmd invocation, campaign interval or
durable task in both interfaces. Their command, Attempt and receipt fields are
generated diagnostic labels, not Kernel-issued records or persistence receipts.
The TUI's padded canvas does not satisfy native 1920×1080 capture.

Retained UI states include CONNECTING, HOLD/UNBOUND, blocked Head Send and an
unknown release decision. Stage-one VHS component tapes under
[`docs/readme-media/`](../../docs/readme-media/) have a separate check lane.
A media-checker success alone cannot qualify the contents, provenance or timing
of these recordings; no full recording acceptance is claimed here.

The [existing xbabe2 work order](../../docs/assurance/xbabe2-development-closeout.md#real-1080p-capture-as-maintained-source)
requires uninterrupted installed TUI/Portal recordings of the same real task
through authenticated provider execution, Candidate preservation, independent
verification and approved integration read-back. Original frames, timestamps,
lossless masters and accessible transcripts must support synchronized contiguous
excerpts, each native 1920×1080 and strictly below 50,000,000 bytes. Those production
recordings, acceptance and approved publication/fetch-back remain outstanding.
