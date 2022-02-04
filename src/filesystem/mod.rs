use crate::{
    filesystem::{
        file_descriptor::FileDescriptor,
        flags::{Mode, OpenFlags},
        path::Path,
    },
    Result,
};
use alloc::boxed::Box;

pub mod fat32;
pub mod file_descriptor;
pub mod flags;
pub mod inode;
pub mod path;
pub mod stat;
pub mod upnode;

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
        mode: Mode,
        flags: OpenFlags,
    ) -> Result<Box<dyn FileDescriptor>>;

    fn mkdir(&self, path: &dyn AsRef<Path>, mode: Mode) -> Result<()>;

    fn rmdir(&self, path: &dyn AsRef<Path>) -> Result<()>;
}
