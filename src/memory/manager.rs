use crate::memory::physical::PhysicalFrameAllocator;
use crate::memory::Error;
use crate::memory::Result;
use core::marker::PhantomData;
use core::ptr;
use kstd::sync::{Mutex, MutexGuard};
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::{
    FrameAllocator, FrameDeallocator, Mapper, OffsetPageTable, Page, PageSize, PageTableFlags,
    PhysFrame, Size4KiB,
};
use x86_64::VirtAddr;

static mut MEMORY_MANAGER: Option<
    Mutex<MemoryManager<Size4KiB, OffsetPageTable<'static>, PhysicalFrameAllocator<Size4KiB>>>,
> = None;

pub fn init_memory_manager(
    page_table: OffsetPageTable<'static>,
    physical_frame_allocator: PhysicalFrameAllocator<Size4KiB>,
) {
    unsafe {
        if MEMORY_MANAGER.is_some() {
            panic!("memory manager already initialized");
        }
        let mm = MemoryManager::new(page_table, physical_frame_allocator);
        MEMORY_MANAGER = Some(Mutex::new(mm));
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ZeroFilled {
    Yes,
    No,
}

impl From<ZeroFilled> for bool {
    fn from(x: ZeroFilled) -> Self {
        x == ZeroFilled::Yes
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum UserAccessible {
    Yes,
    No,
}

impl From<UserAccessible> for bool {
    fn from(x: UserAccessible) -> Self {
        x == UserAccessible::Yes
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum MemoryKind {
    ReadOnly,
    Writable,
    Executable,
}

pub struct MemoryManager<S, M, A>
where
    S: PageSize,
{
    page_table: M,
    physical_frame_allocator: A,
    _page_size: PhantomData<S>,
}

impl MemoryManager<Size4KiB, OffsetPageTable<'static>, PhysicalFrameAllocator<Size4KiB>> {
    pub fn lock() -> MutexGuard<
        'static,
        MemoryManager<Size4KiB, OffsetPageTable<'static>, PhysicalFrameAllocator<Size4KiB>>,
    > {
        unsafe { MEMORY_MANAGER.as_ref().unwrap().lock() }
    }
}

impl<S, M, A> MemoryManager<S, M, A>
where
    S: PageSize,
    M: Mapper<S>,
    A: FrameAllocator<S> + FrameAllocator<Size4KiB>, // 4KiB required since page table mapping pages are 4KiB
{
    fn new(page_table: M, physical_frame_allocator: A) -> Self {
        Self {
            page_table,
            physical_frame_allocator,
            _page_size: PhantomData,
        }
    }
}

impl<S, M, A> MemoryManager<S, M, A>
where
    S: PageSize,
    M: Mapper<S>,
    A: FrameAllocator<S> + FrameAllocator<Size4KiB>, // 4KiB required since page table mapping pages are 4KiB
{
    pub fn allocate_and_map_memory(
        &mut self,
        start_addr: VirtAddr,
        page_count: usize,
        memory_kind: MemoryKind,
        user_accessible: UserAccessible,
        zero_filled: ZeroFilled,
    ) -> Result<()> {
        let mut page_table_flags = PageTableFlags::PRESENT | PageTableFlags::NO_EXECUTE;
        if user_accessible.into() {
            page_table_flags |= PageTableFlags::USER_ACCESSIBLE;
        }
        match memory_kind {
            MemoryKind::ReadOnly => {}
            MemoryKind::Writable => page_table_flags |= PageTableFlags::WRITABLE,
            MemoryKind::Executable => page_table_flags.remove(PageTableFlags::NO_EXECUTE),
        };

        self.allocate_and_map_pages(start_addr, page_count, page_table_flags, zero_filled)
    }

    fn allocate_frame(&mut self) -> Result<PhysFrame<S>> {
        self.physical_frame_allocator
            .allocate_frame()
            .ok_or(Error::FrameAllocationFailed)
    }

    fn allocate_and_map_pages(
        &mut self,
        start_addr: VirtAddr,
        page_count: usize,
        flags: PageTableFlags,
        zero_filled: ZeroFilled,
    ) -> Result<()> {
        let start_page = Page::<S>::containing_address(start_addr);
        let end_page = start_page + (page_count - 1) as u64;
        let page_range = Page::range_inclusive(start_page, end_page);

        self.allocate_and_map_page_range(page_range, flags, zero_filled)
    }

    fn allocate_and_map_page_range(
        &mut self,
        page_range: PageRangeInclusive<S>,
        flags: PageTableFlags,
        zero_filled: ZeroFilled,
    ) -> Result<()> {
        for page in page_range {
            let frame = self.allocate_frame()?;
            self.map_frame_to_page(frame, page, flags)?;
            if zero_filled.into() {
                unsafe {
                    ptr::write_bytes(
                        page.start_address().as_mut_ptr::<u8>(),
                        0,
                        page.size() as usize,
                    );
                }
            }
        }
        Ok(())
    }

    fn map_frame_to_page(
        &mut self,
        frame: PhysFrame<S>,
        page: Page<S>,
        flags: PageTableFlags,
    ) -> Result<()> {
        let pt = &mut self.page_table;
        let fa = &mut self.physical_frame_allocator;
        unsafe { pt.map_to::<A>(page, frame, flags, fa) }?.flush();
        Ok(())
    }
}

impl<S, M, A> MemoryManager<S, M, A>
where
    S: PageSize,
    M: Mapper<S>,
    A: FrameDeallocator<S>,
{
    pub fn deallocate_and_unmap_page(&mut self, addr: VirtAddr) -> Result<()> {
        let page = Page::<S>::containing_address(addr);
        let (frame, flush) = self.page_table.unmap(page)?;
        flush.flush();
        unsafe { self.physical_frame_allocator.deallocate_frame(frame) };
        Ok(())
    }
}
