use alloc::boxed::Box;
use core::fmt::{Debug, Formatter};
use core::time::Duration;

use x86_64::{PhysAddr, VirtAddr};

use crate::multitasking::process;

type Work = Box<dyn 'static + FnOnce() + Send + Sync>;

pub struct ThreadControlBlock {
    stack: Box<[u8; 4096]>,
    state: State,
    work: Option<Work>,
    cpu_time: Duration,
}

impl ThreadControlBlock {
    pub fn new(work: Work) -> Self {
        Self {
            stack: box [0; 4096],
            state: State::default(),
            work: Some(work),
            cpu_time: Duration::default(),
        }
    }

    pub unsafe fn from_raw(stack_addr: PhysAddr) -> Self {
        let stack = Box::from_raw(stack_addr.as_u64() as *mut [u8; 4096]);
        Self {
            stack,
            state: State::default(),
            work: None,
            cpu_time: Duration::default(),
        }
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }
}

impl Debug for ThreadControlBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ThreadControlBlock")
            .field("stack_address", &(&self.stack as *const _ as u64))
            .field("work_address", &(&self.work as *const _ as u64))
            .field("cpu_time", &self.cpu_time)
            .finish()
    }
}

#[derive(Debug)]
pub enum State {
    Ready,
    Running,
    Wait,
    Start,
    Done,
}

impl Default for State {
    fn default() -> Self {
        State::Ready
    }
}