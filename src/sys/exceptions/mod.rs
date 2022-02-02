/*
 * Copyright (c) VisualDevelopment 2021-2022.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

macro_rules! exc_msg {
    ($name:expr, $regs:expr) => {
        use log::error;

        if crate::sys::io::serial::SERIAL.is_locked() {
            crate::sys::io::serial::SERIAL.force_unlock()
        }

        error!("Received {} exception!", $name);
        error!("CPU registers: {:#X?}", $regs);
    };
}

macro_rules! generic_exception {
    ($ident:ident, $name:expr) => {
        pub(crate) unsafe extern "sysv64" fn $ident(regs: &mut amd64::sys::cpu::RegisterState) {
            exc_msg!($name, regs);
        }
    };
}

pub(self) use exc_msg;
pub(self) use generic_exception;

mod gdt;
mod generic;
mod page_fault;

pub fn init() {
    super::idt::set_handler(0, generic::div0_handler, false, false);
    super::idt::set_handler(1, generic::debug_handler, false, false);
    super::idt::set_handler(2, generic::nmi_handler, false, false);
    super::idt::set_handler(3, generic::breakpoint_handler, false, false);
    super::idt::set_handler(4, generic::overflow_handler, false, false);
    super::idt::set_handler(5, generic::bound_range_handler, false, false);
    super::idt::set_handler(6, generic::invalid_opcode_handler, false, false);
    super::idt::set_handler(7, generic::dev_unavailable_handler, false, false);
    super::idt::set_handler(8, generic::coproc_segment_overrun_handler, false, false);
    super::idt::set_handler(9, generic::overflow_handler, false, false);
    super::idt::set_handler(10, gdt::invalid_tss_handler, false, false);
    super::idt::set_handler(11, gdt::segment_not_present_handler, false, false);
    super::idt::set_handler(12, gdt::stack_exc_handler, false, false);
    super::idt::set_handler(13, gdt::general_prot_fault_handler, false, false);
    super::idt::set_handler(14, page_fault::handler, false, false);
    super::idt::set_handler(15, generic::reserved_handler, false, false);
    super::idt::set_handler(16, generic::x87_fp_handler, false, false);
    super::idt::set_handler(17, generic::align_chk_handler, false, false);
    super::idt::set_handler(18, generic::machine_chk_handler, false, false);
    super::idt::set_handler(19, generic::simd_fp_handler, false, false);
    super::idt::set_handler(20, generic::reserved_handler, false, false);
    super::idt::set_handler(21, generic::x87_fp_handler, false, false);
    for i in 22..28 {
        super::idt::set_handler(i, generic::reserved_handler, false, false);
    }
    super::idt::set_handler(28, generic::hv_injection_handler, false, false);
    super::idt::set_handler(29, generic::vmm_com_handler, false, false);
    super::idt::set_handler(30, generic::security_handler, false, false);
    super::idt::set_handler(31, generic::reserved_handler, false, false);
}
