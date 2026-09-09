# High-resolution demo GIFs

Operator-authenticated local recordings of the real Claude, Codex, and Cursor
TUIs plus the Portal Control Tower bootstrap form. They are not Bullet Evidence,
live-admission receipts, or release authority. Live provider admission stays
disabled.

Rebuild on a machine that already has those CLI logins:

```bash
just demo-gif-record    # live TUI + Portal form, production sessions
just demo-gif-render    # agg + FFmpeg, no credentials
just demo-gif-check     # geometry, brightness, redaction
```

`demo-gif-record` writes asciinema casts and a 1920×1080 Portal capture.
`demo-gif-render` uses [agg](https://github.com/asciinema/agg) with DejaVu Sans
Mono at 20px, a high-contrast theme, and `paletteuse=dither=none` so dark navy
does not Bayer-dither into mud. Hosted CI does not spawn providers.
