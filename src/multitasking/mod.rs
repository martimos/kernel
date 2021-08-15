use spin::Mutex;

use scheduler::Scheduler;

use crate::multitasking::thread::{BootstrapTCB, State, TCB};

pub mod process;
pub mod scheduler;
pub mod thread;

pub static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());

pub fn init() {
    unsafe {
        init_kernel_task();
        init_idle_task();
    }
}

unsafe fn init_kernel_task() {
    let mut tcb = BootstrapTCB::new();
    tcb.info_mut().set_state(State::Running);

    crate::serial_println!("kernel tcb: {:?}", tcb);

    SCHEDULER.lock().set_kernel_tcb(box tcb);
}

unsafe fn init_idle_task() {
    // let mut tcb = ThreadControlBlock::new(process::Id::from(0), "idle_task", VirtAddr::new(0), VirtAddr::new(0));
    // tcb.set_priority(Priority::Lowest);
    // tcb.set_state(State::Ready);
    //
    // let idle_instruction_pointer: u64 = __idle_task_impl as *const u8 as u64;
    //
    // SCHEDULER.lock().set_idle_tcb(tcb);
}
