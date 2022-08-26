use crate::memory::physical::PhysicalFrameAllocator;
use bootloader::boot_info::Optional;
use bootloader::BootInfo;
use x86_64::structures::paging::mapper::{MapToError, UnmapError};
use x86_64::structures::paging::{OffsetPageTable, PageSize, PageTable};
use x86_64::VirtAddr;

#[cfg(test)]
use crate::serial_println;
use crate::vga_buffer;

pub mod allocator;
pub mod heap;
pub mod manager;
pub mod physical;
pub mod size;
pub mod span;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    PageMappingFailed,
    PageUnmappingFailed,
    FrameAllocationFailed,
    OutOfMemory,
}

impl<S: PageSize> From<MapToError<S>> for Error {
    fn from(_: MapToError<S>) -> Self {
        Self::PageMappingFailed
    }
}

impl From<UnmapError> for Error {
    fn from(_: UnmapError) -> Self {
        Self::PageUnmappingFailed
    }
}

pub fn init_memory(boot_info: &'static mut BootInfo) {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        vga_buffer::init_vga_buffer(framebuffer);
    } else {
        #[cfg(test)]
        serial_println!("no vga buffer given, skipping initialization");
        #[cfg(not(test))]
        panic!("no vga buffer given");
    }

    let addr = match boot_info.physical_memory_offset {
        Optional::Some(addr) => addr,
        Optional::None => panic!("no boot info physical memory offset given"),
    };
    let phys_mem_offset = VirtAddr::new(addr);
    let mapper = unsafe { create_offset_page_table(phys_mem_offset) };
    let frame_allocator = unsafe { PhysicalFrameAllocator::init(&boot_info.memory_regions) };
    manager::init_memory_manager(mapper, frame_allocator);
    heap::init_heap().expect("heap initialization failed");
}

/// Initialize a new OffsetPageTable.
///
/// # Safety
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn create_offset_page_table(
    physical_memory_offset: VirtAddr,
) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}
