//! Dev-node TUI flows for `bullet tui`, driven through a real pseudo-terminal.
//!
//! These run only on the development host: the console reads an authenticated
//! session from a live daemon, and provider authentication exists nowhere else.
//! The hosted graph cannot contain this lane, and `ops/ci/workflow-policy.sh`
//! enforces that mechanically.
//!
//! Every assertion names a string the console actually renders. Nothing here
//! waits on a quiet screen: the console repaints its snapshot clock on a two
//! second poll, so "idle" is not a state it reaches.
//!
//! Run these serially. Of six concurrent consoles, two never reached first paint
//! within fifteen seconds; serialised, none failed. That is the measurement, and
//! the concurrency at which first paint starts failing is not known.

use std::time::Duration;
use tuiwright::{Key, Page, SpawnConfig};

const WAIT: Duration = Duration::from_secs(15);

fn shot(name: &str) -> String {
    let dir = std::env::var("BULLET_DEVNODE_ARTIFACTS")
        .unwrap_or_else(|_| "/home/ubuntu/.bullet-orch-tuiwright".into());
    format!("{dir}/{name}.png")
}

fn console(cols: u16, rows: u16) -> anyhow::Result<Page> {
    let page = Page::spawn(
        SpawnConfig::new(
            std::env::var("BULLET_BIN").unwrap_or_else(|_| "/home/ubuntu/.local/bin/bullet".into()),
        )
        .arg("tui")
        .size(cols, rows)
        .env("TERM", "xterm-256color")
        .env(
            "XDG_STATE_HOME",
            std::env::var("BULLET_STATE_HOME")
                .unwrap_or_else(|_| "/home/ubuntu/.bullet-live-state".into()),
        )
        .env(
            "BULLET_DATA_DIR",
            std::env::var("BULLET_DATA_DIR").unwrap_or_else(|_| "/home/ubuntu/bullet-live".into()),
        ),
    )?;
    page.wait_for_text("BULLET", WAIT)?;
    Ok(page)
}

#[test]
fn first_paint_states_the_hold_and_the_key_map() -> anyhow::Result<()> {
    let page = console(120, 40)?;
    page.expect_screen().to_contain_text("Operating HOLD")?;
    page.expect_screen().to_contain_text("Missions")?;
    page.expect_screen().to_contain_text("Ctrl+K navigate")?;
    page.expect_screen().to_contain_text("? help")?;
    page.expect_screen().not_to_contain_text("panicked")?;
    page.screenshot(&shot("first-paint"))?;
    Ok(())
}

#[test]
fn an_empty_view_says_so_instead_of_implying_success() -> anyhow::Result<()> {
    let page = console(120, 40)?;
    // The console must never let an empty projection read as a green farm.
    page.expect_screen()
        .to_contain_text("No durable rows in this view")?;
    page.expect_screen()
        .to_contain_text("No work, approval, or provider")?;
    Ok(())
}

#[test]
fn the_help_sheet_opens_and_closes() -> anyhow::Result<()> {
    let page = console(120, 40)?;
    page.type_text("?")?;
    page.wait_for_text("Operator help", WAIT)?;
    page.expect_screen().to_contain_text("Ctrl+K: navigate views")?;
    page.expect_screen()
        .to_contain_text("Ctrl+C: detach; farm work continues")?;
    // The honesty line is part of the contract, not decoration.
    page.expect_screen()
        .to_contain_text("Candidates are preserved subjects, not approval decisions.")?;
    page.screenshot(&shot("help"))?;
    page.press(Key::Esc)?;
    std::thread::sleep(Duration::from_millis(500));
    page.expect_screen().not_to_contain_text("Operator help")?;
    Ok(())
}

#[test]
fn the_palette_opens_on_ctrl_k_and_lists_every_view() -> anyhow::Result<()> {
    let page = console(120, 40)?;
    page.press(Key::Ctrl('k'))?;
    page.wait_for_text("Enter to open", WAIT)?;
    for view in [
        "Missions",
        "Tasks",
        "Attempts / sessions",
        "Candidates / review",
        "Recent audit events",
        "Context handoffs",
    ] {
        page.expect_screen().to_contain_text(view)?;
    }
    page.screenshot(&shot("palette"))?;
    page.press(Key::Esc)?;
    std::thread::sleep(Duration::from_millis(500));
    page.expect_screen().not_to_contain_text("Enter to open")?;
    Ok(())
}

#[test]
fn the_palette_switches_the_view() -> anyhow::Result<()> {
    let page = console(120, 40)?;
    page.press(Key::Ctrl('k'))?;
    page.wait_for_text("Enter to open", WAIT)?;
    page.press(Key::Char('j'))?;
    page.press(Key::Enter)?;
    page.wait_for_text("Tasks", WAIT)?;
    page.expect_screen().not_to_contain_text("panicked")?;
    page.screenshot(&shot("tasks-view"))?;
    Ok(())
}

#[test]
fn panes_and_rows_respond_to_the_documented_keys() -> anyhow::Result<()> {
    let page = console(120, 40)?;
    page.press(Key::Tab)?;
    page.press(Key::Char('j'))?;
    page.press(Key::Char('k'))?;
    page.press(Key::Char('r'))?;
    std::thread::sleep(Duration::from_millis(900));
    page.expect_screen().to_contain_text("BULLET")?;
    page.expect_screen().not_to_contain_text("panicked")?;
    page.screenshot(&shot("navigated"))?;
    Ok(())
}

#[test]
fn a_narrow_terminal_does_not_break_the_frame() -> anyhow::Result<()> {
    let page = console(120, 40)?;
    page.resize(70, 20)?;
    std::thread::sleep(Duration::from_millis(900));
    page.expect_screen().not_to_contain_text("panicked")?;
    page.screenshot(&shot("narrow"))?;
    page.resize(160, 48)?;
    page.wait_for_text("BULLET", WAIT)?;
    page.screenshot(&shot("wide"))?;
    Ok(())
}

#[test]
fn detach_leaves_the_daemon_answering() -> anyhow::Result<()> {
    let page = console(120, 40)?;
    page.press(Key::Ctrl('c'))?;
    std::thread::sleep(Duration::from_millis(900));
    drop(page);
    // Ctrl+C detaches this client only. A second console must still attach.
    let again = console(120, 40)?;
    again.expect_screen().to_contain_text("Operating HOLD")?;
    Ok(())
}
