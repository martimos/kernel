use crate::io::read_at::ReadAt;

pub trait BlockDevice {
    fn block_size(&self) -> usize;
    fn read_block(&self, block: u64, buf: &mut dyn AsMut<[u8]>);
}

impl<T> ReadAt for T
where
    T: BlockDevice,
{
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> crate::Result<usize> {
        let target = buf.as_mut();

        let block_size = self.block_size();
        assert_eq!(
            512, block_size,
            "block sizes other than 512 are not supported yet"
        );
        let block_index = offset >> 9;
        let relative_offset = (offset & 511) as usize;

        let mut block = [0_u8; 512];
        self.read_block(block_index, &mut block);

        target.copy_from_slice(&block[relative_offset..relative_offset + target.len()]);

        Ok(target.len())
    }
}
