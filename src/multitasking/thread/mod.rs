use alloc::boxed::Box;
use core::fmt::{Debug, Formatter};
use core::mem::size_of;
use core::time::Duration;

use crate::hlt_loop;
use crate::multitasking::thread::stack::Stack;

mod stack;

type Work = Box<dyn 'static + FnOnce() + Send + Sync>;

pub trait TCB: Send + Sync {
    fn info_mut(&mut self) -> &mut TCBInfo;
    fn work(&mut self) -> Work;
}

#[derive(Debug)]
pub struct TCBInfo {
    pub(crate) stack_pointer: usize,

    state: State,
    cpu_time: Duration,
}

impl TCBInfo {
    pub fn new(stack_pointer: usize) -> Self {
        TCBInfo {
            stack_pointer,
            state: State::default(),
            cpu_time: Duration::default(),
        }
    }

    pub fn stack_pointer_mut(&mut self) -> &mut usize {
        &mut self.stack_pointer
    }

    pub fn stack_pointer(&self) -> usize {
        self.stack_pointer
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }
}

pub struct TCBImpl {
    info: TCBInfo,
    stack: Stack,
    work: Option<Work>,
}

impl TCBImpl {
    pub fn new(work: Work) -> Self {
        let mut stack = Stack::allocate();
        let thread_entry_addr: usize = thread_entry_point as *const () as usize;
        let mut index = stack.len() - size_of::<usize>();
        stack.write(index, thread_entry_addr.to_ne_bytes().as_slice());
        index -= size_of::<usize>() * 6; // skip the 6 callee saved registers

        index -= size_of::<usize>() * 1; // make space for CR2
        stack.write(index, 0_usize.to_ne_bytes().as_slice());

        index -= size_of::<usize>() * 1; // make space for RFLAGS
        stack.write(index, 0_usize.to_ne_bytes().as_slice());

        Self {
            info: TCBInfo::new(stack.data_address() + index - 1),
            stack,
            work: Some(work),
        }
    }
}

impl Debug for TCBImpl {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ThreadControlBlock")
            .field("stack_address", &(&self.stack as *const _ as u64))
            .field("work_address", &(&self.work as *const _ as u64))
            .field("info", &self.info)
            .finish()
    }
}

impl TCB for TCBImpl {
    fn info_mut(&mut self) -> &mut TCBInfo {
        &mut self.info
    }

    fn work(&mut self) -> Work {
        self.work.take().expect("TCB had no work")
    }
}

#[derive(Debug)]
pub struct BootstrapTCB {
    info: TCBInfo,
    stack_frame: Option<usize>,
}

impl BootstrapTCB {
    pub fn new() -> Self {
        Self {
            info: TCBInfo::new(0),
            stack_frame: None,
        }
    }
}

impl TCB for BootstrapTCB {
    fn info_mut(&mut self) -> &mut TCBInfo {
        &mut self.info
    }

    fn work(&mut self) -> Work {
        unreachable!("called work() on bootstrap TCB");
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

#[no_mangle]
pub extern "C" fn thread_entry_point() -> ! {
    crate::serial_println!("Thread made it to entry point!");

    // Access the thread's TCB and do work

    hlt_loop();
}
