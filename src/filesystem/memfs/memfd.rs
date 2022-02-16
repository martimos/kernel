use crate::filesystem::file_descriptor::{FileDescriptor, Seek};
use crate::filesystem::memfs::memfile::RefcountMemFile;
use crate::filesystem::stat::Stat;
use crate::Result;
use alloc::string::String;

pub struct MemFd {
    file: RefcountMemFile,
}

impl FileDescriptor for MemFd {
    fn seek(&mut self, _seek: Seek) -> Result<usize> {
        todo!()
    }

    fn read(&mut self, _buffer: &mut dyn AsMut<[u8]>) -> Result<usize> {
        todo!()
    }

    fn write(&mut self, _buffer: &dyn AsRef<[u8]>) -> Result<usize> {
        todo!()
    }

    fn stat(&self) -> Result<Stat> {
        Ok(self.file.lock().stat())
    }

    fn absolute_path(&self) -> String {
        todo!()
    }
}
