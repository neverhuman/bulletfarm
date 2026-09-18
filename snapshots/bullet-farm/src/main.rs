use std::{env, ffi::OsString, io::IsTerminal, process::ExitCode};

use bullet_family::{
    coord::CoordError,
    diagnostic::{self, ColorChoice},
};

fn main() -> ExitCode {
    let mut args: Vec<_> = env::args_os().collect();
    // Colour is decided before anything else runs, so that a refusal raised by
    // argument parsing itself is printed under the same rule as every other.
    let choice = match take_color_choice(&mut args) {
        Ok(choice) => choice,
        Err(error) => return print_err(&error, ColorChoice::Auto),
    };
    if bullet_family::forge::should_intercept(&args) {
        return match bullet_family::forge::execute(args, env::current_dir()) {
            Ok(outcome) => print_ok(outcome.output(), outcome.exit_code()),
            Err(error) => print_err(&error, choice),
        };
    }
    if let Some(banner) = bullet_family::forge::setup_forge_banner(&args) {
        print!("{banner}");
        if bullet_family::forge::setup_forge_only(&args) {
            return ExitCode::from(bullet_family::forge::SETUP_FORGE_ONLY_EXIT_CODE);
        }
    }
    match bullet_family::cli::execute(args, env::current_dir()) {
        Ok(outcome) => print_ok(outcome.output(), outcome.exit_code()),
        Err(error) => print_err(&error, choice),
    }
}

/// Remove the colour flags from argv and report what they asked for.
///
/// They are removed rather than passed through because colour is a property of
/// the terminal, not of any command, and every command would otherwise have to
/// know to ignore them. Last flag wins, which is what a person retrying a
/// command with `--no-color` on the end expects.
fn take_color_choice(args: &mut Vec<OsString>) -> Result<ColorChoice, CoordError> {
    let mut choice = ColorChoice::Auto;
    let mut kept: Vec<OsString> = Vec::with_capacity(args.len());
    let mut rest = std::mem::take(args).into_iter();
    if let Some(program) = rest.next() {
        kept.push(program);
    }
    while let Some(arg) = rest.next() {
        match arg.to_str() {
            Some("--no-color" | "--plain") => choice = ColorChoice::Never,
            Some("--color") => {
                let value = rest.next().ok_or_else(|| {
                    CoordError::new("USAGE", "--color expects auto, always or never")
                })?;
                let value = value.to_str().ok_or_else(|| {
                    CoordError::new("USAGE", "--color expects auto, always or never")
                })?;
                choice = ColorChoice::parse(value)?;
            }
            Some(other) if other.starts_with("--color=") => {
                choice = ColorChoice::parse(&other["--color=".len()..])?;
            }
            _ => kept.push(arg),
        }
    }
    *args = kept;
    Ok(choice)
}

fn print_ok(output: &str, exit_code: u8) -> ExitCode {
    if !output.is_empty() {
        if output.ends_with('\n') {
            print!("{output}");
        } else {
            println!("{output}");
        }
    }
    ExitCode::from(exit_code)
}

fn print_err(error: &CoordError, choice: ColorChoice) -> ExitCode {
    let facts = diagnostic::facts_for(std::io::stderr().is_terminal());
    let depth = diagnostic::depth_for(choice, facts.borrow());
    eprint!(
        "{}",
        diagnostic::render(error, depth, diagnostic::width_from_env())
    );
    ExitCode::from(error.exit_code())
}
