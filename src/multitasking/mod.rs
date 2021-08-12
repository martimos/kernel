use spin::Mutex;
use x86_64::VirtAddr;

use scheduler::Scheduler;

use crate::hlt_loop;
use crate::multitasking::thread::{State, ThreadControlBlock};

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
    /*
    create TCB for the kernel code that is already running,
    meaning getting registers like rbp, rsp etc and
    putting them into a new ThreadControlBlock
     */
    let mut stack_pointer: u64;
    let mut base_pointer: u64;
    asm!(
    "mov {0}, rsp",
    "mov {1}, rbp",
    out(reg) stack_pointer,
    out(reg) base_pointer,
    );
    let stack_addr = VirtAddr::try_new(stack_pointer).expect("stack_pointer invalid");
    let mem_addr = VirtAddr::try_new(base_pointer).expect("base_pointer invalid");
    let mut tcb = ThreadControlBlock::new(process::Id::from(0), "kernel_task", stack_addr, mem_addr);
    tcb.set_state(State::Running);

    SCHEDULER.lock().set_kernel_tcb(tcb);
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