use core::sync::atomic::{AtomicUsize, Ordering};

pub struct ProcessControlBlock {
    pid: Id,
}

impl ProcessControlBlock {
    pub fn pid(&self) -> Id {
        self.pid
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Id(usize);

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl Id {
    pub fn new() -> Self {
        Id(ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub(crate) fn from(id: usize) -> Self {
        Id(id)
    }
}

impl Default for Id {
    fn default() -> Self {
        Id::new()
    }
}
