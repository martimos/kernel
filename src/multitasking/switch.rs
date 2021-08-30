#[naked] // required, won't work in release builds otherwise
pub unsafe extern "C" fn switch(_old_stack: *mut usize, _new_stack: usize) -> ! {
    // rdi = old_stack => the address to store the old rsp
    // rsi = new_stack => stack pointer of the new task
    asm!(
        // save registers
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        // save RFLAGS
        "mov rax, cr2",
        "push rax",
        "pushfq",
        // switch stack pointer
        "mov [rdi], rsp",
        "mov rsp, rsi",
        // set "switched"-flag
        "mov rax, cr0",
        "or rax, 8",
        "mov cr0, rax",
        // restore old state of new thread
        "cli",
        "popfq",
        "pop rax",
        "mov cr2, rax",
        // restore registers
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        // load new instruction pointer
        "ret",
        options(noreturn),
    )
}
