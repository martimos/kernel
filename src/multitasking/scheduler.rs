use alloc::boxed::Box;
use alloc::collections::{BTreeMap, VecDeque};
use alloc::sync::Arc;
use core::mem::{size_of, swap};

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts;

use crate::hlt_loop;
use crate::kresult::KResult;
use crate::multitasking::switch::switch;
use crate::multitasking::thread::{Priority, State, Thread, ThreadId};
use crate::multitasking::{Work, SCHEDULER};
use crate::serial_println;

pub struct Scheduler {
    current_thread: Arc<Mutex<Thread>>,
    ready_queue: Mutex<VecDeque<Arc<Mutex<Thread>>>>,
    finish_queue: Mutex<VecDeque<Arc<Mutex<Thread>>>>,
}

lazy_static! {
    static ref IDLE_THREAD: Arc<Mutex<Thread>> = {
        let thread = Scheduler::prepare_thread(box move || hlt_loop(), Priority::Low);
        Arc::new(Mutex::new(thread))
    };
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            current_thread: {
                let thread = IDLE_THREAD.clone();
                {
                    let mut thread_guard = thread.lock();
                    thread_guard.state = State::Running;
                }
                thread
            },
            ready_queue: Mutex::new(VecDeque::new()),
            finish_queue: Mutex::new(VecDeque::new()),
        }
    }

    pub(crate) fn spawn_thread(&mut self, work: Work, prio: Priority) -> KResult<ThreadId> {
        let thread = Scheduler::prepare_thread(work, prio);
        let thread_id = thread.id;
        let thread_mutex = Mutex::new(thread);
        let thread_arc = Arc::new(thread_mutex);

        let irq_disabled = interrupts::are_enabled();
        interrupts::disable();
        // without interrupts
        self.ready_queue.lock().push_back(thread_arc.clone());

        if irq_disabled {
            interrupts::enable()
        }

        Ok(thread_id)
    }

    fn prepare_thread(work: Work, prio: Priority) -> Thread {
        const U64_WIDTH: usize = size_of::<u64>();
        const REGISTERS_WIDTH: usize = size_of::<ThreadRegisters>();

        let mut thread = Thread::new(work, prio);
        let stack = thread.stack_mut();
        let mut index = stack.len();

        // write last RIP at end of stack
        index -= U64_WIDTH;
        stack.write_at(index, &(thread_die as *const () as u64).to_ne_bytes());

        // write registers
        index -= REGISTERS_WIDTH;
        stack.write_at(index, [0_u8; REGISTERS_WIDTH].as_slice());
        unsafe {
            let registers: *mut ThreadRegisters = (stack.bottom() + index) as *mut ThreadRegisters;
            // (*registers).rip = work as *const () as u64;
            (*registers).rip = thread_start as *const () as u64;
            (*registers).rbx = 0xDEADBEEFDEADBEEF;
            (*registers).rflags = 0x1202u64;
        }

        let stack_bottom = stack.bottom();
        thread.set_stack_pointer(stack_bottom + index);
        thread
    }

    fn take_current_work(&mut self) -> Work {
        let mut work = None;
        unsafe {
            let mut guard = SCHEDULER.as_mut().unwrap().current_thread.lock();
            swap(&mut guard.work, &mut work);
        }

        work.expect("no current work in scheduler")
    }

    pub fn schedule(&mut self) -> ! {
        let mut next_thread = match self.ready_queue.lock().pop_front() {
            None => panic!("no next thread"),
            Some(task) => task,
        };

        let next_stack_pointer;
        {
            let mut guard = next_thread.lock();
            guard.set_state(State::Running);
            next_stack_pointer = guard.stack_pointer;
        }

        swap(&mut self.current_thread, &mut next_thread);
        let old_thread = next_thread; // rename to avoid confusion in the code
        let mut old_thread_guard = old_thread.lock();
        let old_thread_id = old_thread_guard.id;
        let old_state = old_thread_guard.state;
        let old_stack_pointer = &mut old_thread_guard.stack_pointer as *mut usize;

        match old_state {
            State::Running => {
                old_thread_guard.set_state(State::Ready);
                self.ready_queue.lock().push_back(old_thread.clone());
            }
            State::Finished => {
                // do not deallocate - stack is reqiured for context switch
                self.finish_queue.lock().push_back(old_thread.clone());
            }
            _ => unreachable!(
                "state of old thread with id {:?} was not running or finished, but {:?}",
                old_thread_id, old_state
            ),
        }
        drop(old_thread_guard); // release lock

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

unsafe extern "C" fn thread_start() {
    serial_println!("thread_start");
    let work = crate::multitasking::SCHEDULER
        .as_mut()
        .unwrap()
        .take_current_work();
    work();
}

extern "C" fn thread_die() -> ! {
    serial_println!("thread_die");
    hlt_loop();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_spawn_creates_thread() {
        let mut sched = Scheduler::new();
        assert_eq!(0, sched.ready_queue.lock().len());
        let result = sched.spawn_thread(box move || {}, Priority::High);
        assert!(result.is_ok());
        let thread_id = result.unwrap();
        assert_eq!(1, sched.ready_queue.lock().len());

        let thread = sched
            .ready_queue
            .lock()
            .pop_front()
            .expect("no thread in queue");
        assert_eq!(thread_id, thread.lock().id);
        assert_eq!(Priority::High, thread.lock().priority);
        assert_eq!(State::Ready, thread.lock().state);
    }
}
