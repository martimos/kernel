use spin::Once;

use crate::{scheduler::tid::Tid, Result};

pub mod round_robin;
pub mod switch;
pub mod task;
pub mod tid;

pub const NUM_PRIORITIES: usize = 32;
pub const STACK_SIZE: usize = 0x2000;

static mut SCHEDULER: Option<round_robin::Scheduler> = None;
static SCHEDULER_INIT: Once = Once::new();

/// Initialise module, must be called once, and only once
pub fn init() {
    SCHEDULER_INIT.call_once(|| unsafe {
        SCHEDULER = Some(round_robin::Scheduler::new());
    });
}

#[cfg(debug_assertions)]
pub fn disable_idle_task() {
    unsafe { SCHEDULER.as_mut().unwrap().disable_idle_task() }
}

/// Create a new kernel task
#[must_use = "spawning a task may fail"]
pub fn spawn(func: extern "C" fn()) -> Result<Tid> {
    unsafe { SCHEDULER.as_mut().unwrap().spawn(func) }
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

/// Returns once the task with the given [`Tid`] is done.
/// If the [`Tid`] does not exist, this returns immediately.
pub fn join(tid: Tid) {
    unsafe {
        SCHEDULER.as_mut().unwrap().join(tid);
    }
}

/// Terminate the current running task
pub fn exit() -> ! {
    unsafe {
        SCHEDULER.as_mut().unwrap().exit();
    }
}

/// Get the TID of the current running task
pub fn get_current_tid() -> Tid {
    unsafe { SCHEDULER.as_ref().unwrap().get_current_tid() }
}
