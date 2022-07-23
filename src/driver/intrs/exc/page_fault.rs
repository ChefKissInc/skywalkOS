//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use core::arch::asm;

pub(crate) unsafe extern "sysv64" fn handler(regs: &mut crate::sys::RegisterState) {
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
        if (regs.err_code & (1 << 3)) != 0 {
            "\nThe page was reserved"
        } else {
            ""
        },
        if (regs.err_code & (1 << 4)) != 0 {
            "\nAnd failed while doing an instruction fetch"
        } else {
            ""
        },
        if (regs.err_code & (1 << 5)) != 0 {
            "\nThe protection key was violated"
        } else {
            ""
        },
        if (regs.err_code & (1 << 15)) != 0 {
            "\nSGX was violated"
        } else {
            ""
        },
    );
}
