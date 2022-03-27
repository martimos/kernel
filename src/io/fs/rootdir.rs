use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::io::fs::{IDir, INode, INodeBase, INodeNum, INodeType, Stat};
use crate::syscall::error::Errno;
use crate::Result;

/// A container that implements [`IDir`], but with a few restrictions.
/// * non-modifiable stat
/// * [`INode`]s can be mounted into this dir
/// * nothing can be created in this dir, only mounted
pub struct RootDir {
    name: String,
    stat: Stat,
    children: BTreeMap<String, INode>,
}

impl RootDir {
    pub fn new(name: String, stat: Stat) -> Self {
        Self {
            name,
            stat,
            children: BTreeMap::new(),
        }
    }
}

impl INodeBase for RootDir {
    fn num(&self) -> INodeNum {
        self.stat.inode
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn stat(&self) -> Stat {
        self.stat
    }
}

impl IDir for RootDir {
    fn lookup(&self, name: &dyn AsRef<str>) -> Result<INode> {
        if let Some(inode) = self.children.get(name.as_ref()) {
            return Ok(inode.clone());
        }
        Err(Errno::ENOENT)
    }

    fn create(&mut self, _: &dyn AsRef<str>, _: INodeType) -> Result<INode> {
        Err(Errno::ENOSYS)
    }

    fn children(&self) -> Result<Vec<INode>> {
        Ok(self.children.values().cloned().collect())
    }

    fn mount(&mut self, node: INode) -> Result<()> {
        if self.children.get(&node.name()).is_some() {
            return Err(Errno::EEXIST);
        }
        self.children.insert(node.name(), node);
        Ok(())
    }
}
