// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::format;

use crate::sys::RegisterState;

pub unsafe extern "sysv64" fn invalid_tss_handler(regs: &mut RegisterState) {
    super::exc_msg!(
        "invalid TSS",
        format!("Segment selector: {}", regs.err_code),
        regs
    );
}

pub unsafe extern "sysv64" fn segment_not_present_handler(regs: &mut RegisterState) {
    super::exc_msg!(
        "segment not present",
        format!("Segment selector: {}", regs.err_code),
        regs
    );
}

pub unsafe extern "sysv64" fn stack_exc_handler(regs: &mut RegisterState) {
    super::exc_msg!(
        "stack exception",
        format!("Segment selector: {}", regs.err_code),
        regs
    );
}

pub unsafe extern "sysv64" fn general_prot_fault_handler(regs: &mut RegisterState) {
    super::exc_msg!(
        "general protection fault",
        format!("Segment selector: {}", regs.err_code),
        regs
    );
}
