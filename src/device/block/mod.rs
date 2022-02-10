pub trait BlockDevice {
    fn block_size(&self) -> usize;
    fn read_block(&mut self, block: u32, buf: &mut dyn AsMut<[u8]>);
}
