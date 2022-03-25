use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec;

use spin::RwLock;

use crate::device::block::BlockDevice;
use crate::io::cursor::Cursor;
use crate::io::fs::ext2::block_group::BlockGroupDescriptorTable;
use crate::io::fs::ext2::inode::{Ext2DirEntry, Ext2INode};
use crate::io::fs::ext2::superblock::Superblock;
use crate::io::fs::{Fs, INode, INodeBase, INodeNum, Stat};
use crate::io::ReadAt;
use crate::syscall::error::Errno;
use crate::{debug, Result};

mod block_group;
mod inode;
mod superblock;

#[derive(Debug)]
struct Ext2INodeAddress(u32);

#[derive(Debug)]
struct InvalidINodeAddress;

impl TryFrom<u32> for Ext2INodeAddress {
    type Error = InvalidINodeAddress;

    fn try_from(value: u32) -> core::result::Result<Self, Self::Error> {
        if value == 0 {
            return Err(InvalidINodeAddress);
        }
        Ok(Ext2INodeAddress(value))
    }
}

pub struct Ext2Fs<D>
where
    D: BlockDevice,
{
    inner: Rc<RwLock<Inner<D>>>,
}

impl<D> Ext2Fs<D>
where
    D: BlockDevice,
{
    pub fn new(device: D) -> Result<Self> {
        let superblock = {
            let mut superblock_buf = vec![0_u8; 1024];
            device.read_at(1024, &mut superblock_buf)?;
            let mut cursor = Cursor::new(superblock_buf);
            Superblock::decode(&mut cursor)?
        };
        debug!("read superblock: {:#?}", superblock);

        let number_of_block_groups = {
            let chck1 = (superblock.num_blocks + superblock.blocks_per_group - 1)
                / superblock.blocks_per_group;
            let chck2 = (superblock.num_inodes + superblock.inodes_per_group - 1)
                / superblock.inodes_per_group;
            if chck1 != chck2 {
                return Err(Errno::EIO);
            }
            chck1
        };

        let block_group_descriptor_table = {
            let mut block_group_descriptor_buf = vec![0_u8; superblock.block_size as usize];
            device.read_at(2048, &mut block_group_descriptor_buf)?;
            let mut cursor = Cursor::new(block_group_descriptor_buf);
            BlockGroupDescriptorTable::decode(&mut cursor, number_of_block_groups as usize)?
        };
        debug!(
            "read block group descriptor table: {:#?}",
            block_group_descriptor_table
        );

        let block_size = superblock.block_size;

        let inner = Inner {
            device,
            superblock,
            block_group_descriptor_table,
        };

        {
            let root_inode = inner.read_inode(2_u32.try_into().unwrap()).unwrap();
            debug!("root inode: {:#?}", root_inode);
            let direntry_block = root_inode.direct_pointers[0];
            let mut data = vec![0_u8; block_size as usize];
            inner
                .device
                .read_at(inner.get_block_address(direntry_block), &mut data)?;
            let mut cursor = Cursor::new(data);
            let direntry = Ext2DirEntry::decode(&mut cursor);
            debug!("direntry: {:#?}", direntry);
        }

        Ok(Self {
            inner: Rc::new(RwLock::new(inner)),
        })
    }
}

impl<D> Fs for Ext2Fs<D>
where
    D: BlockDevice,
{
    fn root_inode(&self) -> INode {
        todo!()
    }
}

struct Inner<D>
where
    D: BlockDevice,
{
    device: D,

    superblock: Superblock,
    block_group_descriptor_table: BlockGroupDescriptorTable,
}

impl<D> Inner<D>
where
    D: BlockDevice,
{
    fn superblock(&self) -> &'_ Superblock {
        &self.superblock
    }

    fn superblock_mut(&mut self) -> &'_ mut Superblock {
        &mut self.superblock
    }

    fn block_group_descriptor_table(&self) -> &'_ BlockGroupDescriptorTable {
        &self.block_group_descriptor_table
    }

    fn block_group_descriptor_table_mut(&mut self) -> &'_ mut BlockGroupDescriptorTable {
        &mut self.block_group_descriptor_table
    }

    fn read_inode(&self, inode: Ext2INodeAddress) -> Result<Ext2INode> {
        debug!("read inode: {:?}", inode);
        let block_group_index = (inode.0 - 1) / self.superblock.inodes_per_group;
        let block_group = &self.block_group_descriptor_table[block_group_index as usize];
        let itable_start_block = block_group.inode_table_starting_block;

        let index = (inode.0 - 1) % self.superblock.inodes_per_group;
        let inode_size = self.superblock.inode_size();
        let address =
            self.get_block_address(itable_start_block) + (index * inode_size as u32) as u64;

        let mut inode_buffer = vec![0_u8; inode_size as usize];
        self.device.read_at(address, &mut inode_buffer)?;
        let mut cursor = Cursor::new(inode_buffer);
        Ext2INode::decode(&mut cursor)
    }

    fn get_block_address(&self, block: u32) -> u64 {
        (1024 + (block - 1) * self.superblock.block_size) as u64
    }
}

struct Ext2NodeBase {}

impl INodeBase for Ext2NodeBase {
    fn num(&self) -> INodeNum {
        todo!()
    }

    fn name(&self) -> String {
        todo!()
    }

    fn stat(&self) -> Stat {
        todo!()
    }
}

pub struct Ext2File {
    base: Ext2NodeBase,
}

impl INodeBase for Ext2File {
    fn num(&self) -> INodeNum {
        self.base.num()
    }

    fn name(&self) -> String {
        self.base.name()
    }

    fn stat(&self) -> Stat {
        self.base.stat()
    }
}

pub struct Ext2Dir {
    base: Ext2NodeBase,
}

impl INodeBase for Ext2Dir {
    fn num(&self) -> INodeNum {
        self.base.num()
    }

    fn name(&self) -> String {
        self.base.name()
    }

    fn stat(&self) -> Stat {
        self.base.stat()
    }
}
