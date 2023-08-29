// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

pub unsafe extern "sysv64" fn page_fault(regs: &mut crate::system::RegisterState) {
    let mut cr2: u64;
    core::arch::asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));

    let msg = format!(
        "There was a {} while {} a {} page at {cr2:#X?}.{}{}{}{}",
        if (regs.err_code & (1 << 0)) == 0 {
            "non-present page access"
        } else {
            "page-level protection violation"
        },
        if (regs.err_code & (1 << 1)) == 0 {
            "reading"
        } else {
            "writing"
        },
        if (regs.err_code & (1 << 2)) == 0 {
            "supervisor"
        } else {
            "user"
        },
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
            " Protection key violation."
        },
        if (regs.err_code & (1 << 15)) == 0 {
            ""
        } else {
            " SGX violation."
        },
    );

    super::exception_msg!("page fault", msg, regs);
}
