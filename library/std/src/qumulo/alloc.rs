//! Override the system global allocator to use our own custom allocator inside the qumulo env,
//! instead of using malloc.
//!
//! Ideally, we would use the `#[global_allocator]` attribute in the qumulo `core` library to do
//! this. But we can't, as that only works when rustc drives the linking itself, while qumulo rust
//! is linked outside of rustc.
//!
//! Previously (prior to rust 1.87), we defined the following weak symbols inside `core` that would
//! provide global allocator functionality:
//!
//! ```
//! __rust_alloc
//! __rust_alloc_zeroed
//! __rust_dealloc
//! __rust_realloc
//! ```
//!
//! These symbols are the same ones that rustc would produce during the final linking step.
//! Unfortunatly, this PR (https://github.com/rust-lang/rust/pull/127173) started mangling those
//! symbols, requiring an alternative method of overriding the allocator.
//!
//! To truly fix this, we require the -Zemit-code-for-final-artifact-to-link feature to land.
//! (https://github.com/rust-lang/compiler-team/issues/858). This would allow us to emit the rust
//! allocator shim directly as a compilation artifact, which could be used with our external linker.

// Adapted from the Rustc std library source, which is licensed under MIT and Apache.
//
// Specifically from 1.82 @ rust/library/std/src/sys/alloc/unix.rs

use crate::alloc::{GlobalAlloc, Layout, System};
use crate::qumulo::bindings::{rust_sys_memory_allocate, rust_sys_memory_deallocate};

#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { rust_sys_memory_allocate(layout.size() as u64, layout.align() as u64) as *mut u8 }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            rust_sys_memory_deallocate(
                layout.size() as u64,
                layout.align() as u64,
                ptr as *mut libc::c_void,
            )
        }
    }
}
