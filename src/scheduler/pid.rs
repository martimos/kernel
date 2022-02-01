use core::fmt::{Display, Formatter};
use core::sync::atomic::{AtomicU32, Ordering};

static PID_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Pid(usize);

impl Pid {
    pub fn new() -> Self {
        Self::from(PID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl From<u32> for Pid {
    fn from(v: u32) -> Self {
        Self::from(v as usize)
    }
}

impl From<usize> for Pid {
    fn from(v: usize) -> Self {
        Self(v)
    }
}

impl Display for Pid {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
