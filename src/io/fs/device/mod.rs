use crate::io::fs::IBlockDeviceHandle;
use kstd::io::block::BlockDevice;

pub mod block;

pub struct FileBlockDevice {
    file: IBlockDeviceHandle,
}

impl FileBlockDevice {
    pub fn new(file: IBlockDeviceHandle) -> Self {
        Self { file }
    }
}

impl BlockDevice for FileBlockDevice {
    fn block_size(&self) -> usize {
        self.file.read().block_size()
    }

    fn block_count(&self) -> usize {
        self.file.read().block_count()
    }

    fn read_block(&self, block: u64, buf: &mut dyn AsMut<[u8]>) -> kstd::io::Result<usize> {
        self.file.read().read_block(block, buf)?;
        Ok(buf.as_mut().len())
    }

    fn write_block(&mut self, _block: u64, _buf: &dyn AsRef<[u8]>) -> kstd::io::Result<usize> {
        todo!()
    }
}
