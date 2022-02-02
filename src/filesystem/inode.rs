use crate::Result;

pub trait Inode {
    fn read_at(&self, off: u64, buf: &mut dyn AsMut<[u8]>) -> Result<u64>;
    fn write_at(&mut self, off: u64, buf: &dyn AsRef<[u8]>) -> Result<u64>;

    fn size(&self) -> usize;
}
