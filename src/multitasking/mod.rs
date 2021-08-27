use alloc::boxed::Box;

use lazy_static::lazy_static;
use spin::Mutex;

use scheduler::Scheduler;

use crate::syscall::error::Errno;

pub mod process;
pub mod scheduler;
pub mod thread;

mod switch;

pub type Work = dyn 'static + FnOnce() + Send + Sync;

lazy_static! {
    static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

pub fn init() {}

pub fn spawn_thread(work: Box<Work>, prio: thread::Priority) -> Result<thread::ThreadId, Errno> {
    Ok(SCHEDULER.lock().spawn_thread(work, prio).lock().id)
}

pub fn schedule() -> ! {
    SCHEDULER.lock().schedule()
    // FIXME: this is bad, mutex is not released
}
