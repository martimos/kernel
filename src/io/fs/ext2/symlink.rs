use alloc::string::String;
use alloc::sync::Arc;

use kstd::io::block::BlockDevice;
use kstd::io::Result;
use kstd::sync::RwLock;

use crate::io::fs::ext2::base::Ext2NodeBase;
use crate::io::fs::ext2::inode::{Ext2INode, Ext2INodeType};
use crate::io::fs::ext2::Inner;
use crate::io::fs::{INodeBase, INodeNum, ISymlink, Stat};

pub struct Ext2Symlink<D>
where
    D: 'static + BlockDevice,
{
    base: Ext2NodeBase<D>,
}

impl<D> Ext2Symlink<D>
where
    D: 'static + BlockDevice,
{
    pub fn new(fs: Arc<RwLock<Inner<D>>>, ext2_inode: Ext2INode, name: String) -> Self {
        if ext2_inode.node_type != Ext2INodeType::SymbolicLink {
            panic!(
                "root inode is not a symlink, but a {:?}",
                ext2_inode.node_type
            );
        }

        Self {
            base: Ext2NodeBase::new(fs, ext2_inode, name),
        }
    }
}

impl<D> INodeBase for Ext2Symlink<D>
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

impl<D> ISymlink for Ext2Symlink<D>
where
    D: 'static + BlockDevice,
{
    fn target(&self) -> Result<String> {
        Ok(self.base.inode().symlink_short_name.clone())
    }
}
