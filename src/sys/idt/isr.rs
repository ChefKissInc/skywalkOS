use log::info;

macro_rules! isr_stub {
    ($err:expr, $i:expr) => {
        asm!(
            "cli",
            $err,
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
            "xor ax, ax",
            "mov es, ax",
            "mov ds, ax",
            "cld",
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
            options(noreturn)
        )
    };
}

pub unsafe extern "C" fn isr_handler(regs: &mut amd64::sys::cpu::RegisterState) {
    let n = (regs.int_num & 0xFF) as u8;
    info!("Got interrupt {}\n{:#X?}", n, regs);
    let handler = (*super::HANDLERS.get()).get(n as usize).unwrap();
    (handler.func)(regs);
    if !handler.should_iret && !handler.is_irq {
        loop {
            asm!("hlt")
        }
    }
}

macro_rules! isr_noerr {
    ($func_name:ident, $i:tt) => {
        #[naked]
        pub unsafe extern "C" fn $func_name() -> ! {
            isr_stub!("push 0", $i)
        }
    };
}

macro_rules! isr_err {
    ($func_name:ident, $i:tt) => {
        #[naked]
        pub unsafe extern "C" fn $func_name() -> ! {
            isr_stub!("", $i)
        }
    };
}

seq_macro::seq!(N in 0..8 {
    isr_noerr!(isr #N, N);
});
isr_err!(isr8, 8);
isr_noerr!(isr9, 9);
seq_macro::seq!(N in 10..16 {
    isr_err!(isr #N, N);
});
seq_macro::seq!(N in 16..255 {
    isr_noerr!(isr #N, N);
});

#[naked]
pub unsafe extern "C" fn isr255() {
    asm!("iretq", options(noreturn))
}
