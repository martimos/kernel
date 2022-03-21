use alloc::string::String;

use crate::io::fs::devfs::DevFsNodeBase;
use crate::io::fs::{IFile, INodeBase, INodeNum, Stat};
use crate::syscall::error::Errno;
use crate::Result;

pub struct Zero {
    base: DevFsNodeBase,
}

impl Zero {
    pub fn new(inode_num: INodeNum) -> Self {
        Self {
            base: DevFsNodeBase {
                name: "zero".into(),
                stat: Stat {
                    inode: inode_num,
                    ..Default::default()
                },
            },
        }
    }
}

impl INodeBase for Zero {
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

impl IFile for Zero {
    fn size(&self) -> u64 {
        0
    }

    fn truncate(&mut self, _: u64) -> Result<()> {
        Err(Errno::ENOSYS)
    }

    fn read_at(&self, _: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
        let buffer = buf.as_mut();
        buffer.fill(0);
        Ok(buffer.len())
    }

    fn write_at(&mut self, _: u64, _: &dyn AsRef<[u8]>) -> Result<usize> {
        Err(Errno::ENOSYS)
    }
}
