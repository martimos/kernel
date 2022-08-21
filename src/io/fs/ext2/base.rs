use alloc::string::String;
use alloc::sync::Arc;

use kstd::sync::RwLock;

use crate::io::fs::ext2::inode::Ext2INode;
use crate::io::fs::ext2::Inner;
use kstd::io::block::BlockDevice;

pub struct Ext2NodeBase<D>
where
    D: BlockDevice,
{
    fs: Arc<RwLock<Inner<D>>>,
    inode: Ext2INode,
    name: String,
}

impl<D> Ext2NodeBase<D>
where
    D: BlockDevice,
{
    pub fn new(fs: Arc<RwLock<Inner<D>>>, inode: Ext2INode, name: String) -> Self {
        Self { fs, inode, name }
    }

    pub fn fs(&self) -> &Arc<RwLock<Inner<D>>> {
        &self.fs
    }

    pub fn inode(&self) -> &Ext2INode {
        &self.inode
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}
