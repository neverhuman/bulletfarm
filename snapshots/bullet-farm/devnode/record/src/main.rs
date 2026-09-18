//! Records the real console at exactly 1920x1080, with its own frame masters.
//!
//! Why this exists rather than another wrapper around the existing renderer:
//! every media artifact committed to this repository misses the geometry it
//! declares. The two terminal GIFs are 2049x1323 and 1903x1228, the second of
//! those despite being named "1080", and the Portal capture is 1920x1200. The
//! `bullet.real-media.v1` manifest declares 1920x1080 and nothing has ever
//! produced it.
//!
//! The arithmetic is not hard, it was simply never done. Tuiwright renders a
//! screen as `cols * cell_width + 2 * padding` by `rows * cell_height + 2 *
//! padding`, and draws real JetBrains Mono scaled to the cell height, so a
//! larger cell means larger glyphs rather than gaps. 120 columns by 36 rows at a
//! 16x30 cell with no padding is exactly 1920x1080, and 120x36 is a terminal
//! size a person would actually use.
//!
//! Frames come from the pseudo-terminal's own parsed screen model, so they are
//! native frames. Every frame is also written out as a lossless PNG, because the
//! M1 gate asks for original masters and `media/dogfood/README.md` states
//! plainly that the current packet has none.
//!
//! This records. It establishes no product completion, no provider
//! qualification and no release authority.

use std::{
    collections::BTreeMap,
    io::Write as _,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context as _, bail};
use sha2::{Digest as _, Sha256};
use tuiwright::{
    Key, Page, RenderOptions, SpawnConfig, TerminalRenderer, Theme,
    record::{GifOptions, GifRecorder},
};

/// 120 x 36 at a 16 x 30 cell with no padding is exactly 1920 x 1080.
const COLS: u16 = 120;
const ROWS: u16 = 36;
const CELL_W: u32 = 16;
const CELL_H: u32 = 30;
const PADDING: u32 = 0;
const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

const WAIT: Duration = Duration::from_secs(20);

/// One recorded beat: what was pressed, what had to be on screen afterwards.
struct Beat {
    name: &'static str,
    keys: &'static [Key],
    expect: &'static [&'static str],
}

/// Open the palette, walk down `n` entries, and commit.
///
/// This is the only way to change view. The console binds no digit keys, so
/// reaching the sixth view is Ctrl+K then five presses of `j` then Enter, every
/// time. That is worth fixing in the console; until it is, the recorder has to
/// drive it the way it actually works rather than the way it should.
const fn palette(name: &'static str, expect: &'static [&'static str], keys: &'static [Key]) -> Beat {
    Beat { name, keys, expect }
}

/// The beats that are true against an empty ledger.
///
/// The seven beats that show a mission, a task, a live attempt, a filtered list,
/// a preserved Candidate and a live audit event are deliberately absent. They
/// need durable work in the ledger, and inventing it would make the recording a
/// mock. They belong here the moment `bullet coding submit` lands one.
const BEATS: &[Beat] = &[
    Beat {
        name: "first-paint",
        keys: &[],
        expect: &["BULLET", "Operating HOLD", "Missions", "? help"],
    },
    Beat {
        name: "palette-open",
        keys: &[Key::Ctrl('k')],
        expect: &["Missions", "Attempts"],
    },
    palette(
        "tasks",
        &["Tasks"],
        &[Key::Char('j'), Key::Enter],
    ),
    palette(
        "attempts",
        &["Attempts"],
        &[Key::Ctrl('k'), Key::Char('j'), Key::Char('j'), Key::Enter],
    ),
    palette(
        "review",
        &["Candidates"],
        &[
            Key::Ctrl('k'),
            Key::Char('j'),
            Key::Char('j'),
            Key::Char('j'),
            Key::Enter,
        ],
    ),
    palette(
        "events",
        &["audit events"],
        &[
            Key::Ctrl('k'),
            Key::Char('j'),
            Key::Char('j'),
            Key::Char('j'),
            Key::Char('j'),
            Key::Enter,
        ],
    ),
    palette(
        "context",
        &["Context"],
        &[
            Key::Ctrl('k'),
            Key::Char('j'),
            Key::Char('j'),
            Key::Char('j'),
            Key::Char('j'),
            Key::Char('j'),
            Key::Enter,
        ],
    ),
    Beat {
        name: "help",
        keys: &[Key::Char('?')],
        expect: &["help"],
    },
    Beat {
        name: "help-closed",
        keys: &[Key::Char('?')],
        expect: &["Operating HOLD"],
    },
];

fn env_or(name: &str, fallback: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| fallback.to_owned())
}

