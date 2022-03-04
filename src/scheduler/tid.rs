use core::{
    fmt::{Display, Formatter},
    sync::atomic::{AtomicU32, Ordering},
};

static TID_COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Tid(usize);

impl !Default for Tid {}

impl Tid {
    pub fn new() -> Self {
        Self::from(TID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl From<u32> for Tid {
    fn from(v: u32) -> Self {
        Self::from(v as usize)
    }
}

impl From<usize> for Tid {
    fn from(v: usize) -> Self {
        Self(v)
    }
}

impl Display for Tid {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
