use alloc::vec::Vec;
use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;

use crate::io::read::Read;
use crate::Result;
use crate::{read_bytes, read_le_u16, read_le_u32};

#[derive(Debug)]
pub struct BlockGroupDescriptorTable(Vec<BlockGroupDescriptor>);

impl BlockGroupDescriptorTable {
    pub fn decode(source: &mut impl Read, num_entries: usize) -> Result<Self> {
        let mut entries = Vec::with_capacity(num_entries);
        for _ in 0..num_entries {
            entries.push(BlockGroupDescriptor::decode(source)?)
        }
        Ok(Self(entries))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<I: SliceIndex<[BlockGroupDescriptor]>> Index<I> for BlockGroupDescriptorTable {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<I: SliceIndex<[BlockGroupDescriptor]>> IndexMut<I> for BlockGroupDescriptorTable {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

#[derive(Debug)]
pub struct BlockGroupDescriptor {
    pub block_usage_bitmap_block: u32,
    pub inode_usage_bitmap_block: u32,
    pub inode_table_starting_block: u32,
    pub num_unallocated_blocks: u16,
    pub num_unallocated_inodes: u16,
    pub num_directories: u16,
}

impl BlockGroupDescriptor {
    pub fn decode(source: &mut impl Read) -> Result<Self> {
        let s = Self {
            block_usage_bitmap_block: read_le_u32!(source),
            inode_usage_bitmap_block: read_le_u32!(source),
            inode_table_starting_block: read_le_u32!(source),
            num_unallocated_blocks: read_le_u16!(source),
            num_unallocated_inodes: read_le_u16!(source),
            num_directories: read_le_u16!(source),
        };
        let _ = read_bytes!(source, 14); // read until we've read 32 bytes in total
        Ok(s)
    }
}
