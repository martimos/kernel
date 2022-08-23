use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;

use kstd::io::block::BlockDevice;
use kstd::io::ReadAt;
use kstd::io::Result;
use kstd::sync::RwLock;

use crate::io::fs::ext2::base::Ext2NodeBase;
use crate::io::fs::ext2::inode::{Ext2INode, Ext2INodeType};
use crate::io::fs::ext2::Inner;
use crate::io::fs::{IFile, INodeBase, INodeNum, Stat};

pub struct Ext2File<D>
where
    D: 'static + BlockDevice,
{
    base: Ext2NodeBase<D>,
}

impl<D> Ext2File<D>
where
    D: 'static + BlockDevice,
{
    pub fn new(fs: Arc<RwLock<Inner<D>>>, ext2_inode: Ext2INode, name: String) -> Self {
        if ext2_inode.node_type != Ext2INodeType::RegularFile {
            panic!("root inode is not a file, but a {:?}", ext2_inode.node_type);
        }

        Self {
            base: Ext2NodeBase::new(fs, ext2_inode, name),
        }
    }

    /// Takes the index of a block within a file node and determines, in which pointer list to look
    /// for the block address. For example, a block index of 0 (the very first block in a file) is
    /// stored in the direct pointers list. A block index of 12 is the first entry in the single
    /// indirect pointers list.
    fn determine_block_pointer_type(&self, block_index: usize) -> BlockPointerType {
        let block_size = self.base.fs().read().superblock.block_size;
        let pointers_per_block = (block_size / 4) as usize;
        let hi_single_indirect = pointers_per_block;
        let hi_double_indirect = hi_single_indirect * pointers_per_block;
        let hi_triple_indirect = hi_double_indirect * pointers_per_block;
        match block_index {
            0..12 => BlockPointerType::Direct,
            x if x >= 12 && x < hi_single_indirect => BlockPointerType::SingleIndirect,
            x if x >= hi_single_indirect && x < hi_double_indirect => {
                BlockPointerType::DoubleIndirect
            }
            x if x >= hi_double_indirect && x < hi_triple_indirect => {
                BlockPointerType::TripleIndirect
            }
            _ => unreachable!("too many blocks"),
        }
    }
}

impl<D> INodeBase for Ext2File<D>
where
    D: 'static + BlockDevice,
{
    fn num(&self) -> INodeNum {
        self.base.inode().inode_num
    }

    fn name(&self) -> String {
        self.base.name()
    }

    fn stat(&self) -> Stat {
        todo!()
    }
}

impl<D> IFile for Ext2File<D>
where
    D: 'static + BlockDevice,
{
    fn size(&self) -> u64 {
        self.base.inode().size()
    }

    fn truncate(&mut self, _size: u64) -> Result<()> {
        todo!()
    }

    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
        let buffer = buf.as_mut();

        let block_size = self.base.fs().read().superblock.block_size as usize;

        let start_block = offset as u32 / block_size as u32;
        let end_block = (offset as u32 + buffer.len() as u32) / block_size as u32;
        let relative_offset = offset as usize % block_size;
        let block_count = if buffer.len() % block_size == 0 {
            end_block - start_block
        } else {
            end_block - start_block + 1
        } as usize;

        // read blocks
        let mut data: Vec<u8> = vec![0_u8; block_count * block_size];
        let guard = self.base.fs().read();
        for i in 0..block_count {
            let read_block_index = (start_block + i as u32) as usize;
            let block_pointer = match self.determine_block_pointer_type(read_block_index) {
                BlockPointerType::Direct => self.base.inode().direct_pointers[read_block_index],
                BlockPointerType::SingleIndirect => {
                    let single_indirect_block_index = read_block_index - 12; // 12 direct pointers, so subtract the 12
                    let pointer_block = self.base.inode().singly_indirect_pointer;
                    let pointer_block_address = guard.get_block_address(pointer_block);
                    let mut pointer_data = [0_u8; 4];
                    let pointer_start_address =
                        pointer_block_address + single_indirect_block_index as u64 * 4;
                    guard
                        .device
                        .read_at(pointer_start_address, &mut pointer_data)?;
                    u32::from_le_bytes(pointer_data)
                }
                BlockPointerType::DoubleIndirect => {
                    todo!("double indirect pointers")
                }
                BlockPointerType::TripleIndirect => {
                    todo!("triple indirect pointers")
                }
            };
            let block_address = guard.get_block_address(block_pointer);

            let start_index = i * block_size;
            let end_index = start_index + block_size;
            guard
                .device
                .read_at(block_address, &mut &mut data[start_index..end_index])?;
        }
        buffer.copy_from_slice(&data[relative_offset..relative_offset + buffer.len()]);

        Ok(buffer.len())
    }

    fn write_at(&mut self, _offset: u64, _buf: &dyn AsRef<[u8]>) -> Result<usize> {
        todo!()
    }
}

enum BlockPointerType {
    Direct,
    SingleIndirect,
    DoubleIndirect,
    TripleIndirect,
}
