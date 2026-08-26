//! Bounded child-process execution for local authority and installation tools.

use std::{
    fs::File,
    io::{self, Read, Write},
    process::{ChildStdin, Command, Output, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use crate::coord::CoordError;

const POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Copy)]
pub(crate) struct Limits {
    pub(crate) timeout: Duration,
    pub(crate) stdout_bytes: usize,
    pub(crate) stderr_bytes: usize,
}

pub(crate) struct InputFileOutput {
    pub(crate) output: Output,
    pub(crate) byte_count: u64,
    pub(crate) digest: [u8; 32],
}

struct ExecutionOutput {
    output: Output,
    input: Option<InputSubject>,
}

struct InputSubject {
    byte_count: u64,
    digest: [u8; 32],
}

pub(crate) fn run_bounded(
    command: &mut Command,
    label: &str,
    limits: Limits,
) -> Result<Output, CoordError> {
    Ok(run_bounded_inner(command, label, limits, None)?.output)
}

pub(crate) fn run_bounded_with_input_file(
    command: &mut Command,
    label: &str,
    limits: Limits,
    input: File,
) -> Result<InputFileOutput, CoordError> {
    let execution = run_bounded_inner(command, label, limits, Some(input))?;
    let input = execution.input.ok_or_else(|| {
        CoordError::new(
            "COMMAND_INPUT_FAILED",
            format!("{label} returned no input receipt"),
        )
    })?;
    Ok(InputFileOutput {
        output: execution.output,
        byte_count: input.byte_count,
        digest: input.digest,
    })
}

fn run_bounded_inner(
    command: &mut Command,
    label: &str,
    limits: Limits,
    input: Option<File>,
) -> Result<ExecutionOutput, CoordError> {
    let deadline = Instant::now().checked_add(limits.timeout).ok_or_else(|| {
        CoordError::new("INVALID_COMMAND_DEADLINE", "command deadline overflowed")
    })?;
    command.stdin(if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    #[cfg(unix)]
    command.process_group(0);

    let mut child = command.spawn().map_err(|error| {
        CoordError::new(
            "COMMAND_START_FAILED",
            format!("could not start {label}: {error}"),
        )
    })?;
    let Some(stdout) = child.stdout.take() else {
        terminate_process_group(&mut child);
        let _ = child.wait();
        return Err(CoordError::new(
            "COMMAND_IO_FAILED",
            format!("{label} stdout pipe was unavailable"),
        ));
    };
    let Some(stderr) = child.stderr.take() else {
        terminate_process_group(&mut child);
        let _ = child.wait();
        return Err(CoordError::new(
            "COMMAND_IO_FAILED",
            format!("{label} stderr pipe was unavailable"),
        ));
    };
    let mut input_writer = match input {
        Some(input) => {
            let Some(stdin) = child.stdin.take() else {
                terminate_process_group(&mut child);
                let _ = child.wait();
                return Err(CoordError::new(
                    "COMMAND_INPUT_FAILED",
                    format!("{label} stdin pipe was unavailable"),
                ));
            };
            Some(capture_input(input, stdin).map_err(|error| {
                terminate_process_group(&mut child);
                let _ = child.wait();
                CoordError::new(
                    "COMMAND_INPUT_FAILED",
                    format!("could not start {label} input writer: {error}"),
                )
            })?)
        }
        None => None,
    };
    let stdout_exceeded = Arc::new(AtomicBool::new(false));
    let stderr_exceeded = Arc::new(AtomicBool::new(false));
    let stdout_reader = match capture(stdout, limits.stdout_bytes, Arc::clone(&stdout_exceeded)) {
        Ok(reader) => reader,
        Err(error) => {
            terminate_and_join_input(&mut child, &mut input_writer, label);
            return Err(CoordError::new(
                "COMMAND_IO_FAILED",
                format!("could not start {label} stdout reader: {error}"),
            ));
        }
    };
    let stderr_reader = match capture(stderr, limits.stderr_bytes, Arc::clone(&stderr_exceeded)) {
        Ok(reader) => reader,
        Err(error) => {
            terminate_and_join_input(&mut child, &mut input_writer, label);
            let _ = join_capture(stdout_reader, label, "stdout");
            return Err(CoordError::new(
                "COMMAND_IO_FAILED",
                format!("could not start {label} stderr reader: {error}"),
            ));
        }
    };
    let status = loop {
        if stdout_exceeded.load(Ordering::Acquire) || stderr_exceeded.load(Ordering::Acquire) {
            terminate_and_join_input(&mut child, &mut input_writer, label);
            drop(stdout_reader);
            drop(stderr_reader);
            return Err(output_limit_error(label, limits));
        }
        if Instant::now() >= deadline {
            terminate_and_join_input(&mut child, &mut input_writer, label);
            drop(stdout_reader);
            drop(stderr_reader);
            return Err(CoordError::new(
                "COMMAND_TIMEOUT",
                format!(
                    "{label} exceeded its {} second deadline",
                    limits.timeout.as_secs()
                ),
            ));
        }
        if stdout_reader.is_finished() && stderr_reader.is_finished() {
            if stdout_exceeded.load(Ordering::Acquire) || stderr_exceeded.load(Ordering::Acquire) {
                terminate_and_join_input(&mut child, &mut input_writer, label);
                let _ = join_capture(stdout_reader, label, "stdout");
                let _ = join_capture(stderr_reader, label, "stderr");
                return Err(output_limit_error(label, limits));
            }
            if Instant::now() >= deadline {
                terminate_and_join_input(&mut child, &mut input_writer, label);
                let _ = join_capture(stdout_reader, label, "stdout");
                let _ = join_capture(stderr_reader, label, "stderr");
                return Err(CoordError::new(
                    "COMMAND_TIMEOUT",
                    format!(
                        "{label} exceeded its {} second deadline",
                        limits.timeout.as_secs()
                    ),
                ));
            }
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => {}
                Err(error) => {
                    terminate_and_join_input(&mut child, &mut input_writer, label);
                    let _ = join_capture(stdout_reader, label, "stdout");
                    let _ = join_capture(stderr_reader, label, "stderr");
                    return Err(CoordError::new(
                        "COMMAND_WAIT_FAILED",
                        format!("could not wait for {label}: {error}"),
                    ));
                }
            }
        }
        thread::sleep(POLL_INTERVAL);
    };
    let stdout = join_capture(stdout_reader, label, "stdout");
    let stderr = join_capture(stderr_reader, label, "stderr");
    if stdout_exceeded.load(Ordering::Acquire) || stderr_exceeded.load(Ordering::Acquire) {
        return Err(output_limit_error(label, limits));
    }
    let stdout = stdout?;
    let stderr = stderr?;
    let input = join_input(&mut input_writer, label)?;
    Ok(ExecutionOutput {
        output: Output {
            status,
            stdout,
            stderr,
        },
        input,
    })
}

