use alloc::string::String;

use crate::io::fs::devfs::DevFsNodeBase;
use crate::io::fs::{ICharacterDeviceFile, INodeBase, INodeNum, Stat};
use kstd::io::Result;

pub struct Serial {
    base: DevFsNodeBase,
}

impl Serial {
    pub fn new(inode_num: INodeNum) -> Self {
        Self {
            base: DevFsNodeBase {
                name: "serial".into(),
                stat: Stat {
                    inode: inode_num,
                    ..Default::default()
                },
            },
        }
    }
}

impl INodeBase for Serial {
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

impl ICharacterDeviceFile for Serial {
    fn read_at(&self, _: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
        let mut serial = crate::serial::SERIAL1.lock();
        let buffer = buf.as_mut();
        buffer.iter_mut().for_each(|b| *b = serial.receive());
        Ok(buffer.len())
    }

    fn write_at(&mut self, _: u64, buf: &dyn AsRef<[u8]>) -> Result<usize> {
        let buffer = buf.as_ref();
        let mut serial = crate::serial::SERIAL1.lock();
        for &b in buffer {
            serial.send(b);
        }
        Ok(buffer.len())
    }
}
