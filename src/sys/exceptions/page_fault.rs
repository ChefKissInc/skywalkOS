/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

use core::arch::asm;

pub(crate) unsafe extern "sysv64" fn handler(regs: &mut amd64::sys::cpu::RegisterState) {
    super::exc_msg!("page fault", regs);

    let mut cr2: u64;
    asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));

    error!(
        "There was {}",
        if (regs.err_code & (1 << 0)) == 0 {
            "a Non-present page access"
        } else {
            "a Page Level protection violation"
        }
    );
    error!(
        "while {} a {} at {:#X?}",
        if (regs.err_code & (1 << 1)) == 0 {
            "reading"
        } else {
            "writing"
        },
        if (regs.err_code & (1 << 2)) == 0 {
            "supervisor page"
        } else {
            "user page"
        },
        cr2
    );
    if (regs.err_code & (1 << 3)) != 0 {
        error!("The page was reserved");
    }
    if (regs.err_code & (1 << 4)) != 0 {
        error!("And failed while doing an instruction fetch");
    }
    if (regs.err_code & (1 << 5)) != 0 {
        error!("The protection key was violated");
    }
    if (regs.err_code & (1 << 15)) != 0 {
        error!("SGX was violated");
    }
}
