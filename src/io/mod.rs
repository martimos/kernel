use alloc::vec::Vec;

use crate::Result;

pub mod cursor;
pub mod fs;
pub mod read;
#[cfg(test)]
pub mod test;

pub trait ReadAt<T> {
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[T]>) -> crate::Result<usize>;
}

impl<T> ReadAt<T> for &Vec<T>
where
    T: Copy,
{
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[T]>) -> Result<usize> {
        let buffer = buf.as_mut();

        buffer.copy_from_slice(&self[offset as usize..offset as usize + buffer.len()]);
        Ok(buffer.len())
    }
}

impl<T> ReadAt<T> for Vec<T>
where
    T: Copy,
{
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[T]>) -> Result<usize> {
        ReadAt::<T>::read_at(&self, offset, buf)
    }
}

pub trait WriteAt<T> {
    fn write_at(&mut self, offset: u64, buf: &dyn AsRef<[T]>) -> Result<usize>;
}
