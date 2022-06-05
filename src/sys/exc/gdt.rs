//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use amd64::cpu::RegisterState;

pub(crate) unsafe extern "sysv64" fn invalid_tss_handler(regs: &mut RegisterState) {
    super::exc_msg!("invalid TSS", regs);
    error!("Segment selector: {}", regs.err_code);
}

pub(crate) unsafe extern "sysv64" fn segment_not_present_handler(regs: &mut RegisterState) {
    super::exc_msg!("segment not present", regs);
    error!("Segment selector: {}", regs.err_code);
}

pub(crate) unsafe extern "sysv64" fn stack_exc_handler(regs: &mut RegisterState) {
    super::exc_msg!("stack exception", regs);
    error!("Segment selector: {}", regs.err_code);
}

pub(crate) unsafe extern "sysv64" fn general_prot_fault_handler(regs: &mut RegisterState) {
    super::exc_msg!("general protection fault", regs);
    error!("Segment selector: {}", regs.err_code);
}
