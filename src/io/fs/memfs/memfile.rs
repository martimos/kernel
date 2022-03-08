use crate::io::fs::stat::Stat;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::ops::{Index, IndexMut};
use core::slice::SliceIndex;
use spin::Mutex;

pub type RefcountMemFile = Rc<Mutex<MemFile>>;

pub struct MemFile {
    buffer: Vec<u8>,
}

impl MemFile {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self { buffer }
    }

    pub fn stat(&self) -> Stat {
        Stat {
            device_id: 0,
            inode_number: 0,
            access_mode: Default::default(), // TODO: return correct access mode
            num_hard_links: 0,
            owner_uid: 0,
            owner_gid: 0,
            special: false,
            size: self.buffer.len(),
            block_size: 0,
            block_count: 0,
        }
    }

    pub fn resize(&mut self, new_len: usize) {
        self.buffer.resize(new_len, 0);
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
        let mut f = MemFile::new(vec![0_u8; 10]);
        for i in 0..10 {
            assert_eq!(0, f[i]);

            let v = (i * 2) as u8;
            f[i] = v;
            assert_eq!(v, f[i]);
        }
    }
}
