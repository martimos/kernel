use core::ops::Index;
use core::slice::SliceIndex;

pub mod fs;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Error {
    UnexpectedEOF,
}

pub trait ReadAt {
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize>;
}

impl<T> ReadAt for T
where
    T: Index<usize, Output = u8>,
{
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
        let buffer = buf.as_mut();
        let len = buffer.len();

        for i in 0..len {
            buffer[i] = self[offset as usize + i];
        }
        Ok(len)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    #[test_case]
    fn test_read_at_vec_u8() {
        let data = vec![0_u8, 1, 2];
        let mut buf = vec![0_u8; 3];
        assert_eq!(buf.len(), data.read_at(0, &mut buf).unwrap());
        assert_eq!(data, buf);
    }

    #[test_case]
    fn test_read_at_center() {
        let data = vec![0_u8, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut buf = vec![0_u8; 3];
        assert_eq!(buf.len(), data.read_at(4, &mut buf).unwrap());
        assert_eq!(vec![4, 5, 6], buf);
    }
}
