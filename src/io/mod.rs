use crate::Result;

pub mod from_read;
pub mod fs;
pub mod read;

pub trait ReadAt {
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> crate::Result<usize>;
}

pub trait WriteAt {
    fn write_at(&mut self, offset: u64, buf: &dyn AsRef<[u8]>) -> crate::Result<usize>;
}
