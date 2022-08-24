use core::time::Duration;

use kstd::sync::Once;

use crate::{scheduler::tid::Tid, Result};

pub mod round_robin;
pub mod switch;
pub mod task;
pub mod tid;

pub const NUM_PRIORITIES: usize = 32;
pub const STACK_SIZE: usize = 0x8000;

static mut SCHEDULER: Option<round_robin::Scheduler> = None;
static SCHEDULER_INIT: Once = Once::new();

/// Initialise module, must be called once, and only once
pub fn init() {
    SCHEDULER_INIT.call_once(|| unsafe {
        SCHEDULER = Some(round_robin::Scheduler::new());
    });
}

pub struct Scheduler;

impl Scheduler {
    /// Create a new kernel task
    #[must_use = "spawning a task may fail"]
    pub fn spawn(func: extern "C" fn()) -> Result<Tid> {
        unsafe { SCHEDULER.as_mut().unwrap().spawn(func) }
    }

    pub fn sleep(duration: Duration) {
        unsafe { SCHEDULER.as_mut().unwrap().sleep(duration) }
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

    pub fn cpu_time() -> Duration {
        unsafe { SCHEDULER.as_mut().unwrap().cpu_time() }
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

    /// Signal to the scheduler that a timer tick occurred.
    /// The tick is ignored if the scheduler is not initialized yet.
    pub fn timer_tick() {
        unsafe {
            if let Some(sched) = SCHEDULER.as_mut() {
                sched.timer_tick()
            }
        }
    }

    pub fn total_ticks() -> u64 {
        unsafe { SCHEDULER.as_ref().unwrap().total_ticks() }
    }
}
