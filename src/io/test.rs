use core::marker::PhantomData;

use crate::io::read::Read;
use crate::io::ReadAt;
use crate::Result;

/// Can wrap a [`Read`] or [`ReadAt`]. Depending on which of the
/// two traits the wrapped type implements, this struct implements
/// the same. All calls on this struct to [`Read::read`] and
/// [`ReadAt::read_at`] are delegated to the wrapped object, but with
/// a single element buffer. This struct will always only implement at
/// most a single element.
/// This is useful for testing implementation that work with a [`Read`]
/// or [`ReadAt`], to ensure that they don't rely on the traits always
/// reading the full buffer.
pub struct SingleRead<T, R> {
    inner: T,
    _result: PhantomData<R>,
}

impl<T, R> SingleRead<T, R> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _result: PhantomData::default(),
        }
    }
}

impl<T, R> Read<R> for SingleRead<T, R>
where
    T: Read<R>,
{
    fn read(&mut self, buf: &mut dyn AsMut<[R]>) -> Result<usize> {
        let b = buf.as_mut();
        self.inner.read(&mut &mut b[0..1])
    }
}

impl<T, R> ReadAt<R> for SingleRead<T, R>
where
    T: ReadAt<R>,
{
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[R]>) -> Result<usize> {
        let b = buf.as_mut();
        self.inner.read_at(offset, &mut &mut b[0..1])
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::io::cursor::Cursor;
    use crate::io::read::Read;
    use crate::io::test::SingleRead;

    #[test_case]
    fn test_single_read() {
        let data = vec![0_u8, 1, 2, 3, 4, 5];
        let c = Cursor::new(data);
        let mut r = SingleRead::new(c);
        let mut buf = vec![0_u8; 3];
        assert_eq!(1, r.read(&mut buf).unwrap());
        assert_eq!(vec![0_u8, 0, 0], buf);
        assert_eq!(1, r.read(&mut buf).unwrap());
        assert_eq!(vec![1_u8, 0, 0], buf);
        assert_eq!(1, r.read(&mut buf).unwrap());
        assert_eq!(vec![2_u8, 0, 0], buf);
        assert_eq!(1, r.read(&mut buf).unwrap());
        assert_eq!(vec![3_u8, 0, 0], buf);
        assert_eq!(1, r.read(&mut buf).unwrap());
        assert_eq!(vec![4_u8, 0, 0], buf);
        assert_eq!(1, r.read(&mut buf).unwrap());
        assert_eq!(vec![5_u8, 0, 0], buf);
    }
}
