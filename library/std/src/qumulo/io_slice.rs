// Adapted from the Rustc std library source, which is licensed under MIT and Apache.

use libc::iovec;

use crate::marker::PhantomData;
use crate::os::fd::{AsFd, AsRawFd};
use crate::qumulo::bindings::{
    self, rust_sys_io_slice, rust_sys_io_slice_bytes, rust_sys_io_slice_bytes_mut,
    rust_sys_io_slice_copy_init, rust_sys_io_slice_length, rust_sys_io_slice_to_iovecs,
    rust_sys_io_slice_to_iovecs_mut,
};
use crate::{mem, ptr, slice};

const ZERO_IOVEC: iovec = iovec {
    iov_base: ptr::null_mut(),
    iov_len: 0,
};

#[derive(Copy)]
#[repr(transparent)]
pub struct IoSlice<'a> {
    inner: rust_sys_io_slice,
    _p: PhantomData<&'a [u8]>,
}

impl<'a> Clone for IoSlice<'a> {
    fn clone(&self) -> Self {
        let mut slice = mem::MaybeUninit::<rust_sys_io_slice>::uninit();
        unsafe {
            rust_sys_io_slice_copy_init(slice.as_mut_ptr(), &self.inner);
        }
        Self {
            inner: unsafe { slice.assume_init() },
            _p: PhantomData,
        }
    }
}

impl<'a> IoSlice<'a> {
    #[inline]
    pub fn new(_buf: &'a [u8]) -> IoSlice<'a> {
        unimplemented!("Use SgVec instead to create IoSlices")
    }

    #[inline]
    pub fn advance(&mut self, _n: usize) {
        unimplemented!()
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [u8] {
        unsafe {
            slice::from_raw_parts(
                rust_sys_io_slice_bytes(&self.inner),
                rust_sys_io_slice_length(&self.inner) as usize,
            )
        }
    }

    // Custom addition for the qumulo enviornment.
    pub(crate) fn to_iovecs(slices: &[Self]) -> Vec<iovec> {
        let mut result = vec![ZERO_IOVEC; slices.len()];
        unsafe {
            rust_sys_io_slice_to_iovecs(
                slices.as_ptr() as *const rust_sys_io_slice,
                slices.len() as u64,
                result.as_mut_ptr() as *mut bindings::iovec,
            )
        };
        result
    }
}

#[repr(transparent)]
pub struct IoSliceMut<'a> {
    inner: rust_sys_io_slice,
    _p: PhantomData<&'a mut [u8]>,
}

impl<'a> IoSliceMut<'a> {
    #[inline]
    pub fn new(_buf: &'a mut [u8]) -> IoSliceMut<'a> {
        unimplemented!("Use SgVec instead to create IoSliceMuts")
    }

    #[inline]
    pub fn advance(&mut self, _n: usize) {
        unimplemented!()
    }

    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                rust_sys_io_slice_bytes(&self.inner),
                rust_sys_io_slice_length(&self.inner) as usize,
            )
        }
    }

    #[inline]
    pub fn into_slice(mut self) -> &'a mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(
                rust_sys_io_slice_bytes_mut(&mut self.inner),
                rust_sys_io_slice_length(&self.inner) as usize,
            )
        }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(
                rust_sys_io_slice_bytes_mut(&mut self.inner),
                rust_sys_io_slice_length(&self.inner) as usize,
            )
        }
    }

    // Custom addition for the qumulo enviornment.
    pub(crate) fn to_iovecs(slices: &mut [Self]) -> Vec<iovec> {
        let mut result = vec![ZERO_IOVEC; slices.len()];
        unsafe {
            rust_sys_io_slice_to_iovecs_mut(
                slices.as_mut_ptr() as *mut rust_sys_io_slice,
                slices.len() as u64,
                result.as_mut_ptr() as *mut bindings::iovec,
            )
        };
        result
    }
}

pub fn is_terminal(fd: &impl AsFd) -> bool {
    let fd = fd.as_fd();
    unsafe { libc::isatty(fd.as_raw_fd()) != 0 }
}
