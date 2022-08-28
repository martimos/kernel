use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::{Page, PageSize};
use x86_64::VirtAddr;

use crate::memory::size::Size;

macro_rules! checks {
    (
        ($name1:ident, $addr1:expr, $size1:expr),
        ($name2:ident, $addr2:expr, $size2:expr)
    ) => {
        assert!(
            ($addr1 as u64) != ($addr2 as u64),
            concat!(stringify!($name1), " and ", stringify!($name2), " have the same starting address")
        );
        assert!(
            ($addr1 as u64) < ($addr2 as u64),
            concat!("keep the list sorted, address of ", stringify!($name1), " is larger than ", stringify!($name2))
        );
        assert!(
            !$name1.overlaps(&$name2),
            concat!("Overlap between ", stringify!($name1), " and ", stringify!($name2))
        );
    };
    (
        ($name1:ident, $addr1:expr, $size1:expr),
        ($name2:ident, $addr2:expr, $size2:expr),
        $(($name:ident, $addr:expr, $size:expr)),+) => {
        checks!(($name1, $addr1, $size1), ($name2, $addr2, $size2));
        checks!(($name2, $addr2, $size2), $(($name, $addr, $size)),*);
    };
}

macro_rules! declare_spans {
    ($(($name:ident, $addr:expr, $size:expr)),+) => {
        $(
            pub const $name: MemorySpan = {
                let sz: $crate::memory::size::Size = $size;
                let addr: u64 = $addr;
                let virt_addr: x86_64::VirtAddr = x86_64::VirtAddr::new_truncate(addr);
                $crate::memory::span::MemorySpan::new(virt_addr, sz.bytes())
            };
        )+
        const _CHECK: () = {
            checks!($(($name, $addr, $size)),+);
        };
    };
}

declare_spans! {
    // 32 TiB as potential user memory should be sufficient
    (USERLAND, 0x1111_1111_0000, Size::TiB(32)),
    // heap is mapped on kernel initialization, make it only as big as necessary
    (HEAP, 0x4444_4444_0000, Size::MiB(1)),
    (KBUFFER, 0x5555_5555_0000, Size::TiB(1))
}

pub struct MemorySpan {
    #[allow(dead_code)] // needed for the compile-time check of the address, never read at runtime
    numeric_address: u64,
    start: VirtAddr,
    len: usize,
}

impl MemorySpan {
    const fn new(start: VirtAddr, len: usize) -> Self {
        assert!(len > 0, "length size must be greater than 0");
        Self {
            numeric_address: start.as_u64(),
            start,
            len,
        }
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

    #[allow(dead_code)] // needed for the compile-time check of the address, never read at runtime
    const fn numeric_address(&self) -> u64 {
        self.numeric_address
    }

    #[allow(dead_code)] // needed for the compile-time check of the address, never read at runtime
    const fn overlaps(&self, other: &Self) -> bool {
        let self_start = self.numeric_address();
        let self_end = self.numeric_address() + self.len() as u64;
        let other_start = other.numeric_address();
        let other_end = other.numeric_address() + other.len() as u64;
        const_max(self_start, other_start) < const_min(self_end, other_end)
    }
}

#[allow(dead_code)] // needed for the compile-time check of the address, never read at runtime
const fn const_max(a: u64, b: u64) -> u64 {
    if a > b {
        a
    } else {
        b
    }
}

#[allow(dead_code)] // needed for the compile-time check of the address, never read at runtime
const fn const_min(a: u64, b: u64) -> u64 {
    if a < b {
        a
    } else {
        b
    }
}
