use std::fs::File;

use bullet_linux_lease::ReadLease;
use nix::{
    errno::Errno,
    sys::{
        signal::{SigSet, Signal},
        signalfd::{SfdFlags, SignalFd},
    },
};

use crate::coord::CoordError;

/// Retains the kernel's read lease on the exact legacy open-file description.
/// A write open/truncate or conflicting writable mapping either prevents
/// acquisition or makes the directed SIGIO observable before authority moves.
#[derive(Debug)]
pub(in crate::coord::generation::recovery) struct LegacyReadLease {
    kernel: ReadLease,
    signals: SignalFd,
    previous_mask: SigSet,
}

impl LegacyReadLease {
    pub(in crate::coord::generation::recovery) fn acquire(
        legacy: &File,
    ) -> Result<Self, CoordError> {
        let file = legacy.try_clone().map_err(CoordError::io)?;
        let previous_mask = SigSet::thread_get_mask().map_err(signal_error)?;
        let mut lease_mask = SigSet::empty();
        lease_mask.add(Signal::SIGIO);
        lease_mask.thread_block().map_err(signal_error)?;
        let signals =
            match SignalFd::with_flags(&lease_mask, SfdFlags::SFD_NONBLOCK | SfdFlags::SFD_CLOEXEC)
            {
                Ok(signals) => signals,
                Err(error) => {
                    let _ = previous_mask.thread_set_mask();
                    return Err(signal_error(error));
                }
            };
        match signals.read_signal() {
            Ok(None) => {}
            Ok(Some(_)) => {
                let _ = previous_mask.thread_set_mask();
                return Err(unknown("SIGIO was already pending on the recovery thread"));
            }
            Err(error) => {
                let _ = previous_mask.thread_set_mask();
                return Err(signal_error(error));
            }
        }

        let kernel = match ReadLease::acquire(file) {
            Ok(kernel) => kernel,
            Err(error) => {
                let _ = previous_mask.thread_set_mask();
                return Err(unknown(format!(
                    "legacy inode has a conflicting writer or writable mapping: {error}"
                )));
            }
        };
        let lease = Self {
            kernel,
            signals,
            previous_mask,
        };
        lease.revalidate()?;
        Ok(lease)
    }

    pub(in crate::coord::generation::recovery) fn revalidate(&self) -> Result<(), CoordError> {
        if self.signals.read_signal().map_err(signal_error)?.is_some() {
            return Err(unknown(
                "a write opener requested that the retained legacy lease break",
            ));
        }
        self.kernel.revalidate().map_err(|error| {
            unknown(format!(
                "retained legacy read lease was downgraded or forcibly broken: {error}"
            ))
        })
    }

    fn drain_notifications(&self) {
        while matches!(self.signals.read_signal(), Ok(Some(_))) {}
    }
}

impl Drop for LegacyReadLease {
    fn drop(&mut self) {
        self.drain_notifications();
        let _ = self.kernel.release();
        self.drain_notifications();
        let _ = self.previous_mask.thread_set_mask();
    }
}

fn signal_error(error: Errno) -> CoordError {
    unknown(format!("cannot establish lease signal authority: {error}"))
}

