use alloc::collections::BTreeMap;
use alloc::rc::Rc;

use spin::rwlock::RwLock;
use spin::Mutex;

use crate::device::block::BlockDevice;
use crate::syscall::error::Errno;
use crate::Result;

#[derive(Clone)]
struct Block(Rc<RwLock<[u8; 512]>>);

impl Block {
    pub fn new() -> Self {
        Self(Rc::new(RwLock::new([0_u8; 512])))
    }
}

pub struct CowBlockDevice<D>
where
    D: BlockDevice,
{
    inner: D,
    blocks: Mutex<BTreeMap<u64, Block>>,
}

impl<D> BlockDevice for CowBlockDevice<D>
where
    D: BlockDevice,
{
    fn block_size(&self) -> usize {
        self.inner.block_size()
    }

    fn block_count(&self) -> usize {
        self.inner.block_count()
    }

    fn read_block(&self, block: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
        let buffer = buf.as_mut();
        let block_size = self.block_size();
        if buffer.len() < block_size {
            return Err(Errno::EIO);
        }

        if let Some(b) = self.blocks.lock().get(&block).cloned() {
            buffer[0..block_size].copy_from_slice(b.0.read().as_slice());
            return Ok(block_size);
        }

        Err(Errno::EFAULT)
    }

    fn write_block(&mut self, block: u64, buf: &dyn AsRef<[u8]>) -> Result<usize> {
        let buffer = buf.as_ref();
        let block_size = self.block_size();
        if buffer.len() < block_size {
            return Err(Errno::EIO);
        }

        if !self.blocks.lock().contains_key(&block) {
            self.load_block(block)?;
        }

        let b = self.blocks.lock().get(&block).cloned().unwrap();
        b.0.write()[0..block_size].copy_from_slice(buffer);
        Ok(block_size)
    }
}

impl<D> CowBlockDevice<D>
where
    D: BlockDevice,
{
    fn load_block(&self, block: u64) -> Result<usize> {
        let b = Block::new();
        self.inner.read_block(block, &mut b.0.write().as_mut())?;
        self.blocks.lock().insert(block, b);
        Ok(0)
    }
}
