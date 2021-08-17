use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::rc::Rc;
use alloc::sync::Arc;
use core::mem::size_of;

use spin::Mutex;
use x86_64::instructions::interrupts;
use x86_64::instructions::interrupts::without_interrupts;

use crate::hlt_loop;
use crate::multitasking::thread::{Priority, Thread, ThreadId};
use crate::multitasking::Task;
use crate::syscall::error::Errno;

pub struct Scheduler {
    ready_queue: Mutex<VecDeque<Arc<Thread>>>,

    threads: Mutex<BTreeMap<ThreadId, Arc<Thread>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            ready_queue: Mutex::new(VecDeque::new()),
            threads: Mutex::new(BTreeMap::new()),
        }
    }

    pub(crate) fn spawn_thread(&self, task: Box<Task>, prio: Priority) -> Result<ThreadId, Errno> {
        let thread = Arc::new(self.prepare_thread(task, prio));

        let irq_disabled = interrupts::are_enabled();
        interrupts::disable();
        // without interrupts
        self.ready_queue.lock().push_back(thread.clone());
        self.threads.lock().insert(thread.id(), thread.clone());

        if irq_disabled {
            interrupts::enable()
        }

        Ok(thread.id())
    }

    fn prepare_thread(&self, task: Box<Task>, prio: Priority) -> Thread {
        const U64_WIDTH: usize = size_of::<u64>();
        const REGISTERS_WIDTH: usize = size_of::<ThreadRegisters>();

        let mut thread = Thread::new(prio);
        let stack = thread.stack_mut();
        let mut index = stack.len();

        // write last RIP at end of stack
        index -= U64_WIDTH;
        stack.write_at(index, &(thread_die as *const () as u64).to_ne_bytes());

        // write registers
        index -= REGISTERS_WIDTH;
        stack.write_at(index, [0_u8; REGISTERS_WIDTH].as_slice());
        unsafe {
            let registers: *mut ThreadRegisters = (stack.top() - index) as *mut ThreadRegisters;
            (*registers).rsp = (index + REGISTERS_WIDTH) as u64;
            (*registers).rbp = ((*registers).rsp + U64_WIDTH as u64) as u64;
            (*registers).rip = task.as_ref() as *const _ as *const () as u64;
            (*registers).rflags = 0x1202u64;
        }

        let stack_top = stack.top();
        thread.set_stack_pointer(stack_top - index);
        thread
    }
}

#[repr(C, packed)]
struct ThreadRegisters {
    gs: u64,
    fs: u64,
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

extern "C" fn thread_die() -> ! {
    hlt_loop();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_spawn_creates_thread() {
        let sched = Scheduler::new();
        assert_eq!(0, sched.ready_queue.lock().len());
        assert_eq!(0, sched.threads.lock().len());
        sched.spawn_thread(box move || {}, Priority::Normal);
        assert_eq!(1, sched.ready_queue.lock().len());
        assert_eq!(1, sched.threads.lock().len());
    }

    #[test_case]
    fn test_ugly_box_deref_works() {
        let foo: Box<Task> = Box::new(move || {});
        let addr_value = foo.as_ref() as *const _ as *const () as u64;
        let box_raw_addr = Box::into_raw(foo) as *const () as u64;
        assert_eq!(addr_value, box_raw_addr);
    }
}
