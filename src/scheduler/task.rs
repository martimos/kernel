use crate::hlt_loop;
use crate::scheduler::pid::Pid;
use crate::scheduler::priority::{Priority, LOW_PRIORITY};
use crate::scheduler::{do_exit, STACK_SIZE};
use alloc::alloc::{alloc, dealloc};
use alloc::rc::Rc;
use core::alloc::Layout;
use core::cell::RefCell;
use core::mem::size_of;
use core::ptr::write_bytes;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProcessStatus {
    Invalid,
    Ready,
    Running,
    Blocked,
    Finished,
    Idle,
}

#[repr(align(64))]
#[repr(C)]
pub struct Stack {
    buffer: [u8; STACK_SIZE],
}

impl Stack {
    pub const fn new() -> Stack {
        Stack {
            buffer: [0; STACK_SIZE],
        }
    }

    pub fn top(&self) -> usize {
        (&(self.buffer[STACK_SIZE - 16]) as *const _) as usize
    }

    pub fn bottom(&self) -> usize {
        (&(self.buffer[0]) as *const _) as usize
    }
}

pub static mut BOOT_STACK: Stack = Stack::new();

/// A task control block, which identifies either a process or a thread
#[repr(align(64))]
pub struct Task {
    /// The ID of this context
    pub pid: Pid,
    /// Task Priority
    pub prio: Priority,
    /// Status of a task, e.g. if the task is ready or blocked
    pub status: ProcessStatus,
    /// Last stack pointer before a context switch to another task
    pub last_stack_pointer: usize,
    /// Stack of the task
    pub stack: *mut Stack,
    // next task in queue
    pub next: Option<Rc<RefCell<Task>>>,
    // previous task in queue
    pub prev: Option<Rc<RefCell<Task>>>,
}

impl Task {
    pub fn new_idle(id: Pid) -> Task {
        Task {
            pid: id,
            prio: LOW_PRIORITY,
            status: ProcessStatus::Idle,
            last_stack_pointer: 0,
            stack: unsafe { &mut BOOT_STACK },
            next: None,
            prev: None,
        }
    }

    pub fn new(id: Pid, status: ProcessStatus, prio: Priority) -> Task {
        let layout = Layout::new::<Stack>();
        let stack = unsafe { alloc(layout) as *mut Stack };
        if stack as usize == 0 {
            panic!(
                "unable to allocate another kernel stack of size {} (out of kernel memory)",
                layout.size()
            );
        }

        Task {
            pid: id,
            prio,
            status,
            last_stack_pointer: 0,
            stack,
            next: None,
            prev: None,
        }
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        if unsafe { self.stack != &mut BOOT_STACK } {
            // deallocate stack
            unsafe {
                dealloc(self.stack as *mut u8, Layout::new::<Stack>());
            }
        }
    }
}

#[repr(C, packed)]
struct State {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rdi: u64,
    rsi: u64,
    rbp: u64,
    rsp: u64,
    rbx: u64,
    rdx: u64,
    rcx: u64,
    rax: u64,
    rflags: u64,
    rip: u64,
}

extern "C" fn leave_task() -> ! {
    do_exit();
    hlt_loop()
}

impl Task {
    pub fn allocate_stack(&mut self, entry_point: extern "C" fn()) {
        unsafe {
            let mut stack: *mut u64 = ((*self.stack).top()) as *mut u64; // "write" qwords

            write_bytes((*self.stack).bottom() as *mut u8, 0xCD, STACK_SIZE); // fill the stack with 0xCD

            *stack = 0xCAFEBABEu64; // marker at stack bottom
            stack = (stack as usize - size_of::<u64>()) as *mut u64;

            /* the first-function-to-be-called's arguments, ... */
            //TODO: add arguments

            *stack = (leave_task as *const ()) as u64; // put return address on the stack
            stack = (stack as usize - size_of::<State>()) as *mut u64; // "allocate" one State

            let state: *mut State = stack as *mut State;
            write_bytes(state, 0x00, 1); // fill "allocated" State with 0x00

            (*state).rsp = (stack as usize + size_of::<State>()) as u64; // stack pointer now points to the State
            (*state).rbp = (*state).rsp + size_of::<u64>() as u64; // base pointer is the stack pointer

            (*state).rip = (entry_point as *const ()) as u64; // push the entry point as instruction pointer
            (*state).rflags = 0x1202u64;

            self.last_stack_pointer = stack as usize; // remember the stack in the TCB (Task)
        }
    }
}
