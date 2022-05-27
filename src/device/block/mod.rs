use alloc::vec;
use alloc::vec::Vec;

use crate::io::ReadAt;
use crate::Result;

pub mod cache;
pub mod cow;
#[cfg(test)]
pub mod one;

pub trait BlockDevice {
    fn block_size(&self) -> usize;
    fn block_count(&self) -> usize;
    fn read_block(&self, block: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize>;
    fn write_block(&mut self, block: u64, buf: &dyn AsRef<[u8]>) -> Result<usize>;
}

impl<T> ReadAt<u8> for T
where
    T: BlockDevice,
{
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
        let buffer = buf.as_mut();
        let block_size = self.block_size();
        if offset % buffer.len() as u64 == 0 && buffer.len() == block_size {
            // if we read exactly one block, and that read is aligned, delegate to the device impl
            return self.read_block(offset / block_size as u64, buf);
        }

        let start_block = offset / block_size as u64;
        let end_block = (offset + buffer.len() as u64) / block_size as u64;
        let relative_offset = offset as usize % block_size;
        let block_count = if relative_offset == 0 && start_block != end_block {
            end_block - start_block
        } else {
            end_block - start_block + 1
        } as usize;

        // read blocks
        let mut data: Vec<u8> = vec![0_u8; block_count * block_size];
        for i in 0..block_count {
            let start_index = i * block_size;
            let end_index = start_index + block_size;
            let read_block_index = start_block + i as u64;

            self.read_block(read_block_index, &mut &mut data[start_index..end_index])?;
        }
        buffer.copy_from_slice(&data[relative_offset..relative_offset + buffer.len()]);

        Ok(buffer.len())
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::device::block::BlockDevice;
    use crate::io::ReadAt;
    use crate::Result;

    struct TestBlockDevice {
        block_size: usize,
        block_count: usize,
    }

    impl BlockDevice for TestBlockDevice {
        fn block_size(&self) -> usize {
            self.block_size
        }

        fn block_count(&self) -> usize {
            self.block_count
        }

        fn read_block(&self, block: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
            if block >= self.block_count as u64 {
                panic!(
                    "attempted to read block index {}, but only have {} blocks",
                    block, self.block_count
                );
            }
            let buffer = buf.as_mut();
            buffer[0..self.block_size].fill(block as u8 + 1);
            Ok(buffer.len())
        }

        fn write_block(&mut self, _: u64, _: &dyn AsRef<[u8]>) -> Result<usize> {
            panic!("no write in this test, please");
        }
    }

    #[test_case]
    fn test_read_at_0() {
        let dev = TestBlockDevice {
            block_size: 512,
            block_count: 1,
        };
        let mut data = vec![0_u8; 512];

        dev.read_at(0, &mut data).unwrap();
        assert_eq!(vec![1_u8; 512], data);
    }

    #[test_case]
    fn test_read_at_512() {
        let dev = TestBlockDevice {
            block_size: 512,
            block_count: 3,
        };
        let mut data = vec![0_u8; 1025];

        dev.read_at(0, &mut &mut data[0..1024]).unwrap();
        for &item in data.iter().take(512) {
            assert_eq!(1, item);
        }
        for &item in data.iter().skip(512).take(512) {
            assert_eq!(2, item);
        }
        assert_eq!(0, data[1024]);
    }

    #[test_case]
    fn test_read_at_7() {
        let dev = TestBlockDevice {
            block_size: 7,
            block_count: 40,
        };
        let mut data = vec![0_u8; 50];

        dev.read_at(19, &mut &mut data[5..37]).unwrap();
        assert_eq!(
            vec![
                0, 0, 0, 0, 0, 3, 3, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6,
                7, 7, 7, 7, 7, 7, 7, 8, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            data
        );
    }
}
