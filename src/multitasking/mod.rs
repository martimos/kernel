use alloc::boxed::Box;

use lazy_static::lazy_static;
use spin::Mutex;

use scheduler::Scheduler;

use crate::syscall::error::Errno;

pub mod process;
pub mod scheduler;
pub mod thread;

mod switch;

pub type Task = dyn 'static + FnOnce() + Send + Sync;

lazy_static! {
    static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

pub fn init() {}

pub fn spawn_thread(task: Box<Task>, prio: thread::Priority) -> Result<thread::ThreadId, Errno> {
    Ok(SCHEDULER.lock().spawn_thread(task, prio).lock().id)
}

pub fn schedule() -> ! {
    SCHEDULER.lock().schedule()
    // FIXME: this is bad, mutex is not released
}
