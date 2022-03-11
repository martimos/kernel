pub trait ReadAt {
    fn read_at(&self, offset: u64, buf: &mut dyn AsMut<[u8]>) -> crate::Result<usize>;
}
