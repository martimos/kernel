use crate::io::fs::{Fs, INode, INodeNum};
use crate::syscall::error::Errno;
use crate::Result;

pub struct NullFs;

impl Fs for NullFs {
    fn root_inode_num(&self) -> INodeNum {
        0_u64.into()
    }

    fn get_node(&self, _: INodeNum) -> Result<INode> {
        Err(Errno::ENOENT)
    }
}
