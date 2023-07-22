// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use crate::system::RegisterState;

pub unsafe extern "sysv64" fn invalid_tss_handler(regs: &mut RegisterState) {
    super::exc_msg!(
        "invalid TSS",
        format!("Segment selector: {:#X?}", regs.err_code),
        regs
    );
}

pub unsafe extern "sysv64" fn segment_not_present_handler(regs: &mut RegisterState) {
    super::exc_msg!(
        "segment not present",
        format!("Segment selector: {:#X?}", regs.err_code),
        regs
    );
}

pub unsafe extern "sysv64" fn stack_exc_handler(regs: &mut RegisterState) {
    super::exc_msg!(
        "stack exception",
        format!("Segment selector: {:#X?}", regs.err_code),
        regs
    );
}

pub unsafe extern "sysv64" fn general_prot_fault_handler(regs: &mut RegisterState) {
    super::exc_msg!(
        "general protection fault",
        format!("Segment selector: {:#X?}", regs.err_code),
        regs
    );
}
