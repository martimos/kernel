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
