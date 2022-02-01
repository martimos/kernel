use crate::filesystem::file_descriptor::{FileDescriptor, Seek};
use crate::filesystem::flags::{Mode, OpenFlags};
use crate::filesystem::stat::Stat;
use crate::filesystem::{FileSystem, FsId};
use crate::syscall::error::Errno;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// A file system that has all files and will
/// fail any write operation with ENOSPC.
/// Opening a file wall return a file descriptor
/// matching the requested file, but writes will
/// fail with ENOSPC.
pub struct EmptyFileSystem {
    fsid: FsId,
}

impl EmptyFileSystem {
    pub fn from(fsid: FsId) -> Self {
        EmptyFileSystem { fsid }
    }
}

impl FileSystem for EmptyFileSystem {
    fn fsid(&self) -> FsId {
        self.fsid
    }

    fn initialize(&self) -> bool {
        true
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn open(
        &self,
        path: &'static str,
        _mode: Mode,
        _flags: OpenFlags,
    ) -> Result<Box<dyn FileDescriptor>, Errno> {
        Ok(Box::new(EmptyFileDescriptor::from(path)))
    }

    fn mkdir(&self, _path: &str, _mode: Mode) -> Result<(), Errno> {
        Err(Errno::ENOSPC)
    }

    fn rmdir(&self, _path: &str) -> Result<(), Errno> {
        Err(Errno::ENOENT)
    }

    fn flush(&self) {}
}

pub struct EmptyFileDescriptor {
    path: &'static str,
}

impl EmptyFileDescriptor {
    pub fn from(path: &'static str) -> EmptyFileDescriptor {
        EmptyFileDescriptor { path }
    }
}

impl FileDescriptor for EmptyFileDescriptor {
    fn is_readable(&self) -> bool {
        true
    }

    fn is_writable(&self) -> bool {
        true
    }

    fn seek(&self, _offset: isize, _whence: Seek) -> Result<usize, Errno> {
        todo!()
    }

    fn read(&mut self, _buffer: Vec<u8>) -> Result<usize, Errno> {
        Ok(0)
    }

    fn write(&mut self, _buffer: Vec<u8>) -> Result<usize, Errno> {
        Err(Errno::ENOSPC)
    }

    fn stat(&self) -> Result<Stat, Errno> {
        Ok(Stat {
            device_id: 0,
            inode_number: 0,
            access_mode: Mode::from(0o777),
            num_hard_links: 0,
            owner_uid: 0,
            owner_gid: 0,
            special: false,
            size: 0,
            block_size: 0,
            block_count: 0,
        })
    }

    fn absolute_path(&self) -> &str {
        self.path
    }
}
