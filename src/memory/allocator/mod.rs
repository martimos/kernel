pub mod backend;
pub mod bump;
pub mod fixed_size_block;
pub mod linked_list;

/// Align the given address `addr` upwards to alignment `align`.
///
/// Requires that `align` is a power of two.
pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
