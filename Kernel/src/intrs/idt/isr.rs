// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

macro_rules! isr_stub {
    ($err:expr, $i:expr) => {
        core::arch::asm!(
            $err,
            "cld",
            "push {}",
            "push rax",
            "push rbx",
            "push rcx",
            "push rdx",
            "push rsi",
            "push rdi",
            "push rbp",
            "push r8",
            "push r9",
            "push r10",
            "push r11",
            "push r12",
            "push r13",
            "push r14",
            "push r15",
            "mov rdi, rsp",
            "call {}",
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rbp",
            "pop rdi",
            "pop rsi",
            "pop rdx",
            "pop rcx",
            "pop rbx",
            "pop rax",
            "add rsp, 16",
            "iretq",
            const $i,
            sym isr_handler,
            options(noreturn),
        )
    };
}

unsafe extern "C" fn isr_handler(regs: &mut crate::system::RegisterState) {
    let n = regs.int_num as u8;
    let handler = &super::HANDLERS.get().as_mut().unwrap()[n as usize];
    (handler.func)(regs);
    if handler.is_irq {
        let state = crate::system::state::SYS_STATE.get().as_mut().unwrap();
        state.lapic.get_mut().unwrap().send_eoi();
    }
    if !handler.should_iret && !handler.is_irq {
        crate::hlt_loop!();
    }
}

macro_rules! isr_noerr {
    ($func_name:ident, $i:tt) => {
        #[naked]
        pub(super) unsafe extern "C" fn $func_name() {
            isr_stub!("push 0", $i)
        }
    };
}

macro_rules! isr_err {
    ($func_name:ident, $i:tt) => {
        #[naked]
        pub(super) unsafe extern "C" fn $func_name() {
            isr_stub!("", $i)
        }
    };
}

seq_macro::seq!(N in 0..8 {
    isr_noerr!(isr~N, N);
});
isr_err!(isr8, 8);
isr_noerr!(isr9, 9);
seq_macro::seq!(N in 10..16 {
    isr_err!(isr~N, N);
});
seq_macro::seq!(N in 16..256 {
    isr_noerr!(isr~N, N);
});
