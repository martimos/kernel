use alloc::boxed::Box;
use core::fmt::{Debug, Formatter};
use core::ops::{Index, IndexMut, Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};

const STACK_SIZE: usize = 4096;

pub struct Stack {
    data: Box<[u8; STACK_SIZE]>,
}

impl Stack {
    pub fn allocate() -> Self {
        Self {
            data: box [0; STACK_SIZE],
        }
    }

    pub fn top(&self) -> usize {
        &self.data[STACK_SIZE - 1] as *const _ as usize
    }

    pub fn bottom(&self) -> usize {
        &self.data[0] as *const _ as usize
    }

    pub const fn len(&self) -> usize {
        self.data.len()
    }

    pub fn write_at(&mut self, index: usize, data: &[u8]) {
        for (i, b) in data.iter().enumerate() {
            self[index + i] = *b;
        }
    }
}

impl Debug for Stack {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Stack")
            .field("bottom", &self.bottom())
            .field("top", &self.top())
            .field("len", &self.len())
            .finish()
    }
}

impl Index<usize> for Stack {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data.index(index)
    }
}

impl IndexMut<usize> for Stack {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}

impl Index<Range<usize>> for Stack {
    type Output = [u8];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.data.index(index)
    }
}

impl IndexMut<Range<usize>> for Stack {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}

impl Index<RangeFrom<usize>> for Stack {
    type Output = [u8];

    fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
        &self.data.index(index)
    }
}

impl IndexMut<RangeFrom<usize>> for Stack {
    fn index_mut(&mut self, index: RangeFrom<usize>) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}

impl Index<RangeTo<usize>> for Stack {
    type Output = [u8];

    fn index(&self, index: RangeTo<usize>) -> &Self::Output {
        &self.data.index(index)
    }
}

impl IndexMut<RangeTo<usize>> for Stack {
    fn index_mut(&mut self, index: RangeTo<usize>) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}

impl Index<RangeToInclusive<usize>> for Stack {
    type Output = [u8];

    fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
        &self.data.index(index)
    }
}

impl IndexMut<RangeToInclusive<usize>> for Stack {
    fn index_mut(&mut self, index: RangeToInclusive<usize>) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}

impl Index<RangeInclusive<usize>> for Stack {
    type Output = [u8];

    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        &self.data.index(index)
    }
}

impl IndexMut<RangeInclusive<usize>> for Stack {
    fn index_mut(&mut self, index: RangeInclusive<usize>) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test_case]
    fn test_stack_top_bottom() {
        let stack = Stack::allocate();
        let bottom = stack.bottom();
        let top = stack.top();

        assert_eq!(bottom + STACK_SIZE - 1, top);
    }

    #[test_case]
    fn test_stack_index_mut() {
        let mut stack = Stack::allocate();
        stack[0] = 1;
        stack[1] = 2;
        stack[2] = 3;

        assert_eq!(1, stack[0]);
        assert_eq!(2, stack[1]);
        assert_eq!(3, stack[2]);
    }

    #[test_case]
    fn test_stack_write_at() {
        let mut stack = Stack::allocate();

        stack.write_at(4, &[1, 2, 3, 4, 5]);

        assert_eq!(stack[0], 0);
        assert_eq!(stack[1], 0);
        assert_eq!(stack[2], 0);
        assert_eq!(stack[3], 0);
        assert_eq!(stack[4], 1);
        assert_eq!(stack[5], 2);
        assert_eq!(stack[6], 3);
        assert_eq!(stack[7], 4);
        assert_eq!(stack[8], 5);
        assert_eq!(stack[9], 0);
    }

    #[test_case]
    fn test_stack_write_at_u64_be() {
        let mut stack = Stack::allocate();

        stack.write_at(1, &97_u64.to_be_bytes());

        assert_eq!(stack[0], 0);
        assert_eq!(stack[1], 0);
        assert_eq!(stack[2], 0);
        assert_eq!(stack[3], 0);
        assert_eq!(stack[4], 0);
        assert_eq!(stack[5], 0);
        assert_eq!(stack[6], 0);
        assert_eq!(stack[7], 0);
        assert_eq!(stack[8], 97);
        assert_eq!(stack[9], 0);
    }

    #[test_case]
    fn test_stack_write_at_u64_le() {
        let mut stack = Stack::allocate();

        stack.write_at(1, &97_u64.to_le_bytes());

        assert_eq!(stack[0], 0);
        assert_eq!(stack[1], 97);
        assert_eq!(stack[2], 0);
        assert_eq!(stack[3], 0);
        assert_eq!(stack[4], 0);
        assert_eq!(stack[5], 0);
        assert_eq!(stack[6], 0);
        assert_eq!(stack[7], 0);
        assert_eq!(stack[8], 0);
        assert_eq!(stack[9], 0);
    }
}
