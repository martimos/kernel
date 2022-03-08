use crate::syscall::error::Errno;

pub trait ReadAt {
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> crate::Result<usize>;
}

impl<T> ReadAt for T
where
    T: AsRef<[u8]>,
{
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> crate::Result<usize> {
        let buffer = buf.as_mut();
        let len = buffer.len();
        let r = self.as_ref();

        if offset as usize + len > r.len() {
            return Err(Errno::EIO);
        }
        buffer.copy_from_slice(&r[offset as usize..offset as usize + len]);
        Ok(len)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use crate::io::read_at::ReadAt;

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

    #[test_case]
    fn test_read_out_of_bounds() {
        let data = vec![0_u8, 1];
        let mut buf = vec![0_u8; 5];

        assert!(data.read_at(4, &mut buf).is_err());
    }
}
