//! thread_local::guard::enable() allows registering some function to be performed after all
//! thread local storage destructors have run (during a thread (aka task) exit).
//!
//! Without overriding, the function that we'd be using is
//! library/std/src/sys/thread_local/guard/key.rs:enable(). Specifically the
//! `not(target_thread_local)` variant.
//!
//! Without overriding, we encounter memory leaks on thread/task exit, due to std wanting to
//! allow all TLS `Drop` destructors to be able to access `Thread::current()`. It hides the
//! final cleanup of the current thread behind another TLS variable which is registered first
//! and destroyed last.
//!
//! And because TLS destructors are not guarunteed to run on unix platforms, the final drop
//! doesn't occur at task exit.
//!
//! So we define our own enabler that simply registers the thread cleanup as a destructor
//! function.

#![allow(dead_code)] // not used on all platforms

use crate::ptr;
use crate::sys::thread_local::key::{set, LazyKey};

pub fn enable() {
    static DTORS: LazyKey = LazyKey::new(Some(run));

    // Setting the key value to something other than NULL will result in the
    // destructor being run at task exit.
    unsafe {
        set(DTORS.force(), ptr::without_provenance_mut(1));
    }

    #[allow(unused_unsafe)]
    unsafe extern "C" fn run(_: *mut u8) {
        unsafe {
            // The only thing to cleanup is the current thread. It's possible other cleanups will
            // be added in the future, so if you're updating rustc and get some ASAN failures on
            // task exit, take a peak at library/std/src/sys/thread_local/guard/key.rs:enable() to
            // see if anything new is being cleaned up.
            crate::rt::thread_cleanup();
        }
    }
}
