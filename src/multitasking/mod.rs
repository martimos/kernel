use alloc::boxed::Box;

use lazy_static::lazy_static;
use spin::Mutex;

use scheduler::Scheduler;

use crate::hlt_loop;
use crate::multitasking::thread::Priority;
use crate::syscall::error::Errno;

pub mod process;
pub mod scheduler;
pub mod thread;

mod switch;

pub type Task = dyn 'static + FnOnce() + Send + Sync;

lazy_static! {
    static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

pub fn init() {
    let idle_thread = SCHEDULER.lock().spawn_thread(
        box move || {
            hlt_loop();
        },
        Priority::Low,
    );
    SCHEDULER.lock().set_current(idle_thread);
}

pub fn spawn_thread(task: Box<Task>, prio: thread::Priority) -> Result<thread::ThreadId, Errno> {
    Ok(SCHEDULER.lock().spawn_thread(task, prio).lock().id)
}

pub fn schedule() -> ! {
    SCHEDULER.lock().schedule()
}
