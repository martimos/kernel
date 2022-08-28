use crate::memory::allocator::align_up;
use crate::memory::allocator::backend::MemoryBackend;
use crate::memory::heap::Locked;
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr,
};

pub struct BumpAllocator<M>
where
    M: MemoryBackend,
{
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
    memory_backend: M,
}

impl<M> BumpAllocator<M>
where
    M: MemoryBackend,
{
    /// Creates a new empty bump allocator.
    pub const fn new(memory_backend: M) -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
            memory_backend,
        }
    }

    /// Initializes the bump allocator with the given heap bounds.
    ///
    /// # Safety
    ///
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.
    pub unsafe fn init(&mut self, heap_start: *mut u8, heap_size: usize) {
        self.heap_start = heap_start as usize;
        self.heap_end = heap_start as usize + heap_size;
        self.next = heap_start as usize;
    }
}

unsafe impl<M> GlobalAlloc for Locked<BumpAllocator<M>>
where
    M: MemoryBackend,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut bump = self.lock(); // get a mutable reference

        let alloc_start = align_up(bump.next, layout.align());
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            ptr::null_mut() // out of memory
        } else {
            bump.next = alloc_end;
            bump.allocations += 1;
            bump.memory_backend
                .memory_allocated(alloc_start as *const u8, alloc_end - alloc_start)
                .expect("memory allocation via backend failed");
            alloc_start as *mut u8
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        /*
        Since we don't really deallocate, we don't need to deallocate via memory backend.
         */

        let mut bump = self.lock(); // get a mutable reference

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
