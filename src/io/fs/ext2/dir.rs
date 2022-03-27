use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use spin::RwLock;

use crate::device::block::BlockDevice;
use crate::io::cursor::Cursor;
use crate::io::fs::ext2::base::Ext2NodeBase;
use crate::io::fs::ext2::file::Ext2File;
use crate::io::fs::ext2::inode::{Ext2DirEntry, Ext2INode, Ext2INodeType};
use crate::io::fs::ext2::{Ext2INodeAddress, Inner};
use crate::io::fs::{IDir, INode, INodeBase, INodeNum, INodeType, Stat};
use crate::io::ReadAt;
use crate::syscall::error::Errno;
use crate::Result;

pub struct Ext2Dir<D>
where
    D: 'static + BlockDevice,
{
    base: Ext2NodeBase<D>,
}

impl<D> Ext2Dir<D>
where
    D: 'static + BlockDevice,
{
    pub fn new(fs: Rc<RwLock<Inner<D>>>, ext2_inode: Ext2INode, name: String) -> Self {
        if ext2_inode.node_type != Ext2INodeType::Directory {
            panic!(
                "root inode is not a directory, but a {:?}",
                ext2_inode.node_type
            );
        }

        Self {
            base: Ext2NodeBase::new(fs, ext2_inode, name),
        }
    }

    fn list_dir_entries(&self) -> Result<Vec<Ext2DirEntry>> {
        let block_size = self.base.fs().read().superblock.block_size as usize;
        let inode = self.base.inode();

        let mut entries = Vec::with_capacity(inode.num_hard_links as usize);
        for &block in inode.direct_pointers.iter().filter(|&&p| p != 0) {
            let mut data = vec![0_u8; block_size];
            let data_len = data.len();
            let block_address = self.base.fs().read().get_block_address(block);
            self.base
                .fs()
                .read()
                .device
                .read_at(block_address, &mut data)?;
            let mut cursor = Cursor::new(data);

            // read all dir entries in this block
            while cursor.offset() < data_len as u64 {
                entries.push(Ext2DirEntry::decode(&mut cursor)?);
            }
        }
        if inode.singly_indirect_pointer != 0 {
            panic!("singly indirect pointer not supported yet");
        }
        if inode.doubly_indirect_pointer != 0 {
            panic!("doubly indirect pointer not supported yet");
        }
        if inode.triply_indirect_pointer != 0 {
            panic!("triply indirect pointer not supported yet");
        }

        Ok(entries)
    }
}

impl<D> INodeBase for Ext2Dir<D>
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

impl<D> IDir for Ext2Dir<D>
where
    D: 'static + BlockDevice,
{
    fn lookup(&self, name: &dyn AsRef<str>) -> Result<INode> {
        let entry = match self
            .list_dir_entries()?
            .into_iter()
            .find(|e| e.name == name.as_ref())
        {
            None => Err(Errno::ENOENT),
            Some(e) => Ok(e),
        }?;
        let inode_address = Ext2INodeAddress::try_from(entry.inode).or(Err(Errno::EFAULT))?;
        let ext2_inode = self.base.fs().read().read_inode(inode_address)?;
        let inode = self.create_inode(ext2_inode, entry.name)?;
        Ok(inode)
    }

    fn create(&mut self, _name: &dyn AsRef<str>, _typ: INodeType) -> Result<INode> {
        todo!()
    }

    fn children(&self) -> Result<Vec<INode>> {
        let entries = self.list_dir_entries()?;
        let mut result = Vec::with_capacity(entries.len());
        for entry in entries {
            let inode_address = Ext2INodeAddress::try_from(entry.inode).unwrap();
            let ext2_inode = self.base.fs().read().read_inode(inode_address)?;
            let inode = self.create_inode(ext2_inode, entry.name)?;
            result.push(inode);
        }
        Ok(result)
    }

    fn mount(&mut self, _node: INode) -> Result<()> {
        todo!()
    }
}

impl<D> Ext2Dir<D>
where
    D: 'static + BlockDevice,
{
    fn create_inode(&self, ext2_inode: Ext2INode, name: String) -> Result<INode> {
        let inode = match ext2_inode.node_type {
            Ext2INodeType::Directory => {
                INode::new_dir(Ext2Dir::new(self.base.fs().clone(), ext2_inode, name))
            }
            Ext2INodeType::RegularFile => {
                INode::new_file(Ext2File::new(self.base.fs().clone(), ext2_inode, name))
            }
            _ => return Err(Errno::ENOSYS),
        };
        Ok(inode)
    }
}
