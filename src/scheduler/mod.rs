use crate::scheduler::pid::Pid;
use crate::scheduler::priority::Priority;
use crate::scheduler::task::Task;
use crate::Result;
use alloc::rc::Rc;
use core::cell::RefCell;

pub mod pid;
pub mod priority;
pub mod queue;
pub mod scheduler;
pub mod switch;
pub mod task;

pub const NUM_PRIORITIES: usize = 32;
pub const STACK_SIZE: usize = 0x2000;

static mut SCHEDULER: Option<scheduler::Scheduler> = None;

/// Initialise module, must be called once, and only once
pub fn init() {
    unsafe {
        SCHEDULER = Some(scheduler::Scheduler::new());
    }
}

/// Create a new kernel task
pub fn spawn(func: extern "C" fn(), prio: Priority) -> Result<Pid> {
    unsafe { SCHEDULER.as_mut().unwrap().spawn(func, prio) }
}

/// Trigger the scheduler to switch to the next available task
pub fn reschedule() {
    unsafe {
        if SCHEDULER.is_none() {
            return;
        }
        SCHEDULER.as_mut().unwrap().reschedule()
    }
}

/// Timer interrupt  call scheduler to switch to the next available task
pub fn schedule() {
    unsafe {
        if SCHEDULER.is_none() {
            return;
        }
        SCHEDULER.as_mut().unwrap().reschedule()
    }
}

/// Terminate the current running task
pub fn do_exit() {
    unsafe {
        SCHEDULER.as_mut().unwrap().exit();
    }
}

/// Terminate the current running task
pub fn abort() -> ! {
    unsafe { SCHEDULER.as_mut().unwrap().abort() }
}

pub fn get_current_stack() -> usize {
    unsafe { SCHEDULER.as_mut().unwrap().get_current_stack_bottom() }
}

pub fn block_current_task() -> Rc<RefCell<Task>> {
    unsafe { SCHEDULER.as_mut().unwrap().block_current_task() }
}

pub fn wakeup_task(task: Rc<RefCell<Task>>) {
    unsafe { SCHEDULER.as_mut().unwrap().wakeup_task(task) }
}

/// Get the PID of the current running task
pub fn get_current_pid() -> Pid {
    unsafe { SCHEDULER.as_ref().unwrap().get_current_pid() }
}
