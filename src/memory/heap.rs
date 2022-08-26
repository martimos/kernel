use crate::memory::allocator::fixed_size_block::FixedSizeBlockAllocator;
use crate::memory::manager::{MemoryKind, MemoryManager, UserAccessible, ZeroFilled};
use crate::memory::{span, Error};
use kstd::sync::{Mutex, MutexGuard};
use x86_64::structures::paging::{Page, Size4KiB};
use x86_64::VirtAddr;

pub const HEAP_START: *mut u8 = span::HEAP.as_mut_ptr::<u8>();
pub const HEAP_SIZE: usize = span::HEAP.len();

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

pub fn init_heap() -> Result<(), Error> {
    let heap_start = VirtAddr::new(HEAP_START as u64);
    let page_range = {
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::<Size4KiB>::containing_address(heap_start);
        let heap_end_page = Page::<Size4KiB>::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };
    let page_count = page_range.count();
    // allocate writable and zeroed kernel memory
    MemoryManager::lock().allocate_and_map_memory(
        heap_start,
        page_count,
        MemoryKind::Writable,
        UserAccessible::No,
        ZeroFilled::Yes,
    )?;

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

/// A wrapper around spin::Mutex to permit trait implementations.
pub struct Locked<A> {
    inner: Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> MutexGuard<A> {
        self.inner.lock()
    }
}
