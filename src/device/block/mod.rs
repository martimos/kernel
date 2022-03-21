use alloc::vec::Vec;

use crate::io::ReadAt;

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

        // read blocks
        // TODO: optimize this, this is doable without heap allocation
        let block_count = 1 + ((target.len() - 1) >> 9); // depends on block_size=512 aka block_size=1<<9
        let mut read_block_data: Vec<u8> = Vec::with_capacity(block_size);
        for _ in 0..block_count {
            let mut block = [0_u8; 512];
            self.read_block(block_index, &mut block);
            read_block_data.reserve(block_size);
            block.iter().for_each(|&b| read_block_data.push(b));
        }
        target.copy_from_slice(&read_block_data[relative_offset..relative_offset + target.len()]);

        Ok(target.len())
    }
}
