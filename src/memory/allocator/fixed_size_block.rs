use alloc::alloc::GlobalAlloc;
use core::{alloc::Layout, mem, ptr, ptr::NonNull};

use x86_64::structures::paging::{PageSize, Size4KiB};

use crate::memory::allocator::backend::MemoryBackend;
use crate::memory::heap::Locked;

struct ListNode {
    next: Option<&'static mut ListNode>,
}

/// The block sizes to use.
///
/// The sizes must each be power of 2 because they are also used as
/// the block alignment (alignments must be always powers of 2).
const BLOCK_SIZES: &[usize] = &[
    8,
    16,
    32,
    64,
    128,
    256,
    512,
    1024,
    2048,
    Size4KiB::SIZE as usize,
];

pub struct FixedSizeBlockAllocator<M> {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
    memory_backend: M,
}

impl<M> FixedSizeBlockAllocator<M>
where
    M: MemoryBackend,
{
    /// Creates an empty FixedSizeBlockAllocator.
    pub const fn new(memory_backend: M) -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        FixedSizeBlockAllocator {
            list_heads: [EMPTY; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
            memory_backend,
        }
    }

    /// Initialize the allocator with the given heap bounds.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller must guarantee that the given
    /// heap bounds are valid and that the heap is unused. This method must be
    /// called only once.
    pub unsafe fn init(&mut self, heap_start: *mut u8, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    /// Allocates using the fallback allocator.
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }
}

unsafe impl<M> GlobalAlloc for Locked<FixedSizeBlockAllocator<M>>
where
    M: MemoryBackend,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        let required_block_size = layout.size().max(layout.align());
        let index = BLOCK_SIZES.iter().position(|&s| s >= required_block_size);
        let pointer = match index {
            Some(index) => {
                match allocator.list_heads[index].take() {
                    Some(node) => {
                        allocator.list_heads[index] = node.next.take();
                        node as *mut ListNode as *mut u8
                    }
                    None => {
                        // no block exists in list => allocate new block
                        let block_size = BLOCK_SIZES[index];
                        // only works if all block sizes are a power of 2
                        let block_align = block_size;
                        let layout = Layout::from_size_align(block_size, block_align).unwrap();
                        allocator.fallback_alloc(layout)
                    }
                }
            }
            None => allocator.fallback_alloc(layout),
        };
        allocator
            .memory_backend
            .memory_allocated(pointer as *const u8, required_block_size)
            .expect("memory allocation via backend failed");
        pointer
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        let required_block_size = layout.size().max(layout.align());
        match BLOCK_SIZES.iter().position(|&s| s >= required_block_size) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.list_heads[index].take(),
                };
                // verify that block has size and alignment required for storing node
                assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
                assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.list_heads[index] = Some(&mut *new_node_ptr);
            }
            None => {
                let ptr = NonNull::new(ptr)
                    .unwrap_or_else(|| panic!("invalid pointer {:p} passed to deallocate", ptr));
                allocator.fallback_allocator.deallocate(ptr, layout);
            }
        };
        allocator
            .memory_backend
            .memory_deallocated(ptr as *const u8, required_block_size)
            .expect("memory deallocation via backend failed");
    }
}
