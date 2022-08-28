use crate::memory::allocator::backend::lazy_page::{LazyPageMappingBackend, NoUnmap};
use crate::memory::allocator::bump::BumpAllocator;
use crate::memory::heap::Locked;
use crate::memory::span::KBUFFER;
use crate::memory::{DefaultPageSize, Result};
use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

static mut KBUFFER_HEAP: Locked<BumpAllocator<LazyPageMappingBackend<DefaultPageSize, NoUnmap>>> =
    Locked::new(BumpAllocator::new(LazyPageMappingBackend::new()));

pub(in crate::memory) fn init_kbuffer_heap() -> Result<()> {
    /*
    Since we use the LazyPageMappingBackend, we don't need to map any of the pages of the heap here.
     */

    unsafe {
        KBUFFER_HEAP
            .lock()
            .init(KBUFFER.as_mut_ptr::<u8>(), KBUFFER.len());
    }

    Ok(())
}

pub struct KBuffer {
    start: NonNull<u8>,
    len: usize,
    allocation_layout: Layout, // used for deallocation
}

impl KBuffer {
    /// Allocates a buffer with size 0.
    pub fn empty() -> Self {
        Self::allocate(0)
    }

    /// Allocates a buffer with the given size and an alignment of 64 bytes.
    pub fn allocate(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 64).unwrap();
        Self::allocate_from_layout(layout)
    }

    /// Allocates a buffer with the given layout.
    pub fn allocate_from_layout(layout: Layout) -> Self {
        let start = unsafe { KBUFFER_HEAP.alloc_zeroed(layout) };
        KBuffer {
            start: NonNull::new(start)
                .expect("did KBUFFER_HEAP run out of memory? (alloc returned 0)"),
            len: layout.size(),
            allocation_layout: layout,
        }
    }

    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.start.cast::<T>().as_ptr()
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.start.cast::<T>().as_ptr()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Drop for KBuffer {
    fn drop(&mut self) {
        unsafe {
            KBUFFER_HEAP.dealloc(self.start.as_ptr(), self.allocation_layout);
        }
    }
}

impl AsRef<[u8]> for KBuffer {
    fn as_ref(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.start.as_ptr(), self.len) }
    }
}

impl AsMut<[u8]> for KBuffer {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.start.as_ptr(), self.len) }
    }
}
