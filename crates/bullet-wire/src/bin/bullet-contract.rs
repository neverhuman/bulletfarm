use std::{env, path::PathBuf, process::ExitCode};

use bullet_wire::{ContractMode, execute_contract_tool};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("bullet-contract: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args().skip(1);
    let mode = match arguments.next().as_deref() {
        Some("generate") => ContractMode::Generate,
        Some("check") => ContractMode::Check,
        _ => return Err("usage: bullet-contract <generate|check> --root PATH".to_owned()),
    };
    if arguments.next().as_deref() != Some("--root") {
        return Err("usage: bullet-contract <generate|check> --root PATH".to_owned());
    }
    let root = arguments
        .next()
        .map(PathBuf::from)
        .ok_or_else(|| "--root requires a path".to_owned())?;
    if arguments.next().is_some() {
        return Err("unexpected trailing argument".to_owned());
    }
    execute_contract_tool(&root, mode).map_err(|error| error.to_string())
}
