//! Backends for allocators.
//!
//! Provides backends for allocators, so that those don't need to handle mapping pages themselves.
//!
//! Memory backends are an abstraction over the memory management system, allowing the allocator to
//! allocate memory without having to worry about how the memory is actually allocated or whether
//! pages are already mapped (depending on the backend used).
//!
//! There's different types of backends available.
//!
//! ## Already mapped
//!
//! The [`MemoryAlreadyMappedBackend`] is a backend that assumes, that the allocator only operates
//! over already mapped memory. It is essentially a no-op implementation, but works as a marker in
//! the source code, that the allocator that is using this backend, needs to use mapped memory.
//!
//! ## Lazy page mapping
//!
//! The [`LazyPageMappingBackend`] is a backend that maps memory lazily. The allocator designates
//! a pointer to return to the caller, and then passes that pointer to this backend. The backend
//! then ensures, that enough pages are mapped, and that the memory is available for use.

use crate::memory::Result;
pub mod already_mapped;
pub mod lazy_page;

pub trait MemoryBackend {
    fn memory_allocated(&mut self, addr: *const u8, size: usize) -> Result<()>;
    fn memory_deallocated(&mut self, addr: *const u8, size: usize) -> Result<()>;
}
