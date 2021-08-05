use crate::filesystem::file_descriptor::FileDescriptor;
use crate::filesystem::flags::{Mode, OpenFlags};
use crate::syscall::error::Errno;
use alloc::boxed::Box;

pub mod file_descriptor;
pub mod flags;
pub mod inode;
pub mod stat;
pub mod upnode;
pub mod vfs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FsId(u32);

impl From<u32> for FsId {
    fn from(fsid: u32) -> Self {
        FsId(fsid)
    }
}

pub trait FileSystem {
    fn fsid(&self) -> FsId;

    fn initialize(&self) -> bool;

    fn is_read_only(&self) -> bool;

    fn open(
        &self,
        path: &'static str,
        mode: Mode,
        flags: OpenFlags,
    ) -> Result<Box<dyn FileDescriptor>, Errno>;

    fn mkdir(&self, path: &str, mode: Mode) -> Result<(), Errno>;

    fn rmdir(&self, path: &str) -> Result<(), Errno>;

    fn flush(&self);
}
