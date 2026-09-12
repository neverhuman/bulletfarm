#![forbid(unsafe_code)]

mod cases;
mod custody;
mod evidence;
mod fixture;
mod junit;
mod launcher;
mod profile;
mod session;
mod validate;

use anyhow::{bail, ensure, Context, Result};
use std::path::PathBuf;
use std::sync::atomic::Ordering;

fn main() {
    if let Err(error) = run() {
        eprintln!("TUIWRIGHT_QUALIFICATION_FAILED: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().map(String::as_str) == Some("--validate-hosted") {
        return validate::cli(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("--launch") {
        return launcher::run(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("--probe") {
        return launcher::probe(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("--parent-probe") {
        ensure!(args.len() == 3, "PARENT_PROBE_ARGUMENTS_INVALID");
        return custody::parent_probe(std::path::Path::new(&args[1]), args[2].parse()?);
    }
    let selected = custody::IDENTITIES
        .iter()
        .chain(cases::IDENTITIES.iter())
        .copied()
        .collect::<Vec<_>>();
    if args.as_slice() == ["--list"] {
        println!("{}", serde_json::to_string_pretty(&selected)?);
        return Ok(());
    }
    if args.first().map(String::as_str) == Some("installed") {
        eprintln!("SIGNED_INSTALLED_PACKAGE_ADMISSION_REQUIRED: package verification and authenticated installation qualification predecessors are unavailable");
        std::process::exit(78);
    }
    let profile = profile::admit(&args)?;
    let binary = PathBuf::from(&args[2]);
    ensure!(
        binary.is_absolute() && binary.canonicalize()? == binary,
        "BINARY_CANONICAL_PATH_REQUIRED"
    );
    ensure!(
        args[4].len() == 64
            && args[4]
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
        "BINARY_DIGEST_INVALID"
    );
    ensure!(
        evidence::hash(&binary)? == args[4],
        "BINARY_DIGEST_MISMATCH"
    );
    let output = PathBuf::from(&args[6]);
    ensure!(output.is_absolute(), "OUTPUT_ABSOLUTE_PATH_REQUIRED");
    let mut report = evidence::Evidence::create(&output, &binary, &args[4], &selected)?;
    report.event("profile", "admitted_component_context", profile.clone())?;
    // The watchdog is independent of any blocking Page/fixture cleanup. A killed
    // run leaves incomplete artifacts and cannot emit an accepted manifest.
    let (deadline_done, deadline_receiver) = std::sync::mpsc::channel::<()>();
    let self_pidfd = rustix::process::pidfd_open(
        rustix::process::getpid(),
        rustix::process::PidfdFlags::empty(),
    )?;
    let watchdog = std::thread::spawn(move || {
        if deadline_receiver.recv_timeout(std::time::Duration::from_secs(180))
            == Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        {
            eprintln!("TUIWRIGHT_RUN_DEADLINE_EXCEEDED: incomplete observations remain nonpassing");
            let _ = rustix::process::pidfd_send_signal(&self_pidfd, rustix::process::Signal::KILL);
        }
    });
    let mut failures = Vec::new();
    for id in &selected {
        report.event(id, "started", serde_json::json!({}))?;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if custody::IDENTITIES.contains(id) {
                custody::run(id, &mut report)
            } else {
                cases::run(id, &binary, &mut report)
            }
        }));
        let result = result
            .map_err(|_| anyhow::anyhow!("CASE_PANICKED"))
            .and_then(|result| result);
        report.complete(id, &result)?;
        if let Err(error) = result {
            failures.push(format!("{id}: {error:#}"));
        }
        if session::CLEANUP_FAILED.load(Ordering::SeqCst) {
            failures.push("PROCESS_CUSTODY_UNRESOLVED: remaining cases not launched".into());
            break;
        }
    }
    if evidence::hash(&binary)? != args[4] {
        failures.push("BINARY_CHANGED_DURING_RUN".into());
    }
    match profile::admit(&args) {
        Ok(observed) if observed == profile => {}
        Ok(_) => failures.push("PROFILE_SUBJECT_CHANGED_DURING_RUN".into()),
        Err(error) => failures.push(format!("PROFILE_RECHECK_FAILED: {error:#}")),
    }
    report
        .finish(&failures)
        .context("EVIDENCE_FINALIZATION_FAILED")?;
    drop(deadline_done);
    ensure!(watchdog.join().is_ok(), "DEADLINE_WATCHDOG_FAILED");
    if !failures.is_empty() {
        bail!("{}", failures.join("\n"));
    }
    println!("COMPONENT_PROOF: {} Tuiwright cases completed; installed authentication and release qualification remain unavailable", selected.len());
    Ok(())
}
