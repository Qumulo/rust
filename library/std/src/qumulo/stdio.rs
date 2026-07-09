use crate::io;
use crate::qumulo::bindings::{
    rust_sys_flush_stderr, rust_sys_flush_stdout, rust_sys_read_stdin, rust_sys_write_stderr,
    rust_sys_write_stdout,
};
use crate::sys::cvt_nz;

pub struct Stdin(());
pub struct Stdout(());
pub struct Stderr(());

impl Stdin {
    pub const fn new() -> Stdin {
        Stdin(())
    }
}

impl io::Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut read: u64 = 0;
        cvt_nz(unsafe {
            rust_sys_read_stdin(buf.as_mut_ptr() as *mut _, buf.len() as u64, &mut read)
        })?;
        Ok(read as usize)
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout(())
    }
}

impl io::Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        cvt_nz(unsafe { rust_sys_write_stdout(buf.as_ptr() as *const _, buf.len() as u64) })?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        cvt_nz(unsafe { rust_sys_flush_stdout() })?;
        Ok(())
    }
}

impl Stderr {
    pub const fn new() -> Stderr {
        Stderr(())
    }
}

impl io::Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        cvt_nz(unsafe { rust_sys_write_stderr(buf.as_ptr() as *const _, buf.len() as u64) })?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        cvt_nz(unsafe { rust_sys_flush_stderr() })?;
        Ok(())
    }
}

// std uses this to ignore ebadf errors, but our code will panic on them. This is just copied from
// the regular Linux target.
pub fn is_ebadf(err: &io::Error) -> bool {
    err.raw_os_error() == Some(libc::EBADF as i32)
}

// Don't allow any buffering of stdin, it doesn't work with the way we are reading from it.
pub const STDIN_BUF_SIZE: usize = 1;

// We handle panicking ourselves, so this shouldn't be written to anyway. This is just copied from
// the regular Linux target.
pub fn panic_output() -> Option<impl io::Write> {
    Some(Stderr::new())
}
