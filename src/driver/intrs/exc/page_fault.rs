// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use core::arch::asm;

pub unsafe extern "sysv64" fn handler(regs: &mut crate::sys::RegisterState) {
    super::exc_msg!("page fault", regs);

    let mut cr2: u64;
    asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));

    error!(
        "There was {} while {} a {} at {:#X?}{}{}{}{}",
        if (regs.err_code & (1 << 0)) == 0 {
            "a Non-present page access"
        } else {
            "a Page Level protection violation"
        },
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
        cr2,
        if (regs.err_code & (1 << 3)) == 0 {
            ""
        } else {
            "\nThe page was reserved"
        },
        if (regs.err_code & (1 << 4)) == 0 {
            ""
        } else {
            "\nAnd failed while doing an instruction fetch"
        },
        if (regs.err_code & (1 << 5)) == 0 {
            ""
        } else {
            "\nThe protection key was violated"
        },
        if (regs.err_code & (1 << 15)) == 0 {
            ""
        } else {
            "\nSGX was violated"
        },
    );
}
