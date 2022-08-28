use crate::memory::allocator::backend::lazy_page::{LazyPageMappingBackend, NoUnmap};
use crate::memory::allocator::bump::BumpAllocator;
use crate::memory::heap::Locked;
use crate::memory::span::KBUFFER;
use crate::memory::{DefaultPageSize, Result};
use core::alloc::{GlobalAlloc, Layout};

static mut KBUFFER_HEAP: Locked<BumpAllocator<LazyPageMappingBackend<DefaultPageSize, NoUnmap>>> =
    Locked::new(BumpAllocator::new(LazyPageMappingBackend::new()));

pub fn init_kbuffer_heap() -> Result<()> {
    // let page_range = KBUFFER.as_page_range::<Size4KiB>();
    //
    // MemoryManager::lock().allocate_and_map_page_range(
    //     page_range,
    //     MemoryKind::Writable,
    //     UserAccessible::No,
    // )?;

    unsafe {
        KBUFFER_HEAP
            .lock()
            .init(KBUFFER.as_mut_ptr::<u8>(), KBUFFER.len());
    }

    Ok(())
}

pub struct KBuffer {
    start: *mut u8,
    allocation_layout: Layout, // used for deallocation
    empty: bool,
}

impl KBuffer {
    pub fn empty() -> Self {
        Self {
            start: core::ptr::null_mut::<u8>(),
            allocation_layout: Layout::from_size_align(0, 1).unwrap(),
            empty: true,
        }
    }

    pub fn allocate(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 64).unwrap();
        Self::allocate_from_layout(layout)
    }

    pub fn allocate_from_layout(layout: Layout) -> Self {
        let start = unsafe { KBUFFER_HEAP.alloc_zeroed(layout) };
        KBuffer {
            start,
            allocation_layout: layout,
            empty: false,
        }
    }

    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.start as *mut T
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.start as *const T
    }
}

impl Drop for KBuffer {
    fn drop(&mut self) {
        if self.empty {
            return;
        }
        unsafe {
            KBUFFER_HEAP.dealloc(self.start, self.allocation_layout);
        }
    }
}
