use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::io::from_read::FromRead;
use crate::io::fs::path::Path;
use crate::io::fs::ustar::header::HeaderBlock;
use crate::io::read::Read;
use crate::io::read_at::ReadAt;
use crate::syscall::error::Errno;
use crate::Result;

pub mod header;

const BLOCK_SIZE: u64 = 512;

pub struct UstarFs<T: ReadAt> {
    source: T,
}

impl<T> UstarFs<T>
where
    T: ReadAt,
{
    pub fn new(source: T) -> Self {
        Self { source }
    }

    pub fn open(&mut self, path: &dyn AsRef<Path>) -> Result<UstarFile> {
        // search for file
        let needle = path.as_ref().to_string();
        let mut offset = 0;
        let header_block: Option<HeaderBlock> = loop {
            let mut current_section = FromRead::new(&self.source, offset);
            let current_header = HeaderBlock::decode(&mut current_section)?;

            // check for end of archive
            if current_header.is_end_block() {
                let mut next_section = FromRead::new(&self.source, offset + BLOCK_SIZE);
                let next_header = HeaderBlock::decode(&mut next_section)?;
                if next_header.is_end_block() {
                    // two end blocks is end of archive
                    return Err(Errno::ENOENT);
                }
            }

            if current_header.name == needle {
                break Some(current_header);
            }
            let jump = (((current_header.size + BLOCK_SIZE - 1) >> 9) + 1) << 9;
            offset += jump;
        };

        // file found, read data
        offset += BLOCK_SIZE;
        let header = header_block.unwrap();
        let mut data = FromRead::new(&self.source, offset);
        let mut buffer = vec![0_u8; header.size as usize];
        data.read_exact(&mut buffer)?;
        Ok(UstarFile::new(header, buffer))
    }
}

pub struct UstarFile {
    header: HeaderBlock,
    data: Vec<u8>,
}

impl UstarFile {
    fn new(header: HeaderBlock, data: Vec<u8>) -> Self {
        Self { header, data }
    }

    pub fn size(&self) -> u64 {
        self.header.size
    }

    pub fn name(&self) -> &str {
        &self.header.name
    }

    pub fn data(&self) -> &[u8] {
        // no data_mut because no write support
        self.data.as_slice()
    }
}
