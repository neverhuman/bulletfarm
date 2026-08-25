//! Dedicated non-release gitd. Production `bullet-gitd` never includes this
//! target. Clone is confined to one pre-opened private fixture root and a
//! local test key.

use bullet_gitd::daemon::Daemon;
use bullet_gitd::fixture_permit::{parse_fixture_key, require_preopened_fixture_root};
use bullet_gitd::protocol;
use serde_json::Value;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let config = match parse_args(std::env::args().skip(1)) {
        Ok(config) => config,
        Err(message) => {
            eprintln!("bullet-gitd-fixture: {message}");
            return ExitCode::from(2);
        }
    };
    let fixture_root = match require_preopened_fixture_root(&config.fixture_root) {
        Ok(root) => root,
        Err(error) => {
            eprintln!("bullet-gitd-fixture: {error}");
            return ExitCode::from(2);
        }
    };
    let key = match parse_fixture_key(&config.key_hex) {
        Ok(key) => key,
        Err(error) => {
            eprintln!("bullet-gitd-fixture: {error}");
            return ExitCode::from(2);
        }
    };
    let mut daemon = match Daemon::fixture(&config.ledger_root, &fixture_root, key) {
        Ok(daemon) => daemon,
        Err(error) => {
            eprintln!("bullet-gitd-fixture: {error}");
            return ExitCode::FAILURE;
        }
    };
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

struct Config {
    fixture_root: PathBuf,
    key_hex: String,
    ledger_root: PathBuf,
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Config, String> {
    let mut fixture_root = None;
    let mut key_hex = None;
    let mut ledger_root = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--fixture-root" => {
                fixture_root = Some(PathBuf::from(
                    args.next().ok_or("--fixture-root requires a directory")?,
                ));
            }
            "--fixture-key-hex" => {
                key_hex = Some(args.next().ok_or("--fixture-key-hex requires 64 hex")?);
            }
            "--mutation-ledger" => {
                ledger_root = Some(PathBuf::from(
                    args.next()
                        .ok_or("--mutation-ledger requires a directory")?,
                ));
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    let fixture_root = fixture_root.ok_or("--fixture-root is required")?;
    let key_hex = key_hex.ok_or("--fixture-key-hex is required")?;
    let ledger_root = ledger_root.unwrap_or_else(|| fixture_root.join("mutation-ledger"));
    Ok(Config {
        fixture_root,
        key_hex,
        ledger_root,
    })
}
