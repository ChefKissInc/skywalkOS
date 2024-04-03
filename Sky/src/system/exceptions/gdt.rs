// Copyright (c) ChefKiss Inc 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use crate::system::RegisterState;

pub unsafe extern "sysv64" fn invalid_tss(regs: &mut RegisterState) {
    super::exception_msg!(
        "invalid TSS",
        format!("Segment selector: {:#X?}", regs.err_code),
        regs
    );
}

pub unsafe extern "sysv64" fn segment_not_present(regs: &mut RegisterState) {
    super::exception_msg!(
        "segment not present",
        format!("Segment selector: {:#X?}", regs.err_code),
        regs
    );
}

pub unsafe extern "sysv64" fn stack_exception(regs: &mut RegisterState) {
    super::exception_msg!(
        "stack exception",
        format!("Segment selector: {:#X?}", regs.err_code),
        regs
    );
}

pub unsafe extern "sysv64" fn general_protection_fault(regs: &mut RegisterState) {
    super::exception_msg!(
        "general protection fault",
        format!("Segment selector: {:#X?}", regs.err_code),
        regs
    );
}
