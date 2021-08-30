use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

use crate::multitasking::thread::stack::Stack;
use crate::multitasking::Work;

mod stack;

pub struct Thread {
    pub id: ThreadId,
    pub priority: Priority,
    pub state: State,
    stack: Stack,
    pub stack_pointer: usize,
    pub work: Option<Work>,
}

impl Thread {
    pub fn new(work: Work, priority: Priority) -> Self {
        Self {
            id: ThreadId::new(),
            priority,
            state: State::Ready,
            stack: Stack::allocate(),
            stack_pointer: 0,
            work: Some(work),
        }
    }

    pub fn set_state(&mut self, new: State) {
        self.state = new;
    }

    pub fn stack_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }

    pub fn set_stack_pointer(&mut self, new: usize) {
        self.stack_pointer = new
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Priority {
    Realtime,
    High,
    Normal,
    Low,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ThreadId(usize);

impl ThreadId {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        ThreadId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum State {
    Invalid,
    Ready,
    Running,
    Blocked,
    Finished,
    Idle,
}

impl Default for State {
    fn default() -> Self {
        State::Invalid
    }
}
