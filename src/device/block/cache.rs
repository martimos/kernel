use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;

use spin::{Mutex, RwLock};

use crate::device::block::BlockDevice;
use crate::syscall::error::Errno;
use crate::Result;
use kstd::collections::lru::LruCache;

struct Block<D>
where
    D: BlockDevice,
{
    device: Rc<RwLock<D>>,
    num: u64,
    data: Vec<u8>,
}

impl<D> Drop for Block<D>
where
    D: BlockDevice,
{
    fn drop(&mut self) {
        let _ = self.device.write().write_block(self.num, &self.data);
        // don't panic, even if the write fails
    }
}

pub struct BlockCache<D>
where
    D: BlockDevice,
{
    cache: Mutex<LruCache<Rc<RwLock<Block<D>>>>>,
    block_size: usize,
    device: Rc<RwLock<D>>,
}

impl<D> BlockCache<D>
where
    D: BlockDevice,
{
    pub fn new(device: D, size: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(size)),
            block_size: device.block_size(),
            device: Rc::new(RwLock::new(device)),
        }
    }
}

impl<D> BlockDevice for BlockCache<D>
where
    D: BlockDevice,
{
    fn block_size(&self) -> usize {
        self.block_size
    }

    fn block_count(&self) -> usize {
        self.device.read().block_count()
    }

    fn read_block(&self, block: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
        let buffer = buf.as_mut();
        let len = buffer.len();
        if len != self.block_size {
            return Err(Errno::EIO);
        }

        let res = self.cache.lock().find(|b| b.read().num == block).cloned();
        // cache.lock() must not live within the match because we may lock it again to insert a new block
        let block = match res {
            Some(b) => b,
            None => {
                let mut data = vec![0_u8; self.block_size];
                let _ = self.device.read().read_block(block, &mut data)?;

                let b = Rc::new(RwLock::new(Block {
                    device: self.device.clone(),
                    num: block,
                    data,
                }));
                self.cache.lock().insert(b.clone());
                b
            }
        };
        buffer.copy_from_slice(&block.read().data);

        Ok(buffer.len())
    }

    fn write_block(&mut self, block: u64, buf: &dyn AsRef<[u8]>) -> Result<usize> {
        self.device.write().write_block(block, buf)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use core::sync::atomic::Ordering;

    use crate::device::block::cache::BlockCache;
    use crate::device::block::one::OneDevice;
    use crate::device::block::BlockDevice;

    #[test_case]
    fn test_cache_read() {
        let device = OneDevice::new(512, 1024);
        let cache = BlockCache::new(device, 10);
        let mut data = vec![0_u8; cache.block_size()];
        for block_num in [1, 2, 3, 1, 2, 3, 4] {
            cache.read_block(block_num, &mut data).unwrap();
        }
        /*
        Given the block sequence above, we should have the following events.
        1: no hit, load from device
        2: no hit
        3: no hit
        1: cache hit, don't touch device
        2: cache hit
        3: cache hit
        4: no hit, load from device again
        As can be seen, we have 4 requests that should touch, the disk, which
        is what we test now.
         */
        assert_eq!(
            4,
            cache.device.read().read_block_count.load(Ordering::SeqCst)
        );
        assert_eq!(
            1,
            cache.device.read().block_size_count.load(Ordering::SeqCst)
        );
    }
}
