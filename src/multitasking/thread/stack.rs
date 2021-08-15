use alloc::boxed::Box;

const SIZE: usize = 4096;

pub struct Stack {
    data: Box<[u8; SIZE]>,
}

impl Stack {
    pub fn allocate() -> Self {
        Stack {
            data: box [0; SIZE],
        }
    }

    pub fn data_address(&self) -> usize {
        self.data.as_ptr() as *const () as usize
    }

    pub const fn len(&self) -> usize {
        self.data.len()
    }

    pub fn write(&mut self, index: usize, data: &[u8]) {
        for (i, b) in data.iter().enumerate() {
            self.data[index + i] = *b;
        }
    }
}
