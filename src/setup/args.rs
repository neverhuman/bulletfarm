use std::path::PathBuf;

use crate::coord::CoordError;

const USAGE: &str = "usage: bullet-family setup --root PATH --source jeryu --cargo-bin ABSOLUTE_PATH --node-bin ABSOLUTE_PATH --npm-cli ABSOLUTE_PATH [--offline]";

#[derive(Debug)]
pub(super) struct SetupArgs {
    pub(super) root: PathBuf,
    pub(super) offline: bool,
    pub(super) cargo_bin: Option<PathBuf>,
    pub(super) node_bin: Option<PathBuf>,
    pub(super) npm_cli: Option<PathBuf>,
}

pub(super) fn parse_args(
    explicit_root: Option<&str>,
    args: &[String],
) -> Result<SetupArgs, CoordError> {
    let mut root = explicit_root.map(PathBuf::from);
    let mut source = None;
    let mut offline = false;
    let mut cargo_bin = None;
    let mut node_bin = None;
    let mut npm_cli = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--offline" if !offline => {
                offline = true;
                index += 1;
            }
            "--root" | "--source" | "--cargo-bin" | "--node-bin" | "--npm-cli" => {
                let option = args[index].as_str();
                let value = args.get(index + 1).ok_or_else(|| {
                    CoordError::new("MISSING_VALUE", format!("{option} needs a value"))
                })?;
                match option {
                    "--root" if root.is_none() => root = Some(PathBuf::from(value)),
                    "--source" if source.is_none() => source = Some(value.clone()),
                    "--cargo-bin" if cargo_bin.is_none() => {
                        cargo_bin = Some(PathBuf::from(value));
                    }
                    "--node-bin" if node_bin.is_none() => node_bin = Some(PathBuf::from(value)),
                    "--npm-cli" if npm_cli.is_none() => npm_cli = Some(PathBuf::from(value)),
                    _ => return Err(CoordError::new("DUPLICATE_OPTION", USAGE)),
                }
                index += 2;
            }
            _ => return Err(CoordError::new("USAGE", USAGE)),
        }
    }
    if source.as_deref() != Some("jeryu") {
        return Err(CoordError::new(
            "UNSUPPORTED_SOURCE",
            "--source jeryu is required; no branch or sibling-path source is permitted",
        ));
    }
    Ok(SetupArgs {
        root: root.ok_or_else(|| CoordError::new("MISSING_OPTION", "--root is required"))?,
        offline,
        cargo_bin,
        node_bin,
        npm_cli,
    })
}
