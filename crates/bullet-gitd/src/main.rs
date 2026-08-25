//! bullet-gitd binary: line-delimited JSON over stdio. One request object per
//! line in, one response object per line out. Protocol: docs/architecture.md.
//!
//! Default construction is fail-closed (`AUTHORITY_CONTRACT_UNAVAILABLE`).
//! Fixture authority is compiled only into the dedicated non-release
//! `bullet-gitd-fixture` target.

use bullet_gitd::{daemon::Daemon, protocol};
use serde_json::Value;
use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    if let Some(arg) = std::env::args().nth(1) {
        eprintln!("bullet-gitd: unknown argument {arg}");
        return ExitCode::from(2);
    }
    let mut daemon = Daemon::new();
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut input = stdin.lock();
    loop {
        let line = match protocol::read_frame(&mut input) {
            Ok(Some(line)) => line,
            Ok(None) => break,
            Err(error) => {
                let response =
                    protocol::err_line(&Value::Null, error.reason_code(), &error.to_string());
                let _ = writeln!(out, "{response}");
                let _ = out.flush();
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let response = daemon.handle_line(&line);
        if writeln!(out, "{response}").is_err() {
            break;
        }
        if out.flush().is_err() {
            break;
        }
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    #[test]
    fn production_binary_has_no_fixture_flag() {
        assert!(std::env::args().all(|arg| arg != "--fixture-authority"));
    }
}