fn unknown(reason: impl Into<String>) -> CoordError {
    CoordError::new("LEGACY_WRITE_AUTHORITY_UNKNOWN", reason)
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, OpenOptions},
        io::BufRead,
        os::unix::fs::{MetadataExt, PermissionsExt},
        process::{Child, Command, Stdio},
        sync::mpsc,
        time::{Duration, Instant},
    };

    use super::*;

    struct ControlledChild {
        child: Child,
    }

    impl ControlledChild {
        fn spawn(command: &mut Command) -> Self {
            Self {
                child: command.spawn().unwrap(),
            }
        }

        fn finish(mut self) {
            self.child.stdin.take();
            assert!(self.child.wait().unwrap().success());
        }
    }

    impl Drop for ControlledChild {
        fn drop(&mut self) {
            self.child.stdin.take();
            let _ = self.child.wait();
        }
    }

    fn fixture() -> (tempfile::TempDir, File) {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("events.jsonl");
        fs::write(&path, b"frozen\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        (root, File::open(path).unwrap())
    }

    #[test]
    fn lease_is_retained_and_read_back() {
        let (_root, file) = fixture();
        let lease = LegacyReadLease::acquire(&file).unwrap();
        lease.revalidate().unwrap();
    }

    #[test]
    fn existing_write_descriptor_refuses_acquisition() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("events.jsonl");
        fs::write(&path, b"frozen\n").unwrap();
        let writer = OpenOptions::new().write(true).open(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let reader = File::open(&path).unwrap();
        let error = LegacyReadLease::acquire(&reader).unwrap_err();
        assert_eq!(error.code(), "LEGACY_WRITE_AUTHORITY_UNKNOWN");
        drop(writer);
    }

    #[test]
    fn different_uid_writable_mapping_refuses_acquisition() {
        // Only this harmless cross-user fixture needs a shared parent. Keep
        // private TMPDIR and proof output permissions unchanged.
        let root = tempfile::Builder::new()
            .prefix("bullet-legacy-map-")
            .tempdir_in("/tmp")
            .unwrap();
        let source = root.path().join("mapping.rs");
        let executable = root.path().join("mapping-child");
        fs::write(
            &source,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/legacy-writable-mapping.rs"
            )),
        )
        .unwrap();
        let compiled = Command::new("rustc")
            .args([
                "--edition=2021",
                "--deny=warnings",
                "--crate-name=legacy_mapping_fixture",
            ])
            .arg(&source)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            compiled.status.success(),
            "{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o755)).unwrap();
        let path = root.path().join("events.jsonl");
        fs::write(&path, b"frozen\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap();
        let mut command = Command::new("sudo");
        command
            .args(["-n", "-u", "nobody", "--"])
            .arg(&executable)
            .arg(&path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        let mut child = ControlledChild::spawn(&mut command);
        let mut ready = String::new();
        std::io::BufReader::new(child.child.stdout.take().unwrap())
            .read_line(&mut ready)
            .unwrap();
        let child_uid: u32 = ready
            .strip_prefix("ready:")
            .and_then(|value| value.strip_suffix('\n'))
            .filter(|value| !value.is_empty())
            .and_then(|value| {
                value.bytes().try_fold(0u32, |acc, byte| {
                    byte.is_ascii_digit()
                        .then(|| acc.checked_mul(10)?.checked_add(u32::from(byte - b'0')))
                        .flatten()
                })
            })
            .expect("child must confirm its retained mapping and UID");
        assert_ne!(child_uid, root.path().metadata().unwrap().uid());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let reader = File::open(&path).unwrap();
        let error = LegacyReadLease::acquire(&reader).unwrap_err();
        assert_eq!(error.code(), "LEGACY_WRITE_AUTHORITY_UNKNOWN");
        child.finish();
        let lease = LegacyReadLease::acquire(&reader).unwrap();
        lease.revalidate().unwrap();
    }

    #[test]
    fn write_open_request_breaks_authority_before_release() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("events.jsonl");
        fs::write(&path, b"frozen\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let reader = File::open(&path).unwrap();
        let lease = LegacyReadLease::acquire(&reader).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let (started_tx, started_rx) = mpsc::sync_channel(0);
        let writer = std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            OpenOptions::new().write(true).open(path).unwrap()
        });
        started_rx.recv().unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let error = loop {
            if let Err(error) = lease.revalidate() {
                break error;
            }
            assert!(
                Instant::now() < deadline,
                "lease-break signal was not observed"
            );
            std::thread::yield_now();
        };
        assert_eq!(error.code(), "LEGACY_WRITE_AUTHORITY_UNKNOWN");
        drop(lease);
        drop(writer.join().unwrap());
    }
}
