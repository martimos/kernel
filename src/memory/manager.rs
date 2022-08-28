use crate::memory::physical::PhysicalFrameAllocator;
use crate::memory::Error;
use crate::memory::Result;
use core::marker::PhantomData;
use kstd::sync::{Mutex, MutexGuard};
use x86_64::structures::paging::page::PageRange;
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
    pub fn ensure_is_mapped(
        &mut self,
        range: PageRange<S>,
        memory_kind: MemoryKind,
        user_accessible: UserAccessible,
    ) -> Result<()> {
        let page_table_flags = Self::translate_page_table_flags(memory_kind, user_accessible);

        for page in range {
            let translate_result = self.page_table.translate_page(page);
            if matches!(translate_result, Err(_)) {
                let frame = self.allocate_frame()?;
                self.map_frame_to_page(frame, page, page_table_flags)?;
            }
        }
        Ok(())
    }

    fn translate_page_table_flags(
        memory_kind: MemoryKind,
        user_accessible: UserAccessible,
    ) -> PageTableFlags {
        let mut page_table_flags = PageTableFlags::PRESENT | PageTableFlags::NO_EXECUTE;
        if user_accessible.into() {
            page_table_flags |= PageTableFlags::USER_ACCESSIBLE;
        }
        match memory_kind {
            MemoryKind::ReadOnly => {}
            MemoryKind::Writable => page_table_flags |= PageTableFlags::WRITABLE,
            MemoryKind::Executable => page_table_flags.remove(PageTableFlags::NO_EXECUTE),
        };
        page_table_flags
    }

    pub fn allocate_and_map_page_range(
        &mut self,
        range: PageRange<S>,
        memory_kind: MemoryKind,
        user_accessible: UserAccessible,
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

        for page in range {
            let frame = self.allocate_frame()?;
            self.map_frame_to_page(frame, page, page_table_flags)?;
        }
        Ok(())
    }

    fn allocate_frame(&mut self) -> Result<PhysFrame<S>> {
        self.physical_frame_allocator
            .allocate_frame()
            .ok_or(Error::FrameAllocationFailed)
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
