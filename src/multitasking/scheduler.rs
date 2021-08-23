use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use core::mem::{size_of, swap};

use spin::Mutex;
use x86_64::instructions::interrupts;

use crate::hlt_loop;
use crate::multitasking::switch::switch;
use crate::multitasking::thread::{Priority, State, Thread, ThreadId};
use crate::multitasking::Task;

pub struct Scheduler {
    current_thread: Option<Arc<Mutex<Thread>>>,
    ready_queue: Mutex<VecDeque<Arc<Mutex<Thread>>>>,
    finish_queue: Mutex<VecDeque<Arc<Mutex<Thread>>>>,

    threads: Mutex<BTreeMap<ThreadId, Arc<Mutex<Thread>>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            current_thread: None,
            ready_queue: Mutex::new(VecDeque::new()),
            finish_queue: Mutex::new(VecDeque::new()),
            threads: Mutex::new(BTreeMap::new()),
        }
    }

    /// Sets a new `current_thread` in this scheduler. Only call when you know what you're doing.
    /// This will discard the old `current_thread`, and there may be no way to recover it.
    pub(crate) fn set_current(&mut self, new_current: Arc<Mutex<Thread>>) {
        swap(&mut self.current_thread, &mut Some(new_current));
    }

    pub(crate) fn spawn_thread(&self, task: Box<Task>, prio: Priority) -> Arc<Mutex<Thread>> {
        let thread = Arc::new(Mutex::new(self.prepare_thread(task, prio)));
        let thread_id = thread.lock().id;

        let irq_disabled = interrupts::are_enabled();
        interrupts::disable();
        // without interrupts
        self.ready_queue.lock().push_back(thread.clone());
        self.threads.lock().insert(thread_id, thread.clone());

        if irq_disabled {
            interrupts::enable()
        }

        thread.clone()
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
            (*registers).rip = task.as_ref() as *const _ as *const () as u64;
            (*registers).rflags = 0x1202u64;
        }

        let stack_top = stack.top();
        thread.set_stack_pointer(stack_top - index);
        thread
    }

    pub fn schedule(&mut self) -> ! {
        let mut next_thread = Some(match self.ready_queue.lock().pop_front() {
            None => panic!("no next thread"),
            Some(task) => task,
        });
        let next_stack_pointer;
        {
            let mut guard = next_thread.as_mut().unwrap().lock();
            guard.set_state(State::Running);
            next_stack_pointer = guard.stack_pointer;
        }

        swap(&mut self.current_thread, &mut next_thread);
        let old_thread = next_thread.unwrap();
        let old_state = old_thread.lock().state;
        let old_stack_pointer = &mut old_thread.lock().stack_pointer as *mut usize;

        match old_state {
            State::Running => {
                old_thread.lock().set_state(State::Ready);
                self.ready_queue.lock().push_back(old_thread.clone());
            }
            State::Finished => {
                // do not deallocate - stack is reqiured for context switch
                self.finish_queue.lock().push_back(old_thread.clone());
            }
            _ => unreachable!("old state was not running or finished"),
        }

        unsafe { switch(old_stack_pointer, next_stack_pointer) }
    }
}

#[repr(C, packed)]
struct ThreadRegisters {
    // gs: u64,
    // fs: u64,
    // r11: u64,
    // r10: u64,
    // r9: u64,
    // r8: u64,
    // rdi: u64,
    // rsi: u64,
    // rdx: u64,
    // rcx: u64,
    // rax: u64,
    rflags: u64,
    cr2: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbp: u64,
    rbx: u64,
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
        let result = sched.spawn_thread(box move || {}, Priority::High);
        let thread_id = result.lock().id;
        assert_eq!(1, sched.ready_queue.lock().len());
        assert_eq!(1, sched.threads.lock().len());

        let thread = sched
            .ready_queue
            .lock()
            .pop_front()
            .expect("no thread in queue");
        assert_eq!(thread_id, thread.lock().id);
        assert_eq!(Priority::High, thread.lock().priority);
        assert_eq!(State::Ready, thread.lock().state);
    }

    #[test_case]
    fn test_ugly_box_deref_works() {
        let foo: Box<Task> = Box::new(move || {});
        let addr_value = foo.as_ref() as *const _ as *const () as u64;
        let box_raw_addr = Box::into_raw(foo) as *const () as u64;
        assert_eq!(addr_value, box_raw_addr);
    }
}
