use core::sync::atomic::{AtomicUsize, Ordering};

pub struct ProcessControlBlock {
    pid: ProcessId,
}

impl ProcessControlBlock {
    pub fn pid(&self) -> ProcessId {
        self.pid
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct ProcessId(usize);

impl ProcessId {
    pub fn new() -> Self {
        static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        ProcessId(ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub(crate) fn from(id: usize) -> Self {
        ProcessId(id)
    }
}

impl Default for ProcessId {
    fn default() -> Self {
        ProcessId::new()
    }
}
