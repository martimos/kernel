use bootloader::boot_info::{MemoryRegionKind, MemoryRegions};
use core::marker::PhantomData;
use x86_64::structures::paging::{FrameAllocator, Page, PageSize, PhysFrame};
use x86_64::PhysAddr;

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct PhysicalFrameAllocator<S: PageSize> {
    memory_regions: &'static MemoryRegions,
    next: usize,
    _marker: PhantomData<S>,
}

impl<S: PageSize> PhysicalFrameAllocator<S> {
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
            _marker: PhantomData,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame<S>> {
        self.memory_regions
            .iter()
            // get usable regions from memory map
            .filter(|r| r.kind == MemoryRegionKind::Usable)
            // map each region to its address range
            .map(|r| r.start..r.end)
            // transform to an iterator of frame start addresses
            .flat_map(|r| r.step_by(Page::<S>::SIZE as usize))
            // create `PhysFrame` types from the start addresses
            .map(|addr| PhysFrame::<S>::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl<S: PageSize> FrameAllocator<S> for PhysicalFrameAllocator<S> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
