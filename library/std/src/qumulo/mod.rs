#![allow(missing_debug_implementations, missing_docs)]

#[allow(
    dead_code,
    deref_nullptr,
    improper_ctypes,
    missing_unsafe_on_extern,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unsafe_attr_outside_unsafe,
    unused_imports
)]
mod bindings {
    use crate::os::raw;
    include!("bindings.rs");
}

pub mod alloc;
pub mod condvar;
pub mod fd;
pub mod io_slice;
pub mod mutex;
pub mod random;
pub mod stdio;
pub mod thread;
pub mod thread_local_guard;
pub mod thread_local_key;
pub mod thread_parking;
