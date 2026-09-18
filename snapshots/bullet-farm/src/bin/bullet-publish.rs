//! Explicit public-index publication; never runs from a proof lane.
#[cfg(target_os = "linux")]
fn main() -> std::process::ExitCode {
    match bullet_family::publication::run(std::env::args().skip(1).collect()) {
        Ok(output) => {
            println!("{output}");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn main() -> std::process::ExitCode {
    eprintln!("PUBLICATION_PLATFORM_UNSUPPORTED: Linux publication custody is required");
    std::process::ExitCode::FAILURE
}
