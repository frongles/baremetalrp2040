use crate::{alloc::boxed::Box, register::sio::spinlock};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{Ordering, fence};

/// A mutex around type T, using spinlock N
///
/// note that this mutex implementation holds the spinlock
/// for the entire lifetime of the guard, and so great care must be taken when
/// taking the lock, if anything else in the entire program uses the same spinlock
///
/// It's recommended to wrap the mutex around a [`bool`] and keep lock contention
/// as minimal as possible until I implement an AtomicBool.

pub struct Mutex<T> {
    inner: UnsafeCell<T>,
    taken: UnsafeCell<bool>,
    spinlock: SpinLock<31>,
}

impl<T> Mutex<T> {
    /// Wrap a new type `T`
    pub const fn new(inner: T) -> Mutex<T> {
        Mutex {
            inner: UnsafeCell::new(inner),
            taken: UnsafeCell::new(false),
            spinlock: SpinLock::new(),
        }
    }

    pub fn lock(&self) -> Guard<'_, T> {
        loop {
            if unsafe { !*self.taken.get() } {
                self.spinlock.lock();
                if unsafe { !*self.taken.get() } {
                    unsafe { *self.taken.get() = true }
                    self.spinlock.unlock();
                    break;
                }
                self.spinlock.unlock();
            }
        }
        fence(Ordering::Acquire);
        Guard {
            inner: unsafe { &mut *self.inner.get() },
            taken: unsafe { &mut *self.taken.get() },
            spinlock: self.spinlock,
        }
    }
}

unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<const N: usize> Send for SpinLock<N> {}

impl Mutex<Option<Box<dyn Fn() + Send>>> {
    pub fn set(&self, f: impl Fn() + 'static + Send) {
        let mut guard = self.lock();
        *guard = Some(Box::new(f));
    }
}

pub struct Guard<'a, T> {
    inner: &'a mut T,
    taken: &'a mut bool,
    spinlock: SpinLock<31>,
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.spinlock.lock();
        *self.taken = false;
        self.spinlock.unlock();
        fence(Ordering::Release);
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

#[derive(Clone, Copy)]
pub struct SpinLock<const N: usize> {
    inner: *mut usize,
}

impl<const N: usize> SpinLock<N> {
    /// Get access to the Nth spinlock
    pub const fn new() -> SpinLock<N> {
        // This const block is evaluated at compile time no matter
        // how `new` itself is called (const or runtime context).
        const { assert!(N < 32, "spinlock index out of range") };

        SpinLock {
            inner: spinlock(N) as *mut usize,
        }
    }
    pub fn lock(&self) {
        unsafe {
            while self.inner.read_volatile() == 0 {
                core::hint::spin_loop();
            }
        };
    }
    pub fn unlock(&self) {
        unsafe {
            self.inner.write_volatile(1);
        }
    }
}