fn sha256_file(path: &Path) -> anyhow::Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(&bytes)))
}

fn main() -> anyhow::Result<()> {
    // A hosted runner holds no operator session and no provider credentials, so
    // a hosted copy of this would look like proof while proving something
    // weaker. Refuse before touching anything, by name.
    if std::env::var_os("GITHUB_ACTIONS").is_some() {
        bail!(
            "DEVNODE_RECORD_IS_LOCAL_ONLY: this recorder drives a real console against an \
             authenticated session, which exists only on the development host"
        );
    }

    let bullet = PathBuf::from(env_or("BULLET_BIN", "/home/ubuntu/.local/bin/bullet"));
    if !bullet.is_file() {
        bail!(
            "DEVNODE_RECORD_BINARY_MISSING: no bullet at {}; remedy: run ./bootstrap.sh, or set \
             BULLET_BIN",
            bullet.display()
        );
    }
    let bind = env_or("BULLET_DEVNODE_BIND", "127.0.0.1:7420");
    let out = PathBuf::from(env_or(
        "BULLET_RECORD_OUT",
        "/home/ubuntu/.bullet-record/tui",
    ));
    let frames_dir = out.join("frames");
    std::fs::create_dir_all(&frames_dir)
        .with_context(|| format!("creating {}", frames_dir.display()))?;

    // The daemon's own health body is a subject of the recording, not decoration:
    // it names the exact Portal bundle the same daemon serves.
    let health = std::process::Command::new("curl")
        .args(["--fail", "--silent", "--show-error", "--max-time", "5"])
        .arg(format!("http://{bind}/health"))
        .output()
        .context("running curl for /health")?;
    if !health.status.success() {
        bail!(
            "DEVNODE_RECORD_DAEMON_UNREACHABLE: nothing answered http://{bind}/health; remedy: \
             start the daemon first, then rerun"
        );
    }
    let health_body = String::from_utf8_lossy(&health.stdout).trim().to_owned();

    let renderer = TerminalRenderer::new(
        RenderOptions {
            cell_width: CELL_W,
            cell_height: CELL_H,
            draw_cursor: true,
            padding: PADDING,
        },
        Theme::xterm_dark(),
    );
    let (w, h) = renderer.image_size(COLS, ROWS);
    if (w, h) != (WIDTH, HEIGHT) {
        bail!("DEVNODE_RECORD_GEOMETRY_INVALID: {COLS}x{ROWS} renders {w}x{h}, not {WIDTH}x{HEIGHT}");
    }

    let page = Page::spawn(
        SpawnConfig::new(bullet.to_string_lossy().into_owned())
            .arg("tui")
            .size(COLS, ROWS)
            .env("TERM", "xterm-256color")
            .env("COLORTERM", "truecolor")
            .env(
                "XDG_STATE_HOME",
                env_or("XDG_STATE_HOME", "/home/ubuntu/.local/state"),
            )
            .env(
                "BULLET_DATA_DIR",
                env_or("BULLET_DATA_DIR", "/home/ubuntu/bullet-live"),
            ),
    )
    .context("spawning bullet tui in a pseudo-terminal")?;

    let mut recorder = GifRecorder::new();
    let mut transcript = String::new();
    let mut masters: Vec<(String, PathBuf)> = Vec::new();

    for (index, beat) in BEATS.iter().enumerate() {
        for key in beat.keys {
            page.press(*key)
                .with_context(|| format!("press for beat {}", beat.name))?;
            std::thread::sleep(Duration::from_millis(120));
        }
        // Assert before recording, so the artifact cannot contain a frame the
        // harness did not verify. Never wait for a quiet screen: the console
        // repaints its snapshot clock on a two second poll and never idles.
        for needle in beat.expect {
            page.wait_for_text(needle, WAIT)
                .with_context(|| format!("beat {} expected {needle:?}", beat.name))?;
        }
        let screen = page.screen();
        if screen.contains_text("panicked") {
            bail!("DEVNODE_RECORD_CONSOLE_PANICKED: at beat {}", beat.name);
        }
        recorder.capture_frame(screen.clone(), page.elapsed_ms());

        let master = frames_dir.join(format!("{index:04}-{}.png", beat.name));
        renderer
            .render_screen(&screen)
            .save(&master)
            .with_context(|| format!("writing master {}", master.display()))?;
        masters.push((beat.name.to_owned(), master));

        transcript.push_str(&format!(
            "=== {index:04} {} @ {} ms ===\n{}\n",
            beat.name,
            page.elapsed_ms(),
            screen.plain_text()
        ));
    }

    // Detach the way an operator does, and prove the daemon outlives the client.
    page.press(Key::Ctrl('c')).context("detach")?;
    std::thread::sleep(Duration::from_millis(600));
    let after = std::process::Command::new("curl")
        .args(["--fail", "--silent", "--max-time", "5"])
        .arg(format!("http://{bind}/health"))
        .output()
        .context("running curl after detach")?;
    if !after.status.success() {
        bail!("DEVNODE_RECORD_DAEMON_DIED_ON_DETACH: /health stopped answering after Ctrl+C");
    }

    let gif = out.join("tui-1080p.gif");
    recorder
        .encode_gif(
            &gif,
            &renderer,
            &GifOptions {
                max_fps: 10,
                max_width_px: None,
                loop_forever: true,
                drop_duplicate_frames: true,
                min_delay_cs: 60,
            },
        )
        .context("encoding the gif")?;

    // Read the geometry back out of the encoded file's own header rather than
    // trusting the encoder. Bytes 6..10 of a GIF are width then height, little
    // endian. Every committed artifact would have failed this check.
    let head = std::fs::read(&gif).context("reading the gif back")?;
    if head.len() < 10 || &head[0..3] != b"GIF" {
        bail!("DEVNODE_RECORD_GIF_INVALID: {} is not a GIF", gif.display());
    }
    let declared_w = u32::from(u16::from_le_bytes([head[6], head[7]]));
    let declared_h = u32::from(u16::from_le_bytes([head[8], head[9]]));
    if (declared_w, declared_h) != (WIDTH, HEIGHT) {
        bail!(
            "DEVNODE_RECORD_GIF_GEOMETRY_DRIFT: header declares {declared_w}x{declared_h}, \
             expected {WIDTH}x{HEIGHT}"
        );
    }

    let transcript_path = out.join("transcript.txt");
    std::fs::File::create(&transcript_path)
        .and_then(|mut f| f.write_all(transcript.as_bytes()))
        .with_context(|| format!("writing {}", transcript_path.display()))?;

    let mut digests = BTreeMap::new();
    digests.insert(
        "tui-1080p.gif".to_owned(),
        serde_json::Value::String(sha256_file(&gif)?),
    );
    digests.insert(
        "transcript.txt".to_owned(),
        serde_json::Value::String(sha256_file(&transcript_path)?),
    );
    for (name, path) in &masters {
        digests.insert(
            format!(
                "frames/{}",
                path.file_name().unwrap_or_default().to_string_lossy()
            ),
            serde_json::json!({"beat": name, "sha256": sha256_file(path)?}),
        );
    }

    let run = serde_json::json!({
        "schema": "bullet.devnode-record.v1",
        "classification": "development recording; no release authority, no gate cleared",
        "geometry": {
            "cols": COLS, "rows": ROWS,
            "cell_width": CELL_W, "cell_height": CELL_H, "padding": PADDING,
            "width_px": WIDTH, "height_px": HEIGHT,
            "declared_by_gif_header": [declared_w, declared_h],
        },
        "subjects": {
            "bullet": {
                "path": bullet.display().to_string(),
                "sha256": sha256_file(&bullet)?,
            },
            "daemon_health": health_body,
            "bind": bind,
        },
        "frames": {
            "captured": recorder.frame_count(),
            "masters": masters.len(),
            "note": "masters are the renderer's own RGBA output written losslessly; the GIF is \
                     palette quantized and is not claimed to preserve them",
        },
        "beats": BEATS.iter().map(|b| b.name).collect::<Vec<_>>(),
        "absent_beats": [
            "a real mission", "its designed detail", "the accepted task", "a LIVE attempt",
            "a filtered list", "a preserved Candidate", "a live audit event",
        ],
        "absent_beats_reason":
            "the ledger holds no durable work: snapshot 0, attempts 0, candidates 0, commands 0. \
             Recording invented rows would make this a mock, so those beats are absent rather \
             than fabricated.",
        "digests": digests,
    });
    let run_path = out.join("run.json");
    std::fs::File::create(&run_path)
        .and_then(|mut f| f.write_all(serde_json::to_string_pretty(&run)?.as_bytes()))
        .with_context(|| format!("writing {}", run_path.display()))?;

    println!(
        "DEVNODE_RECORD_COMPLETE: {} at {declared_w}x{declared_h}, {} frames, {} masters",
        gif.display(),
        recorder.frame_count(),
        masters.len()
    );
    println!("  run manifest {}", run_path.display());
    Ok(())
}
