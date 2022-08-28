use crate::memory::allocator::backend::MemoryBackend;
use crate::memory::Result;

pub struct MemoryAlreadyMappedBackend;

impl MemoryBackend for MemoryAlreadyMappedBackend {
    fn memory_allocated(&mut self, _addr: *const u8, _size: usize) -> Result<()> {
        Ok(())
    }

    fn memory_deallocated(&mut self, _addr: *const u8, _size: usize) -> Result<()> {
        Ok(())
    }
}
