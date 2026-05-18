use crate::alloc::boxed::Box;
use crate::register::sio::SPINLOCK_OFFSET;
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// Wrapped `UnsafeCell` to allow local implementation
pub struct Wrapped<T> {
    inner: UnsafeCell<T>,
    spinlock: *mut usize,
    number: usize,
}

impl<T> Wrapped<T> {
    /// Wrap a new type `T`
    pub const fn new(inner: T, offset: usize) -> Wrapped<T> {
        let spinlock = (SPINLOCK_OFFSET + offset * 4) as *mut usize;
        Wrapped {
            inner: UnsafeCell::new(inner),
            spinlock,
            number: offset,
        }
    }
    /// Get a mutable reference to `T`
    fn get(&self) -> &mut T {
        unsafe { &mut *self.inner.get() }
    }

    pub fn lock(&self) -> Guard<'_, T> {
        self.lock_hw();
        Guard {
            inner: self.get(),
            spinlock: self.spinlock,
        }
    }
    fn unlock_hw(&self) {
        unsafe {
            self.spinlock.write_volatile(1);
        }
    }
    fn lock_hw(&self) {
        unsafe {
            while self.spinlock.read_volatile() == 0 {
                core::hint::spin_loop();
            }
        };
    }
}

unsafe impl<T: Send> Sync for Wrapped<T> {}

impl Wrapped<Option<Box<dyn Fn() + Send>>> {
    /// Call the inner function
    pub fn set(&self, f: impl Fn() + 'static + Send) {
        let mut guard = self.lock();
        *guard = Some(Box::new(f));
    }
}

pub struct Guard<'a, T> {
    inner: &'a mut T,
    spinlock: *mut usize,
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.unlock_hw();
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}
impl<T> Guard<'_, T> {
    fn unlock_hw(&self) {
        unsafe {
            self.spinlock.write_volatile(1);
        }
    }
}
