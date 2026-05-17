use core::{alloc::GlobalAlloc, cell::UnsafeCell, ptr};

pub struct Wrapped<T> {
    inner: UnsafeCell<T>,
}
impl<T> Wrapped<T> {
    const fn new(inner: T) -> Wrapped<T> {
        Wrapped {
            inner: UnsafeCell::new(inner),
        }
    }
    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.inner.get() }
    }
}
unsafe impl<T> Sync for Wrapped<T> {}

pub struct Allocator {
    heap_start: usize,
    heap_end: usize,
    last_alloced: usize,
    next: usize,
    allocations: usize,
}

impl Allocator {
    pub const fn new() -> Allocator {
        Allocator {
            heap_start: 0,
            heap_end: 0,
            last_alloced: 0,
            next: 0,
            allocations: 0,
        }
    }
    pub unsafe fn init(&mut self, heap_start: usize, heap_end: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_end;
        self.next = heap_start;
    }
}

#[global_allocator]
pub static ALLOCATOR: Wrapped<Allocator> = Wrapped::new(Allocator::new());

unsafe impl GlobalAlloc for Wrapped<Allocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let allocator = self.get();
        let alloc_start = align_up(allocator.next, layout.align());
        let alloc_end = alloc_start + layout.size();
        if alloc_end > allocator.heap_end {
            ptr::null_mut()
        } else {
            allocator.allocations += 1;
            allocator.next = alloc_end;
            allocator.last_alloced = alloc_start;
            alloc_start as *mut u8
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let allocator = self.get();
        if allocator.last_alloced == ptr as usize {
            allocator.next = ptr as usize;
            allocator.allocations -= 1;
        }
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
