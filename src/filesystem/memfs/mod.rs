use crate::filesystem::file_descriptor::FileDescriptor;
use crate::filesystem::flags::OpenFlags;
use crate::filesystem::path::Path;
use crate::filesystem::perm::Permission;
use crate::filesystem::{FileSystem, FsId};
use crate::Result;
use alloc::boxed::Box;

pub mod memfd;
pub mod memfile;

pub struct MemFs {
    id: FsId,
}

impl MemFs {
    pub fn new(id: FsId) -> Self {
        MemFs { id }
    }
}

impl FileSystem for MemFs {
    fn fsid(&self) -> FsId {
        self.id
    }

    fn open(
        &mut self,
        _path: &dyn AsRef<Path>,
        _mode: Permission,
        _flags: OpenFlags,
    ) -> Result<Box<dyn FileDescriptor>> {
        todo!()
    }

    fn mkdir(&self, _path: &dyn AsRef<Path>, _mode: Permission) -> Result<()> {
        todo!()
    }

    fn rmdir(&self, _path: &dyn AsRef<Path>) -> Result<()> {
        todo!()
    }
}
