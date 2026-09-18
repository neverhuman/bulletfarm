#![cfg(target_os = "linux")]

use std::{fs::File, io};

#[derive(Debug)]
pub struct ReadLease {
    file: File,
    armed: bool,
}

impl ReadLease {
    pub fn acquire(file: File) -> io::Result<Self> {
        syscall::direct_notifications_to_current_thread(&file)?;
        syscall::set_lease(&file, syscall::LeaseKind::Read)?;
        let lease = Self { file, armed: true };
        lease.revalidate()?;
        Ok(lease)
    }

    pub fn revalidate(&self) -> io::Result<()> {
        match syscall::get_lease(&self.file)? {
            syscall::LeaseKind::Read => Ok(()),
            observed => Err(io::Error::other(format!(
                "expected retained read lease, observed {observed:?}"
            ))),
        }
    }

    pub fn release(&mut self) -> io::Result<()> {
        if self.armed {
            syscall::set_lease(&self.file, syscall::LeaseKind::None)?;
            self.armed = false;
        }
        Ok(())
    }
}

impl Drop for ReadLease {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

#[allow(unsafe_code)]
mod syscall {
    use std::{fs::File, io, os::fd::AsRawFd};

    use libc::{self, c_int, pid_t};

    const F_SETOWN_EX_LINUX: c_int = 15;
    const F_OWNER_TID_LINUX: c_int = 0;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(super) enum LeaseKind {
        None,
        Read,
        Write,
    }

    #[repr(C)]
    struct FileOwner {
        kind: c_int,
        pid: pid_t,
    }

    pub(super) fn direct_notifications_to_current_thread(file: &File) -> io::Result<()> {
        let owner = FileOwner {
            kind: F_OWNER_TID_LINUX,
            // SAFETY: SYS_gettid takes no arguments and returns the caller's Linux TID.
            pid: unsafe { libc::syscall(libc::SYS_gettid) as pid_t },
        };
        // SAFETY: the retained descriptor is valid and F_SETOWN_EX reads exactly
        // one correctly laid-out FileOwner during the call.
        let result = unsafe {
            libc::fcntl(
                file.as_raw_fd(),
                F_SETOWN_EX_LINUX,
                std::ptr::from_ref(&owner),
            )
        };
        errno_result(result).map(drop)
    }

    pub(super) fn set_lease(file: &File, lease: LeaseKind) -> io::Result<()> {
        let value = match lease {
            LeaseKind::None => libc::F_UNLCK,
            LeaseKind::Read => libc::F_RDLCK,
            LeaseKind::Write => libc::F_WRLCK,
        };
        // SAFETY: F_SETLEASE accepts the integer lease kind as its third argument.
        let result = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETLEASE, value) };
        errno_result(result).map(drop)
    }

    pub(super) fn get_lease(file: &File) -> io::Result<LeaseKind> {
        // SAFETY: F_GETLEASE has no variadic payload and does not mutate memory.
        let result = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETLEASE) };
        match errno_result(result)? {
            value if value == libc::F_UNLCK => Ok(LeaseKind::None),
            value if value == libc::F_RDLCK => Ok(LeaseKind::Read),
            value if value == libc::F_WRLCK => Ok(LeaseKind::Write),
            value => Err(io::Error::other(format!(
                "kernel returned unknown lease kind {value}"
            ))),
        }
    }

    fn errno_result(result: c_int) -> io::Result<c_int> {
        if result == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(result)
        }
    }
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use std::{
        fs::{self, File, OpenOptions},
        io::Read,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::fs::PermissionsExt,
        },
    };

    use super::ReadLease;

    struct Child(libc::pid_t);

    impl Drop for Child {
        fn drop(&mut self) {
            // SAFETY: the PID was returned by fork in the parent; kill and
            // waitpid operate only on that child and do not access Rust memory.
            unsafe {
                libc::kill(self.0, libc::SIGKILL);
                libc::waitpid(self.0, std::ptr::null_mut(), 0);
            }
        }
    }

    #[test]
    fn writable_mapping_in_another_process_blocks_read_lease() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("events.jsonl");
        fs::write(&path, b"frozen\n").unwrap();
        let writer = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let mut pipe = [0; 2];
        // SAFETY: `pipe` has storage for exactly two descriptors.
        assert_eq!(unsafe { libc::pipe(pipe.as_mut_ptr()) }, 0);
        // SAFETY: the child performs only async-signal-safe syscalls before _exit.
        let pid = unsafe { libc::fork() };
        assert!(pid >= 0);
        if pid == 0 {
            // SAFETY: all arguments describe the inherited regular file; the
            // child closes its writer descriptor after retaining the mapping.
            let mapping = unsafe {
                libc::mmap(
                    std::ptr::null_mut(),
                    7,
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_SHARED,
                    writer.as_raw_fd(),
                    0,
                )
            };
            let ready = u8::from(mapping != libc::MAP_FAILED);
            // SAFETY: this post-fork child uses valid inherited descriptors and a
            // one-byte pointer to `ready`, then pauses or exits without returning.
            unsafe {
                libc::close(writer.as_raw_fd());
                libc::close(pipe[0]);
                libc::write(pipe[1], std::ptr::from_ref(&ready).cast(), 1);
                if ready == 1 {
                    libc::pause();
                }
                libc::_exit(i32::from(ready == 0));
            }
        }
        let child = Child(pid);
        drop(writer);
        // SAFETY: the parent owns these pipe ends after fork.
        unsafe {
            libc::close(pipe[1]);
        }
        let mut ready = [0_u8; 1];
        // SAFETY: from_raw_fd takes ownership of the parent's read end once.
        let mut readiness = unsafe { File::from_raw_fd(pipe[0]) };
        readiness.read_exact(&mut ready).unwrap();
        assert_eq!(ready, [1]);
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        let error = ReadLease::acquire(File::open(path).unwrap()).unwrap_err();
        assert_eq!(error.raw_os_error(), Some(libc::EAGAIN));
        drop(child);
    }
}
