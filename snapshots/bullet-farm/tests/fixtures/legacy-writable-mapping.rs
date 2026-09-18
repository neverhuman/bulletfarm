//! Harmless Linux child for the cross-user legacy lease regression.
use std::{
    ffi::{c_int, c_void},
    fs::OpenOptions,
    io::{self, Read, Write},
    os::fd::AsRawFd,
};

unsafe extern "C" {
    fn mmap(
        address: *mut c_void,
        length: usize,
        protection: c_int,
        flags: c_int,
        descriptor: c_int,
        offset: i64,
    ) -> *mut c_void;
    fn munmap(address: *mut c_void, length: usize) -> c_int;
    fn getuid() -> u32;
}

fn main() -> io::Result<()> {
    let path = std::env::args_os().nth(1).expect("fixture path required");
    let file = OpenOptions::new().read(true).write(true).open(path)?;
    assert_eq!(file.metadata()?.len(), 7);
    // SAFETY: the descriptor names the seven-byte regular fixture. Linux
    // PROT_READ | PROT_WRITE and MAP_SHARED retain its writable mapping.
    let mapping = unsafe { mmap(std::ptr::null_mut(), 7, 3, 1, file.as_raw_fd(), 0) };
    if mapping == (-1_isize) as *mut c_void {
        return Err(io::Error::last_os_error());
    }
    drop(file);
    // SAFETY: getuid takes no arguments and returns this child's real UID.
    println!("ready:{}", unsafe { getuid() });
    io::stdout().flush()?;
    let mut stop = [0_u8; 1];
    io::stdin().read(&mut stop)?;
    // SAFETY: this process still owns exactly the retained seven-byte mapping.
    if unsafe { munmap(mapping, 7) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
