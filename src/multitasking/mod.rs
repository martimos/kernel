use alloc::boxed::Box;

use lazy_static::lazy_static;
use spin::Mutex;

use scheduler::Scheduler;

use crate::syscall::error::Errno;

pub mod process;
pub mod scheduler;
pub mod thread;

mod switch;

pub type Work = Box<dyn FnOnce() + Send + Sync>;

static mut SCHEDULER: Option<Scheduler> = None;

pub fn init() {
    unsafe {
        SCHEDULER = Some(Scheduler::new());
    }
}

pub fn spawn_thread(work: Work, prio: thread::Priority) -> Result<thread::ThreadId, Errno> {
    unsafe { SCHEDULER.as_mut().unwrap().spawn_thread(work, prio) }
}

pub fn schedule() {
    unsafe { SCHEDULER.as_mut().unwrap().schedule() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn test_atomic_counter() {}
}
