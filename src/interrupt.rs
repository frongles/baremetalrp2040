use crate::alloc::boxed::Box;
use core::cell::UnsafeCell;
pub struct InterruptSlot {
    inner: UnsafeCell<Option<Box<dyn Fn()>>>,
}
unsafe impl Sync for InterruptSlot {}

impl InterruptSlot {
    pub const fn new() -> InterruptSlot {
        InterruptSlot {
            inner: UnsafeCell::new(None),
        }
    }
    pub fn call(&self) {
        unsafe {
            if let Some(f) = &*self.inner.get() {
                f()
            }
        }
    }
    pub fn set(&self, f: impl Fn() + 'static) {
        unsafe {
            *self.inner.get() = Some(Box::new(f));
        }
    }
}
