use core::marker::PhantomData;

use crate::io::read::Read;
use crate::io::ReadAt;
use crate::Result;

pub struct Cursor<T, R> {
    inner: T,
    offset: u64,
    _result: PhantomData<R>,
}

impl<T, R> Cursor<T, R>
where
    T: ReadAt<R>,
{
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            offset: 0,
            _result: PhantomData::default(),
        }
    }

    pub fn with_offset(inner: T, offset: u64) -> Self {
        Self {
            inner,
            offset,
            _result: PhantomData::default(),
        }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn skip(&mut self, n: usize) {
        self.offset += n as u64;
    }
}

impl<T, R> Read<R> for Cursor<T, R>
where
    T: ReadAt<R>,
{
    fn read(&mut self, buf: &mut dyn AsMut<[R]>) -> Result<usize> {
        match self.inner.read_at(self.offset, buf) {
            Ok(n) => {
                self.offset += n as u64;
                Ok(n)
            }
            r @ Err(_) => r,
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::io::cursor::Cursor;
    use crate::io::read::Read;

    #[test_case]
    fn test_cursor_ref_vec_u8() {
        let data = vec![0_u8, 1, 2, 3, 4];
        let mut c = Cursor::new(&data);
        let mut buf = vec![0_u8; 3];
        assert_eq!(buf.len(), c.read(&mut buf).unwrap());
        assert_eq!(vec![0_u8, 1, 2], buf);
    }

    #[test_case]
    fn test_cursor_vec_u8() {
        let data = vec![0_u8, 1, 2, 3, 4];
        let mut c = Cursor::new(data);
        let mut buf = vec![0_u8; 3];
        assert_eq!(buf.len(), c.read(&mut buf).unwrap());
        assert_eq!(vec![0_u8, 1, 2], buf);
    }

    #[test_case]
    fn test_multiple_read() {
        let data: Vec<u8> = (0_u8..100).into_iter().collect();
        let mut c = Cursor::new(&data);
        let mut buf = vec![0_u8; 10];
        for i in 0_u8..10 {
            let start = i * 10;
            let end = start + 10;
            let expected: Vec<u8> = (start..end).into_iter().collect();

            assert_eq!(buf.len(), c.read(&mut buf).unwrap());
            assert_eq!(expected, buf);
        }
    }

    #[test_case]
    fn test_skip() {
        let data: Vec<u8> = (0_u8..100).into_iter().collect();
        let mut c = Cursor::new(&data);
        let mut buf = vec![0_u8; 5];

        c.skip(15);
        assert_eq!(buf.len(), c.read(&mut buf).unwrap());
        assert_eq!(vec![15_u8, 16, 17, 18, 19], buf);
    }
}
