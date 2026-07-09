use crate::cell::UnsafeCell;
use crate::mem::MaybeUninit;
use crate::pin::Pin;
use crate::qumulo::bindings::{
    rust_sys_parker, rust_sys_parker_destroy, rust_sys_parker_init, rust_sys_parker_park,
    rust_sys_parker_park_timeout, rust_sys_parker_unpark,
};
use crate::time::Duration;

impl rust_sys_parker {
    fn new() -> Self {
        let mut parker = MaybeUninit::<rust_sys_parker>::uninit();
        unsafe {
            rust_sys_parker_init(parker.as_mut_ptr());
            parker.assume_init()
        }
    }
}

impl Drop for rust_sys_parker {
    fn drop(&mut self) {
        unsafe {
            rust_sys_parker_destroy(self);
        }
    }
}

pub struct Parker(UnsafeCell<rust_sys_parker>);

unsafe impl Send for Parker {}
unsafe impl Sync for Parker {}

impl Parker {
    pub unsafe fn new_in_place(parker: *mut Parker) {
        unsafe { parker.write(Self(UnsafeCell::new(rust_sys_parker::new()))) }
    }

    pub unsafe fn park(self: Pin<&Self>) {
        unsafe { rust_sys_parker_park(self.0.get()) };
    }

    pub unsafe fn park_timeout(self: Pin<&Self>, dur: Duration) {
        unsafe { rust_sys_parker_park_timeout(self.0.get(), dur.as_nanos()) };
    }

    pub fn unpark(self: Pin<&Self>) {
        unsafe { rust_sys_parker_unpark(self.0.get()) };
    }
}
