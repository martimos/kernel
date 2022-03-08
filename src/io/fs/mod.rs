use alloc::boxed::Box;

use crate::io::fs::perm::Permission;
use crate::io::fs::{file_descriptor::FileDescriptor, flags::OpenFlags, path::Path};
use crate::Result;

pub mod fat32;
pub mod file_descriptor;
pub mod flags;
pub mod inode;
pub mod memfs;
pub mod path;
pub mod perm;
pub mod stat;
pub mod upnode;
pub mod ustar;
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

    fn open(
        &mut self,
        path: &dyn AsRef<Path>,
        perm: Permission,
        flags: OpenFlags,
    ) -> Result<Box<dyn FileDescriptor>>;

    fn mkdir(&self, path: &dyn AsRef<Path>, mode: Permission) -> Result<()>;

    fn rmdir(&self, path: &dyn AsRef<Path>) -> Result<()>;
}
