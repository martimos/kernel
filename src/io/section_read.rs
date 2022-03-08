use crate::io::read::Read;
use crate::io::read_at::ReadAt;
use crate::Result;

pub struct SectionRead<'a, T> {
    inner: &'a T,
    offset: u64,
}

impl<'a, T> SectionRead<'a, T>
where
    T: ReadAt,
{
    pub fn new(inner: &'a T, offset: u64) -> Self {
        Self { inner, offset }
    }
}

impl<T> Read for SectionRead<'_, T>
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
