use crate::filesystem::stat::Stat;
use crate::Result;
use alloc::string::String;

pub enum Seek {
    Set(usize),
    Cur(usize),
    End(usize),
}

pub trait FileDescriptor {
    fn is_readable(&self) -> bool;
    fn is_writable(&self) -> bool;

    fn seek(&self, _seek: Seek) -> Result<usize>;

    fn read(&mut self, _buffer: &mut dyn AsRef<[u8]>) -> Result<usize>;
    fn write(&mut self, _buffer: &dyn AsRef<[u8]>) -> Result<usize>;
    fn stat(&self) -> Result<Stat>;

    fn absolute_path(&self) -> String;
}
