use alloc::vec::Vec;

use crate::Result;

pub mod from_read;
pub mod fs;
pub mod read;

pub trait ReadAt {
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> crate::Result<usize>;
}

impl ReadAt for Vec<u8> {
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
        let buffer = buf.as_mut();

        buffer.copy_from_slice(&self[offset as usize..offset as usize + buffer.len()]);
        Ok(buffer.len())
    }
}

pub trait WriteAt {
    fn write_at(&mut self, offset: u64, buf: &dyn AsRef<[u8]>) -> crate::Result<usize>;
}
