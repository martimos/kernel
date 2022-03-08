use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::io::fs::path::Path;
use crate::io::fs::ustar::header::HeaderBlock;
use crate::io::read::Read;
use crate::io::read_at::ReadAt;
use crate::io::section_read::SectionRead;
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
        let mut header_block: Option<HeaderBlock> = None;
        loop {
            let mut current_section = SectionRead::new(&self.source, offset);
            let current_header = HeaderBlock::decode(&mut current_section)?;

            // check for end of archive
            if current_header.is_end_block() {
                let mut next_section = SectionRead::new(&self.source, offset + BLOCK_SIZE);
                let next_header = HeaderBlock::decode(&mut next_section)?;
                if next_header.is_end_block() {
                    // two end blocks is end of archive
                    return Err(Errno::ENOENT);
                }
            }

            if current_header.name == needle {
                header_block = Some(current_header);
                break;
            }
            offset += BLOCK_SIZE + current_header.size; // skip to the next header
        }

        // file found, read data
        offset += BLOCK_SIZE;
        let header = header_block.unwrap();
        let mut data = SectionRead::new(&self.source, offset);
        let mut buffer = vec![0_u8; header.size as usize];
        data.read_exact(&mut buffer)?;
        Ok(UstarFile {
            header,
            data: buffer,
        })
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
}
