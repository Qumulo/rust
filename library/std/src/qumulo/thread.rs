use crate::cell::UnsafeCell;
use crate::ffi::CStr;
use crate::mem::{self, MaybeUninit};
use crate::num::NonZeroUsize;
use crate::qumulo::bindings::{
    fn_once_ptr, rust_sys_event, rust_sys_event_destroy, rust_sys_event_init, rust_sys_event_set,
    rust_sys_event_wait, rust_sys_thread_current_os_id, rust_sys_thread_new,
    rust_sys_thread_set_name, rust_sys_thread_sleep, rust_sys_thread_yield_now,
};
use crate::sync::Arc;
use crate::time::{Duration, Instant};
use crate::{io, ptr};

pub const DEFAULT_MIN_STACK_SIZE: usize = 0;

pub struct Thread {
    event: Arc<UnsafeCell<rust_sys_event>>,
}

impl rust_sys_event {
    pub fn new() -> Self {
        unsafe {
            let mut event = MaybeUninit::<rust_sys_event>::uninit();
            rust_sys_event_init(event.as_mut_ptr());
            event.assume_init()
        }
    }
}

impl Drop for rust_sys_event {
    fn drop(&mut self) {
        unsafe {
            rust_sys_event_destroy(&mut *self);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_sys_thread_trampoline(f: fn_once_ptr) {
    unsafe {
        let f: *mut dyn FnOnce() = mem::transmute(f);
        Box::from_raw(f)();
    }
}

impl Thread {
    pub unsafe fn new(
        _stack: usize,
        name: Option<&str>,
        f: Box<dyn FnOnce() + '_>,
    ) -> io::Result<Thread> {
        let my_event = Arc::new(UnsafeCell::new(rust_sys_event::new()));
        let their_event = my_event.clone();

        let closure = Box::new(move || {
            f();
            unsafe { rust_sys_event_set(their_event.get()) };
        });

        let closure: *mut dyn FnOnce() = Box::into_raw(closure);
        let name_ptr = name.map(str::as_ptr).unwrap_or(ptr::null());
        let name_len = name.map(str::len).unwrap_or(0);
        unsafe { rust_sys_thread_new(name_ptr, name_len, mem::transmute(closure)) };
        Ok(Thread { event: my_event })
    }

    pub fn join(self) {
        unsafe {
            rust_sys_event_wait(self.event.get());
        }
    }
}

/// We don't use this Guard type anywere because we don't use Rust's stack overflow handling code.
/// Some parts of the standard library still rely on calling these functions and the Guard type to
/// exist, so we just make it unit.
#[allow(dead_code)]
pub mod guard {
    pub type Guard = ();

    pub unsafe fn current() -> Option<Guard> {
        Some(())
    }

    pub unsafe fn init() -> Option<Guard> {
        Some(())
    }

    pub unsafe fn deinit() {}
}

pub fn available_parallelism() -> io::Result<NonZeroUsize> {
    match unsafe { libc::sysconf(libc::_SC_NPROCESSORS_ONLN) } {
        -1 => Err(io::Error::last_os_error()),
        0 => Err(io::const_error!(
            io::ErrorKind::NotFound,
            &"The number of hardware threads is not known for the target platform",
        )),
        cpus => Ok(unsafe { NonZeroUsize::new_unchecked(cpus as usize) }),
    }
}

pub fn current_os_id() -> Option<u64> {
    let mut id = 0;
    if unsafe { rust_sys_thread_current_os_id(&mut id) } {
        Some(id)
    } else {
        None
    }
}

pub fn set_name(name: &CStr) {
    unsafe {
        rust_sys_thread_set_name(name.as_ptr());
    }
}

pub fn sleep(dur: Duration) {
    let sec = dur.as_secs();
    let nsec = dur.subsec_nanos();
    unsafe {
        rust_sys_thread_sleep(sec, nsec);
    }
}

pub fn sleep_until(deadline: Instant) {
    // XXX cwallace: we could be more precise (use e.g. `alarm_clock`), but as of Rust 1.90.0
    // `thread::sleep_until` is unstable and most officially supported platforms just dispatch
    // to `thread::sleep` anyway.
    let now = Instant::now().into_inner();
    if let Some(delay) = deadline.into_inner().checked_sub_instant(&now) {
        sleep(delay);
    }
}

pub fn yield_now() {
    unsafe {
        rust_sys_thread_yield_now();
    }
}
