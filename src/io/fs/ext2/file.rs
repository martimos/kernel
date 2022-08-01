use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use kstd::sync::RwLock;

use crate::io::fs::ext2::base::Ext2NodeBase;
use crate::io::fs::ext2::inode::{Ext2INode, Ext2INodeType};
use crate::io::fs::ext2::Inner;
use crate::io::fs::{IFile, INodeBase, INodeNum, Stat};
use kstd::io::device::block::BlockDevice;
use kstd::io::ReadAt;
use kstd::io::Result;

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
    pub fn new(fs: Rc<RwLock<Inner<D>>>, ext2_inode: Ext2INode, name: String) -> Self {
        if ext2_inode.node_type != Ext2INodeType::RegularFile {
            panic!("root inode is not a file, but a {:?}", ext2_inode.node_type);
        }

        Self {
            base: Ext2NodeBase::new(fs, ext2_inode, name),
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
        let block_count = if relative_offset == 0 && start_block != end_block {
            end_block - start_block
        } else {
            end_block - start_block + 1
        } as usize;

        // read blocks
        let mut data: Vec<u8> = vec![0_u8; block_count * block_size];
        let guard = self.base.fs().read();
        for i in 0..block_count {
            let read_block_index = (start_block + i as u32) as usize;
            let block_pointer = self.base.inode().direct_pointers[read_block_index];
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
