use x86_64::instructions::interrupts::without_interrupts;

use crate::multitasking::thread::ThreadControlBlock;

pub struct Scheduler {
    kernel_tcb: Option<ThreadControlBlock>,
    idle_tcb: Option<ThreadControlBlock>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            kernel_tcb: None,
            idle_tcb: None,
        }
    }

    pub fn set_idle_tcb(&mut self, tcb: ThreadControlBlock) {
        self.idle_tcb = Some(tcb);
    }

    pub fn set_kernel_tcb(&mut self, tcb: ThreadControlBlock) {
        self.kernel_tcb = Some(tcb);
    }

    pub fn kernel_tcb(&self) -> &ThreadControlBlock {
        self.kernel_tcb.as_ref().unwrap()
    }

    pub fn kernel_tcb_mut(&mut self) -> &mut ThreadControlBlock {
        self.kernel_tcb.as_mut().unwrap()
    }

    unsafe fn switch_to(&mut self, tcb: ThreadControlBlock) {
        without_interrupts(|| {});
    }
}
