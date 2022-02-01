use crate::scheduler::NO_PRIORITIES;
use core::fmt::{Display, Formatter};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub struct Priority(u8);

impl Priority {
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    const fn from(v: u8) -> Self {
        Self(v)
    }
}

impl Display for Priority {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(
            f,
            "{}",
            match self.0 {
                x if x < 16 => "Low",
                x if x < 24 => "Normal",
                x if x < 32 => "High",
                _ => "Invalid",
            }
        )
    }
}

pub const REALTIME_PRIORITY: Priority = Priority::from(NO_PRIORITIES as u8 - 1);
pub const HIGH_PRIORITY: Priority = Priority::from(24);
pub const NORMAL_PRIORITY: Priority = Priority::from(16);
pub const LOW_PRIORITY: Priority = Priority::from(0);