fn capture_input(
    mut input: File,
    mut stdin: ChildStdin,
) -> io::Result<JoinHandle<io::Result<InputSubject>>> {
    thread::Builder::new()
        .name("bullet-child-input".to_owned())
        .spawn(move || {
            let mut byte_count = 0_u64;
            let mut hasher = blake3::Hasher::new();
            let mut buffer = [0_u8; 64 * 1024];
            loop {
                let count = input.read(&mut buffer)?;
                if count == 0 {
                    stdin.flush()?;
                    return Ok(InputSubject {
                        byte_count,
                        digest: *hasher.finalize().as_bytes(),
                    });
                }
                stdin.write_all(&buffer[..count])?;
                hasher.update(&buffer[..count]);
                byte_count = byte_count
                    .checked_add(count as u64)
                    .ok_or_else(|| io::Error::other("child input byte count overflow"))?;
            }
        })
}

fn join_input(
    writer: &mut Option<JoinHandle<io::Result<InputSubject>>>,
    label: &str,
) -> Result<Option<InputSubject>, CoordError> {
    let Some(writer) = writer.take() else {
        return Ok(None);
    };
    writer
        .join()
        .map_err(|_| {
            CoordError::new(
                "COMMAND_INPUT_FAILED",
                format!("{label} input writer panicked"),
            )
        })?
        .map(Some)
        .map_err(|error| {
            CoordError::new(
                "COMMAND_INPUT_FAILED",
                format!("could not stream {label} input: {error}"),
            )
        })
}

fn terminate_and_join_input(
    child: &mut std::process::Child,
    writer: &mut Option<JoinHandle<io::Result<InputSubject>>>,
    label: &str,
) {
    terminate_process_group(child);
    let _ = child.wait();
    let _ = join_input(writer, label);
}

fn output_limit_error(label: &str, limits: Limits) -> CoordError {
    CoordError::new(
        "COMMAND_OUTPUT_LIMIT",
        format!(
            "{label} exceeded its output limit (stdout {} bytes, stderr {} bytes)",
            limits.stdout_bytes, limits.stderr_bytes
        ),
    )
}

fn capture<R: Read + Send + 'static>(
    mut reader: R,
    limit: usize,
    exceeded: Arc<AtomicBool>,
) -> io::Result<JoinHandle<io::Result<Vec<u8>>>> {
    thread::Builder::new()
        .name("bullet-child-output".to_owned())
        .spawn(move || {
            let mut bytes = Vec::with_capacity(limit.min(8 * 1024));
            let mut buffer = [0_u8; 8 * 1024];
            loop {
                let count = reader.read(&mut buffer)?;
                if count == 0 {
                    return Ok(bytes);
                }
                let remaining = limit.saturating_sub(bytes.len());
                if count > remaining {
                    bytes.extend_from_slice(&buffer[..remaining]);
                    exceeded.store(true, Ordering::Release);
                    return Ok(bytes);
                }
                bytes.extend_from_slice(&buffer[..count]);
            }
        })
}

fn join_capture(
    reader: JoinHandle<io::Result<Vec<u8>>>,
    label: &str,
    stream: &str,
) -> Result<Vec<u8>, CoordError> {
    reader
        .join()
        .map_err(|_| {
            CoordError::new(
                "COMMAND_IO_FAILED",
                format!("{label} {stream} reader panicked"),
            )
        })?
        .map_err(|error| {
            CoordError::new(
                "COMMAND_IO_FAILED",
                format!("could not read {label} {stream}: {error}"),
            )
        })
}

#[cfg(unix)]
fn terminate_process_group(child: &mut std::process::Child) {
    use nix::{
        sys::signal::{Signal, killpg},
        unistd::Pid,
    };

    if let Ok(raw_pid) = i32::try_from(child.id()) {
        let _ = killpg(Pid::from_raw(raw_pid), Signal::SIGKILL);
    }
    let _ = child.kill();
}

#[cfg(not(unix))]
fn terminate_process_group(child: &mut std::process::Child) {
    let _ = child.kill();
}

#[cfg(all(test, unix))]
#[path = "../tests/support/process_unit.rs"]
mod tests;
