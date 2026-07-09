use crate::cell::UnsafeCell;
use crate::pin::Pin;
use crate::qumulo::bindings::{
    rust_sys_mutex, rust_sys_mutex_destroy, rust_sys_mutex_init, rust_sys_mutex_lock,
    rust_sys_mutex_try_lock, rust_sys_mutex_unlock,
};
use crate::sys::sync::OnceBox;

impl rust_sys_mutex {
    const fn new() -> Self {
        Self { inner: [0; 12] }
    }
}

struct AllocatedMutex(UnsafeCell<rust_sys_mutex>);

impl AllocatedMutex {
    fn new() -> Self {
        Self(UnsafeCell::new(rust_sys_mutex::new()))
    }

    unsafe fn init(&mut self) {
        unsafe { rust_sys_mutex_init(self.0.get()) };
    }

    unsafe fn destroy(&self) {
        unsafe { rust_sys_mutex_destroy(self.0.get()) };
    }

    fn get(&self) -> *mut rust_sys_mutex {
        self.0.get()
    }
}

impl Drop for AllocatedMutex {
    fn drop(&mut self) {
        unsafe { self.destroy() }
    }
}

pub struct Mutex {
    inner: OnceBox<AllocatedMutex>,
}

#[inline]
pub unsafe fn raw(m: &Mutex) -> *mut rust_sys_mutex {
    m.inner().get()
}

unsafe impl Send for Mutex {}
unsafe impl Sync for Mutex {}

#[allow(dead_code)] // sys isn't exported yet
impl Mutex {
    pub const fn new() -> Mutex {
        Mutex {
            inner: OnceBox::new(),
        }
    }

    fn inner(&self) -> Pin<&AllocatedMutex> {
        self.inner.get_or_init(|| {
            let mut mutex = Box::pin(AllocatedMutex::new());
            unsafe { mutex.init() };
            mutex
        })
    }

    #[inline]
    pub unsafe fn lock(&self) {
        unsafe { rust_sys_mutex_lock(self.inner().get_ref().get()) };
    }

    #[inline]
    pub unsafe fn unlock(&self) {
        unsafe { rust_sys_mutex_unlock(self.inner().get_ref().get()) };
    }

    #[inline]
    pub unsafe fn try_lock(&self) -> bool {
        unsafe { rust_sys_mutex_try_lock(self.inner().get_ref().get()) }
    }
}
