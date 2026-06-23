use crate::util::Mutex;
use core::{alloc::GlobalAlloc, ops::DerefMut, ptr};

/// Bump Allocator
///
/// Sequentially allocates memory.
///
/// On deallocation:
///  - if the memory to be deallocated is the last memory that was
///    allocated, it is reclaimed
///  - If after the deallocation no more memory is allocated, reclaim all
///    memory
///  - Else do nothing
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
pub static ALLOCATOR: Mutex<Allocator> = Mutex::new(Allocator::new());

impl Mutex<Allocator> {
    pub fn init(&self, sheap: usize, eheap: usize) {
        let mut guard = self.lock();
        unsafe { guard.deref_mut().init(sheap, eheap) }
    }
}

unsafe impl GlobalAlloc for Mutex<Allocator> {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut guard = self.lock();
        let allocator = guard.deref_mut();
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
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        let mut guard = self.lock();
        let allocator = guard.deref_mut();
        if allocator.last_alloced == ptr as usize {
            allocator.next = ptr as usize;
        }
        allocator.allocations -= 1;
        if allocator.allocations == 0 {
            allocator.next = allocator.heap_start
        }
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
