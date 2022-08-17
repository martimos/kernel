use crate::io::fs::{IBlockDeviceFile, INodeBase, INodeNum, Stat};
use alloc::string::String;
use kstd::io::block::BlockDevice;
use kstd::io::ReadAt;

pub struct BlockDeviceFile<D>
where
    D: BlockDevice,
{
    device: D,
    stat: Stat,
    name: String,
}

impl<D> BlockDeviceFile<D>
where
    D: BlockDevice,
{
    pub fn new(device: D, inode: INodeNum, name: String) -> Self {
        let block_size = device.block_size();
        let block_count = device.block_count();
        Self {
            device,
            stat: Stat {
                inode,
                size: block_size as u64 * block_count as u64,
                ..Default::default()
            },
            name,
        }
    }
}

impl<D> INodeBase for BlockDeviceFile<D>
where
    D: BlockDevice,
{
    fn num(&self) -> INodeNum {
        self.stat.inode
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn stat(&self) -> Stat {
        self.stat
    }
}

impl<D> IBlockDeviceFile for BlockDeviceFile<D>
where
    D: 'static + BlockDevice,
{
    fn block_count(&self) -> usize {
        self.device.block_count()
    }

    fn block_size(&self) -> usize {
        self.device.block_size()
    }

    fn read_block(&self, block: u64, buf: &mut dyn AsMut<[u8]>) -> kstd::io::Result<()> {
        self.device.read_block(block, buf).map(|_| ())
    }

    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> kstd::io::Result<usize> {
        self.device.read_at(offset, buf)
    }

    fn write_at(&mut self, _offset: u64, _buf: &dyn AsRef<[u8]>) -> kstd::io::Result<usize> {
        todo!("implement write_at for BlockDevice")
    }
}
