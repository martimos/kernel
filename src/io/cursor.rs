use crate::io::read::Read;
use crate::io::ReadAt;
use crate::Result;

pub struct Cursor<T> {
    inner: T,
    offset: u64,
}

impl<T> Cursor<T>
where
    T: ReadAt,
{
    pub fn new(inner: T) -> Self {
        Self { inner, offset: 0 }
    }

    pub fn with_offset(inner: T, offset: u64) -> Self {
        Self { inner, offset }
    }

    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }
}

impl<T> Read for Cursor<T>
where
    T: ReadAt,
{
    fn read(&mut self, buf: &mut dyn AsMut<[u8]>) -> Result<usize> {
        match self.inner.read_at(self.offset, buf) {
            Ok(n) => {
                self.offset += n as u64;
                Ok(n)
            }
            r @ Err(_) => r,
        }
    }
}
