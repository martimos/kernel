use crate::memory::allocator::backend::already_mapped::MemoryAlreadyMappedBackend;
use crate::memory::allocator::fixed_size_block::FixedSizeBlockAllocator;
use crate::memory::manager::{MemoryKind, MemoryManager, UserAccessible};
use crate::memory::{span, Error};
use kstd::sync::{Mutex, MutexGuard};
use x86_64::structures::paging::Size4KiB;

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator<MemoryAlreadyMappedBackend>> =
    Locked::new(FixedSizeBlockAllocator::new(MemoryAlreadyMappedBackend));

pub(in crate::memory) fn init_heap() -> Result<(), Error> {
    const HEAP_START: *mut u8 = span::HEAP.as_mut_ptr::<u8>();
    const HEAP_SIZE: usize = span::HEAP.len();

    let page_range = span::HEAP.as_page_range::<Size4KiB>();
    // allocate writable and zeroed kernel memory
    MemoryManager::lock().allocate_and_map_page_range(
        page_range,
        MemoryKind::Writable,
        UserAccessible::No,
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
