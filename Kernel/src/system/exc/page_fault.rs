// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

pub unsafe extern "C" fn handler(regs: &mut crate::system::RegisterState) {
    let mut cr2: u64;
    core::arch::asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags, pure));

    let msg = format!(
        "There was {} while {} a {} at {:#X?}.{}{}{}{}",
        if (regs.err_code & (1 << 0)) == 0 {
            "a non-present page access"
        } else {
            "a page-level protection violation"
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
            " Page was reserved."
        },
        if (regs.err_code & (1 << 4)) == 0 {
            ""
        } else {
            " Failed during an instruction fetch."
        },
        if (regs.err_code & (1 << 5)) == 0 {
            ""
        } else {
            " Protection key violatation."
        },
        if (regs.err_code & (1 << 15)) == 0 {
            ""
        } else {
            " SGX violation."
        },
    );

    super::exc_msg!("page fault", msg, regs);
}
