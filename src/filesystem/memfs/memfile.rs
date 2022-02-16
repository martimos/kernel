use crate::filesystem::stat::Stat;
use crate::RcMut;
use alloc::vec::Vec;
use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;

pub type RefcountMemFile = RcMut<MemFile>;

pub struct MemFile {
    info: Stat,
    buffer: Vec<u8>,
}

impl MemFile {
    pub fn new(info: Stat, buffer: Vec<u8>) -> Self {
        Self { info, buffer }
    }

    pub fn stat(&self) -> Stat {
        self.info
    }
}

impl<I: SliceIndex<[u8]>> Index<I> for MemFile {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.buffer[index]
    }
}

impl<I: SliceIndex<[u8]>> IndexMut<I> for MemFile {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.buffer[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test_case]
    fn test_memfile_index() {
        let mut f = MemFile::new(Default::default(), vec![0_u8; 10]);
        for i in 0..10 {
            assert_eq!(0, f[i]);

            let v = (i * 2) as u8;
            f[i] = v;
            assert_eq!(v, f[i]);
        }
    }
}
