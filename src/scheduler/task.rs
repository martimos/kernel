use alloc::alloc::{alloc, dealloc};
use core::{alloc::Layout, mem::size_of, ptr::write_bytes};

use crate::scheduler::{exit, tid::Tid, STACK_SIZE};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ProcessStatus {
    Invalid,
    Ready,
    Running,
    Blocked,
    Sleeping,
    Finished,
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
    pub tid: Tid,
    /// Status of a task, e.g. if the task is ready or blocked
    pub status: ProcessStatus,
    pub sleep_ticks: usize,
    /// Last stack pointer before a context switch to another task
    pub last_stack_pointer: usize,
    /// Stack of the task
    pub stack: *mut Stack,
    /// The amount of timer ticks that this task has been
    /// executed on the cpu.
    pub ticks: u64,
}

impl Task {
    /// Creates a new task that has status=ready with the BOOT_STACK, but
    /// without a stack pointer.
    pub fn new_idle(id: Tid) -> Task {
        Task {
            tid: id,
            sleep_ticks: 0,
            status: ProcessStatus::Ready,
            last_stack_pointer: 0,
            stack: unsafe { &mut BOOT_STACK },
            ticks: 0,
        }
    }

    /// Creates a Task that represents the currently running code, aka. the kernel.
    pub fn new_for_current(id: Tid) -> Task {
        let mut task = Self::new(id, ProcessStatus::Running); // current task is (of course) running
        unsafe {
            // copied and adapted from [`Task::allocate_stack`]

            let mut stack: *mut u64 = ((*task.stack).top()) as *mut u64;

            write_bytes((*task.stack).bottom() as *mut u8, 0xCD, STACK_SIZE);

            *stack = 0xCAFEBABEu64;
            stack = (stack as usize - size_of::<u64>()) as *mut u64;

            /* the first-function-to-be-called's arguments, ... */
            //TODO: add arguments

            *stack = (leave_task as *const ()) as u64;
            stack = (stack as usize - size_of::<State>()) as *mut u64;

            /*
            We don't need a State here.
            If we switch from this task to another, everything is pushed.
            We don't want to jump into this before we jump out of it
            (since this is considered the currently running task), so
            we will push a full State before we pop one (right?).
            */

            task.last_stack_pointer = stack as usize;
        }
        task
    }

    /// Creates a new task with the given status. Allocate stack for it with [`Task::allocate_stack`].
    pub fn new(id: Tid, status: ProcessStatus) -> Task {
        let layout = Layout::new::<Stack>();
        let stack = unsafe { alloc(layout) as *mut Stack };
        if stack as usize == 0 {
            panic!(
                "unable to allocate another kernel stack of size {} (allocation failure)",
                layout.size()
            );
        }

        Task {
            tid: id,
            status,
            sleep_ticks: 0,
            last_stack_pointer: 0,
            stack,
            ticks: 0,
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
    exit()
}

impl Task {
    /// Allocates stack memory for this task. The given entry_point is the code that
    /// this task executes.
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
