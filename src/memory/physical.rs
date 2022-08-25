use bootloader::boot_info::{MemoryRegionKind, MemoryRegions};
use x86_64::structures::paging::{FrameAllocator, Page, PhysFrame, Size4KiB};
use x86_64::PhysAddr;

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct PhysicalFrameAllocator {
    memory_regions: &'static MemoryRegions,
    next: usize,
}

impl PhysicalFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_regions: &'static MemoryRegions) -> Self {
        PhysicalFrameAllocator {
            memory_regions,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        self.memory_regions
            .iter()
            // get usable regions from memory map
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            // map each region to its address range
            .map(|r| r.start..r.end)
            // transform to an iterator of frame start addresses
            .flat_map(|r| r.step_by(Page::<Size4KiB>::SIZE as usize))
            // create `PhysFrame` types from the start addresses
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for PhysicalFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
