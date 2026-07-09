// Adapted from the Rustc std library source, which is licensed under MIT and Apache.

#![unstable(reason = "not public", issue = "none", feature = "fd")]

use libc::{c_int, c_void};

use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut, Read};
use crate::os::unix::io::{AsFd, AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, OwnedFd, RawFd};
use crate::sys::{AsInner, FromInner, IntoInner, cvt};
use crate::{cmp, mem};

#[derive(Debug)]
pub struct FileDesc(OwnedFd);

// The maximum read limit on most POSIX-like systems is `SSIZE_MAX`,
// with the man page quoting that if the count of bytes to read is
// greater than `SSIZE_MAX` the result is "unspecified".
const READ_LIMIT: usize = libc::ssize_t::MAX as usize;

const fn max_iov() -> usize {
    libc::UIO_MAXIOV as usize
}

impl FileDesc {
    #[inline]
    pub fn try_clone(&self) -> io::Result<Self> {
        self.duplicate()
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::read(
                self.as_raw_fd(),
                buf.as_mut_ptr() as *mut c_void,
                cmp::min(buf.len(), READ_LIMIT),
            )
        })?;
        Ok(ret as usize)
    }

    pub fn read_vectored(&self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        let iovecs = crate::sys::io::IoSliceMut::to_iovecs(unsafe { mem::transmute(bufs) });
        let ret = cvt(unsafe {
            libc::readv(
                self.as_raw_fd(),
                iovecs.as_ptr(),
                cmp::min(iovecs.len(), max_iov()) as c_int,
            )
        })?;
        Ok(ret as usize)
    }

    #[inline]
    pub fn is_read_vectored(&self) -> bool {
        true
    }

    pub fn read_to_end(&self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let mut me = self;
        (&mut me).read_to_end(buf)
    }

    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        use libc::pread64;

        cvt(unsafe {
            pread64(
                self.as_raw_fd(),
                buf.as_mut_ptr() as *mut c_void,
                cmp::min(buf.len(), READ_LIMIT),
                offset as i64,
            )
        })
        .map(|n| n as usize)
    }

    pub fn read_buf(&self, mut cursor: BorrowedCursor<'_>) -> io::Result<()> {
        // SAFETY: `cursor.as_mut()` starts with `cursor.capacity()` writable bytes
        let ret = cvt(unsafe {
            libc::read(
                self.as_raw_fd(),
                cursor.as_mut().as_mut_ptr() as *mut libc::c_void,
                cmp::min(cursor.capacity(), READ_LIMIT),
            )
        })?;

        // SAFETY: `ret` bytes were written to the initialized portion of the buffer
        unsafe {
            cursor.advance_unchecked(ret as usize);
        }

        Ok(())
    }

    pub fn read_buf_at(&self, mut cursor: BorrowedCursor<'_>, offset: u64) -> io::Result<()> {
        use libc::pread64;

        // SAFETY: `cursor.as_mut()` starts with `cursor.capacity()` writable bytes
        let ret = cvt(unsafe {
            pread64(
                self.as_raw_fd(),
                cursor.as_mut().as_mut_ptr().cast::<libc::c_void>(),
                cmp::min(cursor.capacity(), READ_LIMIT),
                offset as _, // EINVAL if offset + count overflows
            )
        })?;

        // SAFETY: `ret` bytes were written to the initialized portion of the buffer
        unsafe {
            cursor.advance_unchecked(ret as usize);
        }
        Ok(())
    }

    pub fn read_vectored_at(&self, bufs: &mut [IoSliceMut<'_>], offset: u64) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::preadv(
                self.as_raw_fd(),
                bufs.as_mut_ptr() as *mut libc::iovec as *const libc::iovec,
                cmp::min(bufs.len(), max_iov()) as libc::c_int,
                offset as _,
            )
        })?;
        Ok(ret as usize)
    }

    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::write(
                self.as_raw_fd(),
                buf.as_ptr() as *const c_void,
                cmp::min(buf.len(), READ_LIMIT),
            )
        })?;
        Ok(ret as usize)
    }

    pub fn write_vectored(&self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let iovecs = crate::sys::io::IoSlice::to_iovecs(unsafe { mem::transmute(bufs) });
        let ret = cvt(unsafe {
            libc::writev(
                self.as_raw_fd(),
                iovecs.as_ptr(),
                cmp::min(iovecs.len(), max_iov()) as c_int,
            )
        })?;
        Ok(ret as usize)
    }

    #[inline]
    pub fn is_write_vectored(&self) -> bool {
        true
    }

    pub fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        use libc::pwrite64;

        unsafe {
            cvt(pwrite64(
                self.as_raw_fd(),
                buf.as_ptr() as *const c_void,
                cmp::min(buf.len(), READ_LIMIT),
                offset as i64,
            ))
            .map(|n| n as usize)
        }
    }

    pub fn write_vectored_at(&self, bufs: &[IoSlice<'_>], offset: u64) -> io::Result<usize> {
        let ret = cvt(unsafe {
            libc::pwritev(
                self.as_raw_fd(),
                bufs.as_ptr() as *const libc::iovec,
                cmp::min(bufs.len(), max_iov()) as libc::c_int,
                offset as _,
            )
        })?;
        Ok(ret as usize)
    }

    pub fn set_cloexec(&self) -> io::Result<()> {
        unsafe {
            let previous = cvt(libc::fcntl(self.as_raw_fd(), libc::F_GETFD))?;
            let new = previous | libc::FD_CLOEXEC;
            if new != previous {
                cvt(libc::fcntl(self.as_raw_fd(), libc::F_SETFD, new))?;
            }
            Ok(())
        }
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        unsafe {
            let v = nonblocking as c_int;
            cvt(libc::ioctl(self.as_raw_fd(), libc::FIONBIO, &v))?;
            Ok(())
        }
    }

    pub fn duplicate(&self) -> io::Result<FileDesc> {
        Ok(Self(self.0.try_clone()?))
    }
}

impl<'a> Read for &'a FileDesc {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (**self).read(buf)
    }

    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        (**self).read_buf(cursor)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        (**self).read_vectored(bufs)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        (**self).is_read_vectored()
    }
}

impl AsInner<OwnedFd> for FileDesc {
    #[inline]
    fn as_inner(&self) -> &OwnedFd {
        &self.0
    }
}

impl IntoInner<OwnedFd> for FileDesc {
    fn into_inner(self) -> OwnedFd {
        self.0
    }
}

impl FromInner<OwnedFd> for FileDesc {
    fn from_inner(owned_fd: OwnedFd) -> Self {
        Self(owned_fd)
    }
}

impl AsFd for FileDesc {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for FileDesc {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl IntoRawFd for FileDesc {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl FromRawFd for FileDesc {
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self(unsafe { FromRawFd::from_raw_fd(raw_fd) })
    }
}
