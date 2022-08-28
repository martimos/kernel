use crate::memory::allocator::backend::MemoryBackend;
use crate::memory::manager::{MemoryKind, MemoryManager, UserAccessible};
use crate::memory::{DefaultPageSize, Result};
use core::marker::PhantomData;
use x86_64::structures::paging::{Page, PageSize};
use x86_64::VirtAddr;

pub struct NoUnmap;
pub struct WithUnmap;

pub struct LazyPageMappingBackend<S, U>
where
    S: PageSize,
{
    _page_size: PhantomData<S>,
    _unmap_strategy: PhantomData<U>,
}

impl<S, U> LazyPageMappingBackend<S, U>
where
    S: PageSize,
{
    pub const fn new() -> Self {
        Self {
            _page_size: PhantomData,
            _unmap_strategy: PhantomData,
        }
    }
}

impl MemoryBackend for LazyPageMappingBackend<DefaultPageSize, NoUnmap> {
    fn memory_allocated(&mut self, addr: *const u8, size: usize) -> Result<()> {
        let page_range = Page::range(
            Page::containing_address(VirtAddr::new_truncate(addr as u64)),
            Page::containing_address(VirtAddr::new_truncate(addr as u64 + size as u64 - 1)) + 1,
        );
        MemoryManager::lock().ensure_is_mapped(page_range, MemoryKind::Writable, UserAccessible::No)
    }

    fn memory_deallocated(&mut self, _addr: *const u8, _size: usize) -> Result<()> {
        Ok(())
    }
}
