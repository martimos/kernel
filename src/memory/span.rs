use crate::memory::size::Size;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::{Page, PageSize};
use x86_64::VirtAddr;

/*
 * This is all pretty unsafe. You manually have to make sure that the addresses are valid
 * and don't overlap.
 */

macro_rules! span {
    ($name:ident, $addr:expr, $size:expr) => {
        pub const $name: MemorySpan = {
            let sz: $crate::memory::size::Size = $size;
            let addr: u64 = $addr;
            let virt_addr: x86_64::VirtAddr = x86_64::VirtAddr::new_truncate(addr);
            $crate::memory::span::MemorySpan::new(virt_addr, sz.bytes())
        };
    };
}

span!(HEAP, 0x4444_4444_0000, Size::MiB(1));
span!(KBUFFER, 0x5555_5555_0000, Size::MiB(16));

pub struct MemorySpan {
    start: VirtAddr,
    len: usize,
}

impl MemorySpan {
    const fn new(start: VirtAddr, len: usize) -> Self {
        Self { start, len }
    }

    pub const fn as_mut_ptr<T>(&self) -> *mut T {
        self.start.as_mut_ptr::<T>()
    }

    #[allow(clippy::len_without_is_empty)] // since a span should never be empty, is_empty doesn't make sense
    pub const fn len(&self) -> usize {
        self.len
    }

    pub fn as_page_range<S: PageSize>(&self) -> PageRange<S> {
        Page::range(
            Page::<S>::containing_address(self.start),
            Page::<S>::containing_address(self.start + self.len),
        )
    }
}
