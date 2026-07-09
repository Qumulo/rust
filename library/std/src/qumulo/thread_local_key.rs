// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)] // not used on all platforms

use crate::mem;
use crate::qumulo::bindings::{
    rust_sys_thread_local_create, rust_sys_thread_local_destroy, rust_sys_thread_local_get,
    rust_sys_thread_local_set,
};

pub type Key = u32;

#[inline]
pub fn create(dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    unsafe { rust_sys_thread_local_create(mem::transmute(dtor)) }
}

#[inline]
pub unsafe fn set(key: Key, value: *mut u8) {
    unsafe { rust_sys_thread_local_set(key, value as *mut _) };
}

#[inline]
pub unsafe fn get(key: Key) -> *mut u8 {
    unsafe { rust_sys_thread_local_get(key) as *mut u8 }
}

#[inline]
pub unsafe fn destroy(key: Key) {
    unsafe { rust_sys_thread_local_destroy(key) };
}
