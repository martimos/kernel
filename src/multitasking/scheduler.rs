use alloc::boxed::Box;

use x86_64::instructions::interrupts;

use crate::multitasking::thread::TCB;

pub struct Scheduler {
    current_active: Option<Box<dyn TCB>>,

    kernel_tcb: Option<Box<dyn TCB>>,
    idle_tcb: Option<Box<dyn TCB>>,
}

impl Scheduler {
    pub const fn new() -> Self {
        Scheduler {
            current_active: None,

            kernel_tcb: None,
            idle_tcb: None,
        }
    }

    pub fn set_idle_tcb(&mut self, tcb: Box<dyn TCB>) {
        self.idle_tcb = Some(tcb);
    }

    pub fn set_kernel_tcb(&mut self, tcb: Box<dyn TCB>) {
        self.kernel_tcb = Some(tcb);
    }

    pub fn kernel_tcb(&self) -> &Box<dyn TCB> {
        self.kernel_tcb.as_ref().unwrap()
    }

    pub fn kernel_tcb_mut(&mut self) -> &mut Box<dyn TCB> {
        self.kernel_tcb.as_mut().unwrap()
    }

    unsafe fn switch_to(&mut self, tcb: &mut Box<dyn TCB>) {
        interrupts::disable();
        asm!(
        // 6 callee saved registers
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "mov rax, cr2",
        "push rax",               // push cr2
        "pushfq",                 // push RFLAGS
        "mov {current_rsp}, rsp", // save the current rsp
        "mov rsp, {new_rsp}",     // set the new rsp
        "cli",                    // clear interrupt flag
        "popfq",                  // pop RFLAGS
        "pop rax",
        "mov cr2, rax", // pop cr2
        // 6 callee saved registers
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        current_rsp = out(reg) self.current_active.as_mut().unwrap().info_mut().stack_pointer,
        new_rsp = in(reg) tcb.info_mut().stack_pointer,
        );
        interrupts::enable();
        asm!(
            // load RIP
            "ret",
        );
    }
}
