# Real xbabe2 operator console (not replayable)

These GIFs are stage-two real recordings of an authenticated loopback farmd
on the xbabe2 development host. They are **not** a trusted installer, not
VERIFIED, and not a Claude replacement. Operating HOLD remains. `release_eligible`
is false.

They are not the stage-one VHS component tapes. Those stay under
`docs/readme-media/` and still pass `just readme-check`.

| Artifact | What the tape shows |
| --- | --- |
| `operator-tui.gif` | `bullet tui` first paint CONNECTING then HOLD / LIVE n / UNBOUND / `HEAD_RUNTIME_BINDING_REQUIRED` / `STOP_UNIMPLEMENTED`, Portal-titled jump list, empty-fleet honesty, help, detach |
| `operator-portal.gif` | Same farmd and operator: Shift Brief (`RELEASE DECISION: unknown`), Control Tower HOLD / LIVE n, Head chip honesty. Send is omitted |

Geometry is 1920×1080. The TUI native grid is 225×54 (about 1913×1078) expanded
by `GIF_LOGICAL_SCREEN_EXPANSION`. Account in each manifest is `REDACTED`.

Validate with `scripts/readme-real-check.sh` against this directory. A passing
check is a local observation, not a gate clearance.
