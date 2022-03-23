use alloc::rc::Rc;
use alloc::vec;
use alloc::vec::Vec;

use spin::{Mutex, RwLock};

use crate::collection::lru::LruCache;
use crate::device::block::BlockDevice;
use crate::syscall::error::Errno;
use crate::Result;

struct Block {
    num: u64,
    data: Vec<u8>,
}

pub struct BlockCache<D>
where
    D: BlockDevice,
{
    cache: Mutex<LruCache<Rc<RwLock<Block>>>>,
    block_size: usize,
    device: D,
}

impl<D> BlockCache<D>
where
    D: BlockDevice,
{
    pub fn new(device: D, size: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(size)),
            block_size: device.block_size(),
            device,
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
                let _ = self.device.read_block(block, &mut data)?;

                let b = Rc::new(RwLock::new(Block { num: block, data }));
                self.cache.lock().insert(b.clone());
                b
            }
        };
        buffer.copy_from_slice(&block.read().data);

        Ok(buffer.len())
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use core::sync::atomic::{AtomicUsize, Ordering};

    use crate::device::block::cache::BlockCache;
    use crate::device::block::BlockDevice;
    use crate::Result;

    struct ZeroDevice {
        block_size_count: AtomicUsize,
        read_block_count: AtomicUsize,
    }

    impl ZeroDevice {
        fn new() -> Self {
            Self {
                block_size_count: Default::default(),
                read_block_count: Default::default(),
            }
        }
    }

    impl BlockDevice for ZeroDevice {
        fn block_size(&self) -> usize {
            let _ = self.block_size_count.fetch_add(1, Ordering::SeqCst);

            512
        }

        fn read_block(&self, _: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
            let _ = self.read_block_count.fetch_add(1, Ordering::SeqCst);

            let buffer = buf.as_mut();
            buffer.fill(0);

            Ok(buffer.len())
        }
    }

    #[test_case]
    fn test_cache_read() {
        let device = ZeroDevice::new();
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
        assert_eq!(4, cache.device.read_block_count.load(Ordering::SeqCst));
        assert_eq!(1, cache.device.block_size_count.load(Ordering::SeqCst));
    }
}
