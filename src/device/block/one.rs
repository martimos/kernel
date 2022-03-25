use core::sync::atomic::{AtomicUsize, Ordering};

use crate::device::block::BlockDevice;
use crate::Result;

pub struct OneDevice {
    pub block_size_count: AtomicUsize,
    pub read_block_count: AtomicUsize,
    pub write_block_count: AtomicUsize,
    pub block_count_count: AtomicUsize,

    block_size: usize,
    block_count: usize,
}

impl OneDevice {
    pub fn new(block_size: usize, block_count: usize) -> Self {
        Self {
            block_size_count: AtomicUsize::default(),
            read_block_count: AtomicUsize::default(),
            write_block_count: AtomicUsize::default(),
            block_count_count: AtomicUsize::default(),
            block_size,
            block_count,
        }
    }
}

impl BlockDevice for OneDevice {
    fn block_size(&self) -> usize {
        let _ = self.block_size_count.fetch_add(1, Ordering::SeqCst);

        self.block_size
    }

    fn block_count(&self) -> usize {
        let _ = self.block_count_count.fetch_add(1, Ordering::SeqCst);

        self.block_count
    }

    fn read_block(&self, _: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
        let _ = self.read_block_count.fetch_add(1, Ordering::SeqCst);

        let buffer = buf.as_mut();
        buffer[0..self.block_size].fill(1);

        Ok(self.block_size)
    }

    fn write_block(&mut self, _: u64, _: &dyn AsRef<[u8]>) -> Result<usize> {
        let _ = self.write_block_count.fetch_add(1, Ordering::SeqCst);

        Ok(self.block_size)
    }
}
