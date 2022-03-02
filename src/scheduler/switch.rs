use core::arch::asm;

macro_rules! push_context {
    () => {
        concat!(
            r#"
			pushfq
			push rax
			push rcx
			push rdx
			push rbx
			sub  rsp, 8
			push rbp
			push rsi
			push rdi
			push r8
			push r9
			push r10
			push r11
			push r12
			push r13
			push r14
			push r15
			"#,
        )
    };
}

macro_rules! pop_context {
    () => {
        concat!(
            r#"
			pop r15
			pop r14
			pop r13
			pop r12
			pop r11
			pop r10
			pop r9
			pop r8
			pop rdi
			pop rsi
			add rsp, 8
			pop rbp
			pop rbx
			pop rdx
			pop rcx
			pop rax
			popfq
			ret
			"#
        )
    };
}

#[naked]
pub unsafe extern "C" fn switch(_old_stack: *mut usize, _new_stack: usize) {
    // _old_stack is located in $rdi, _new_stack is in $rsi

    asm!(
        push_context!(),
        "mov [rdi], rsp",
        "mov rsp, rsi",
        pop_context!(),
        options(noreturn)
    );
}
