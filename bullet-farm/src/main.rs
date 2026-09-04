use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().collect();
    if bullet_family::forge::should_intercept(&args) {
        return match bullet_family::forge::execute(args, env::current_dir()) {
            Ok(outcome) => print_ok(outcome.output(), outcome.exit_code()),
            Err(error) => print_err(&error),
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
        Err(error) => print_err(&error),
    }
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

fn print_err(error: &bullet_family::coord::CoordError) -> ExitCode {
    eprintln!("bullet-family: {error}");
    ExitCode::from(error.exit_code())
}
