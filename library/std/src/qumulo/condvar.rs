// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::cell::UnsafeCell;
use crate::pin::Pin;
use crate::qumulo::bindings::{
    rust_sys_condvar, rust_sys_condvar_destroy, rust_sys_condvar_init, rust_sys_condvar_notify_all,
    rust_sys_condvar_notify_one, rust_sys_condvar_wait, rust_sys_condvar_wait_timeout,
};
use crate::qumulo::mutex::{self, Mutex};
use crate::sys::sync::OnceBox;
use crate::time::Duration;

impl rust_sys_condvar {
    const fn new() -> Self {
        Self { inner: [0; 12] }
    }
}

struct AllocatedCondvar(UnsafeCell<rust_sys_condvar>);

impl AllocatedCondvar {
    fn new() -> Self {
        Self(UnsafeCell::new(rust_sys_condvar::new()))
    }

    unsafe fn init(&mut self) {
        unsafe { rust_sys_condvar_init(self.0.get()) };
    }

    unsafe fn destroy(&self) {
        unsafe { rust_sys_condvar_destroy(self.0.get()) };
    }

    fn get(&self) -> *mut rust_sys_condvar {
        self.0.get()
    }
}

impl Drop for AllocatedCondvar {
    fn drop(&mut self) {
        unsafe { self.destroy() }
    }
}

pub struct Condvar {
    inner: OnceBox<AllocatedCondvar>,
}

unsafe impl Send for Condvar {}
unsafe impl Sync for Condvar {}

impl Condvar {
    pub const fn new() -> Condvar {
        Condvar {
            inner: OnceBox::new(),
        }
    }

    #[inline]
    fn inner(&self) -> Pin<&AllocatedCondvar> {
        self.inner.get_or_init(|| {
            let mut condvar = Box::pin(AllocatedCondvar::new());
            unsafe { condvar.init() };
            condvar
        })
    }

    #[inline]
    pub fn notify_one(&self) {
        unsafe {
            rust_sys_condvar_notify_one(self.inner().get_ref().get());
        }
    }

    #[inline]
    pub fn notify_all(&self) {
        unsafe {
            rust_sys_condvar_notify_all(self.inner().get_ref().get());
        }
    }

    #[inline]
    pub unsafe fn wait(&self, mutex: &Mutex) {
        unsafe { rust_sys_condvar_wait(self.inner().get_ref().get(), mutex::raw(mutex)) };
    }

    #[inline]
    pub unsafe fn wait_timeout(&self, mutex: &Mutex, dur: Duration) -> bool {
        let sec = dur.as_secs();
        let nsec = dur.subsec_nanos();
        unsafe {
            rust_sys_condvar_wait_timeout(
                self.inner().get_ref().get(),
                mutex::raw(mutex),
                sec,
                nsec,
            )
        }
    }
}
