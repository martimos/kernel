use alloc::string::String;

use crate::io::fs::devfs::DevFsNodeBase;
use crate::io::fs::{IFile, INodeBase, INodeNum, Stat};
use crate::syscall::error::Errno;
use crate::Result;

pub struct Null {
    base: DevFsNodeBase,
}

impl Null {
    pub fn new(inode_num: INodeNum) -> Self {
        Self {
            base: DevFsNodeBase {
                name: "null".into(),
                stat: Stat {
                    inode: inode_num,
                    ..Default::default()
                },
            },
        }
    }
}

impl INodeBase for Null {
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

impl IFile for Null {
    fn size(&self) -> u64 {
        0
    }

    fn truncate(&mut self, _: u64) -> Result<()> {
        Ok(())
    }

    fn read_at(&self, _: u64, _: &mut dyn AsMut<[u8]>) -> Result<usize> {
        Err(Errno::ENOSYS)
    }

    fn write_at(&mut self, _: u64, buf: &dyn AsRef<[u8]>) -> Result<usize> {
        Ok(buf.as_ref().len())
    }
}
