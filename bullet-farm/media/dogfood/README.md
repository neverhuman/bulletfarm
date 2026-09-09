# The dogfood screencast

What the GIF in `docs/readme-media/dogfood-candidate/` shows is a real run. The
provider turn inside it is a real, billed `claude` turn against a real account,
executed while the recorder was running. Nothing is replayed, re-enacted, or
reconstructed after the fact.

## What is in the recording

1. The enrolled provider runtime, and the digest the enrollment pins it to.
2. What the containment removes: network, host environment, writable tree.
3. The bridge running live, on a snapshot of Bullet Farm's own kernel source,
   with an elapsed clock and the contained process count visible throughout.
4. The Runner's journal from that run: the turn, the applied patch, the gate,
   the prepared Candidate, the preserved Candidate.
5. The Candidate itself, opened from the bundle that run preserved.
6. The receipt, with every eligibility flag false.

The recording also shows a real open defect rather than editing around it: the
Runner releases its lease before `bullet-gitd` cleans the workspace, and gitd's
cleanup re-reads that lease online, so a completed attempt ends on
`AUTHORITY_REFUSED`. The Candidate is already prepared and preserved by then.
It reproduces on the simulator too.

## Reproducing it

    media/dogfood/run-real-e2e.sh          # the real end-to-end run, alone
    media/dogfood/session.sh               # the narrated version, for recording

`run-real-e2e.sh` needs an operator-staged provider runtime, a dogfood policy,
binding and enrollment, and a credential grant. It refuses by name if any of
them is absent; it never falls back to the simulator.

To record and render:

    python3 scripts/lib/demo-gif-pty-record.py \
      --cast <dir>/session.cast --transcript <dir>/transcript.txt \
      --cols 120 --rows 34 --max-seconds 600 \
      -- bash media/dogfood/session.sh

    agg --theme "$THEME" --font-size 28 --line-height 1.35 \
        --fps-cap 10 --idle-time-limit 1.5 --renderer resvg \
        <dir>/session.cast dogfood-candidate-hi.gif

`THEME` is a pure-black background with the bright ANSI set, chosen so nothing
is dimmed:

    000000,ffffff,1c1c1c,ff5f5f,5fff5f,ffff5f,5fafff,ff5fff,5fffff,f0f0f0,
    808080,ff8787,87ff87,ffff87,87d7ff,ff87ff,87ffff,ffffff

The 1080-class variant is rendered natively at `--font-size 26` rather than
downscaled, so it carries no resampling loss.

## Quality, as measured rather than asserted

| Property | Measured |
| --- | --- |
| High-resolution GIF | 2049 x 1323, 5.3 MB |
| 1080-class GIF | 1903 x 1228, 4.8 MB |
| Frames | 108, 126.8 s |
| Colours per frame | 40 min, 255 max |
| Frames at the 256-colour ceiling | 0 |

No frame reaches the GIF palette ceiling, so no colour was dropped to fit it.
Decoding the GIF and decoding a lossless FFV1 master built from its frames give
the same pixels, `sha256 57486d88f0b0b33fae30631739d0ac48fc161316`, so the
distributed artifact carries the rendered frames exactly.

## What this is not

This is a `DOGFOOD_RUN`: an operational observation. It clears no release gate,
satisfies no self-hosting claim, and every eligibility flag in its receipt is
false. The recording says so on screen rather than in a footnote.
