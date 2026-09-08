# Operator-authenticated README media

These assets are unsigned local observations. They are not Bullet Evidence,
live-admission receipts, install receipts, `TRANSACTION_PROOF`, or release
authority.

`just readme-live-record` requires already-authenticated Claude, Codex, and
Cursor CLI sessions plus a loopback `bullet-farmd` and Portal preview. It
records those operator credentials in place; it never writes keys, enrollments,
or operator-decision lines, and it does not enable `live_admission`.

`just readme-live-render` rebuilds the GIFs from the committed transcripts and
Portal frames with the same pinned VHS/FFmpeg image used by stage-one media.
Hosted CI runs `just readme-live-check` only: it re-renders and hash-checks the
committed artifacts and does not spawn a provider.

Regenerate before committing media changes:

```bash
just readme-live-record
just readme-live-render
just readme-live-check
```
