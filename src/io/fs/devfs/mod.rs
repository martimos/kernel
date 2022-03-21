use alloc::collections::BTreeMap;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, Ordering};

use crate::io::fs::devfs::null::Null;
use crate::io::fs::devfs::zero::Zero;
use crate::io::fs::{Fs, IDir, INode, INodeBase, INodeNum, INodeType, Stat};
use crate::syscall::error::Errno;
use crate::Result;

mod null;
mod zero;

pub struct DevFs {
    root: INode,
}

impl DevFs {
    pub fn new(root_node_name: String) -> Self {
        let cnt = AtomicU64::new(0);
        let next = || INodeNum::from(cnt.fetch_add(1, Ordering::SeqCst));

        let mut root = DevDir::new(next(), root_node_name);
        root.mount(INode::new_file(Zero::new(next()))).unwrap();
        root.mount(INode::new_file(Null::new(next()))).unwrap();

        Self {
            root: INode::new_dir(root),
        }
    }
}

impl Fs for DevFs {
    fn root_inode(&self) -> INode {
        self.root.clone()
    }
}

struct DevFsNodeBase {
    name: String,
    stat: Stat,
}

impl INodeBase for DevFsNodeBase {
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

struct DevDir {
    base: DevFsNodeBase,
    children: BTreeMap<String, INode>,
}

impl DevDir {
    fn new(inode_num: INodeNum, name: String) -> Self {
        Self {
            base: DevFsNodeBase {
                name,
                stat: Stat {
                    dev: 0,
                    inode: inode_num,
                    rdev: 0,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    size: 0,
                    atime: 0,
                    mtime: 0,
                    ctime: 0,
                    blksize: 0,
                    blocks: 0,
                },
            },
            children: BTreeMap::new(),
        }
    }
}

impl INodeBase for DevDir {
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

impl IDir for DevDir {
    fn lookup(&self, name: &dyn AsRef<str>) -> Result<INode> {
        match self.children.get(name.as_ref()) {
            None => Err(Errno::ENOENT),
            Some(n) => Ok(n.clone()),
        }
    }

    fn create(&mut self, _: &dyn AsRef<str>, _: INodeType) -> Result<INode> {
        Err(Errno::ENOSYS)
    }

    fn mount(&mut self, node: INode) -> Result<()> {
        let name = node.name();
        if self.lookup(&name).is_ok() {
            return Err(Errno::EEXIST);
        }
        self.children.insert(name, node);
        Ok(())
    }
}
