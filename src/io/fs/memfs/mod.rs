use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::vec;

use spin::Mutex;

use crate::io::fs::file_descriptor::FileDescriptor;
use crate::io::fs::flags::OpenFlags;
use crate::io::fs::memfs::memfd::MemFd;
use crate::io::fs::memfs::memfile::{MemFile, RefcountMemFile};
use crate::io::fs::path::owned::OwnedPath;
use crate::io::fs::path::Path;
use crate::io::fs::perm::Permission;
use crate::io::fs::{FileSystem, FsId};
use crate::syscall::error::Errno;
use crate::Result;

pub mod memfd;
pub mod memfile;

struct MemFsEntry {
    file: RefcountMemFile,
    _perm: Permission,
}

pub struct MemFs {
    id: FsId,
    files: BTreeMap<OwnedPath, MemFsEntry>,
}

impl MemFs {
    pub fn new<I: Into<FsId>>(id: I) -> Self {
        MemFs {
            id: id.into(),
            files: BTreeMap::new(),
        }
    }
}

impl FileSystem for MemFs {
    fn fsid(&self) -> FsId {
        self.id
    }

    fn open(
        &mut self,
        path: &dyn AsRef<Path>,
        perm: Permission,
        flags: OpenFlags,
    ) -> Result<Box<dyn FileDescriptor>> {
        let p = path.as_ref().to_owned();

        if let Some(entry) = self.files.get(&p) {
            return Ok(Box::new(MemFd::new(entry.file.clone())));
        }

        if !flags.contains(OpenFlags::O_CREAT) {
            return Err(Errno::ENOENT);
        }

        // create the file
        let file = Rc::new(Mutex::new(MemFile::new(vec![])));
        let entry = MemFsEntry {
            file: file.clone(),
            _perm: perm,
        };

        // store the file in the fs
        self.files.insert(p, entry);

        // create the file descriptor that's being returned
        let fd = MemFd::new(file);
        Ok(Box::new(fd))
    }

    fn mkdir(&self, _path: &dyn AsRef<Path>, _mode: Permission) -> Result<()> {
        todo!()
    }

    fn rmdir(&self, _path: &dyn AsRef<Path>) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::io::fs::file_descriptor::Seek;

    use super::*;

    #[test_case]
    fn test_foo_for_starters() {
        let mut fs = MemFs::new(1);
        let path = "hello.txt";
        let f_res = fs.open(&path, Permission::empty(), OpenFlags::O_CREAT);
        let mut f = f_res.expect("unable to open file");

        let write_data = "Hello, World!";
        assert_eq!(13, f.write(&write_data).unwrap());
        f.seek(Seek::Set(0)).unwrap();

        let mut data = vec![0_u8; 10];
        assert_eq!(10, f.read(&mut data).unwrap());
        assert_eq!(data, "Hello, Wor".as_bytes());
        assert_eq!(3, f.read(&mut data).unwrap());
        assert_eq!(&data[..3], "ld!".as_bytes());
    }
}