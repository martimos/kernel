use crate::memory::size::Size;
use x86_64::VirtAddr;

/*
 * This is all pretty unsafe. You manually have to make sure that the addresses are valid
 * and don't overlap.
 */

pub const HEAP: MemorySpan = MemorySpan::new(
    VirtAddr::new_truncate(0x4444_4444_0000),
    Size::MiB(1).bytes(),
);
pub const KERNEL_STACK: MemorySpan = MemorySpan::new(
    VirtAddr::new_truncate(0x5555_5555_0000),
    Size::MiB(1).bytes(),
);

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
}
