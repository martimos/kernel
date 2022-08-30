use core::ptr::NonNull;
use core::time::Duration;

use kstd::sync::Once;

use crate::{scheduler::tid::Tid, Result};

pub mod round_robin;
pub mod switch;
pub mod task;
pub mod tid;

pub const STACK_SIZE: usize = 0x8000; // 32 KiB

static mut SCHEDULER: Option<round_robin::RoundRobin> = None;
static SCHEDULER_INIT: Once = Once::new();

/// Initialise module, must be called once, and only once
pub fn init() {
    SCHEDULER_INIT.call_once(|| unsafe {
        SCHEDULER = Some(round_robin::RoundRobin::new());
    });
}

pub struct Scheduler;

impl Scheduler {
    /// Create a new kernel task
    pub fn spawn_from_c_fn(func: extern "C" fn()) -> Result<Tid> {
        unsafe {
            Self::spawn_from_entry_point(NonNull::new(func as *const () as *mut usize).unwrap())
        }
    }

    /// Create a new kernel task with the given pointer as entry point.
    ///
    /// # Safety:
    /// The caller must ensure that the given entry is a valid
    /// pointer to executable code.
    pub unsafe fn spawn_from_entry_point(entry: NonNull<usize>) -> Result<Tid> {
        unsafe { SCHEDULER.as_mut().unwrap().spawn(entry) }
    }

    /// Puts the current task to sleep for **at least** the given duration.
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

    /// Returns the amount of cpu time that the current task has been run.
    /// This is a function of [`Scheduler::total_ticks`].
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

    /// Returns the amount of total ticks that the current task has been running.
    pub fn total_ticks() -> u64 {
        unsafe { SCHEDULER.as_ref().unwrap().total_ticks() }
    }
}
