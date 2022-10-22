// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

macro_rules! exc_msg {
    ($name:expr, $msg:expr, $regs:expr) => {
        while crate::sys::io::serial::SERIAL.is_locked() {
            crate::sys::io::serial::SERIAL.force_unlock()
        }

        if $regs.cs.trailing_zeros() >= 2 {
            crate::sys::state::SYS_STATE
                .get()
                .as_mut()
                .unwrap()
                .interrupt_context = Some(*$regs);
            panic!("Received {} exception: {}", $name, $msg);
        } else {
            error!("Received {} exception in user-land: {}", $name, $msg);
            error!("{:#X?}", $regs);
        }
    };
}

macro_rules! generic_exception {
    ($ident:ident, $name:expr) => {
        pub unsafe extern "sysv64" fn $ident(regs: &mut crate::sys::RegisterState) {
            exc_msg!($name, "<No Additional Information>", regs);
        }
    };
}

pub(self) use exc_msg;
pub(self) use generic_exception;

use crate::sys::gdt::PrivilegeLevel;

mod gdt;
mod generic;
mod page_fault;

pub fn init() {
    debug!("Initialising.");
    let dpl = PrivilegeLevel::Supervisor;
    super::idt::set_handler(0, 0, dpl, generic::div0_handler, false, false);
    super::idt::set_handler(1, 0, dpl, generic::debug_handler, false, false);
    super::idt::set_handler(2, 0, dpl, generic::nmi_handler, false, false);
    super::idt::set_handler(3, 0, dpl, generic::breakpoint_handler, false, false);
    super::idt::set_handler(4, 0, dpl, generic::overflow_handler, false, false);
    super::idt::set_handler(5, 0, dpl, generic::bound_range_handler, false, false);
    super::idt::set_handler(6, 0, dpl, generic::invalid_opcode_handler, false, false);
    super::idt::set_handler(7, 0, dpl, generic::dev_unavailable_handler, false, false);
    super::idt::set_handler(8, 1, dpl, generic::double_fault, false, false);
    super::idt::set_handler(
        9,
        0,
        dpl,
        generic::coproc_segment_overrun_handler,
        false,
        false,
    );
    super::idt::set_handler(10, 0, dpl, gdt::invalid_tss_handler, false, false);
    super::idt::set_handler(11, 0, dpl, gdt::segment_not_present_handler, false, false);
    super::idt::set_handler(12, 0, dpl, gdt::stack_exc_handler, false, false);
    super::idt::set_handler(13, 0, dpl, gdt::general_prot_fault_handler, false, false);
    super::idt::set_handler(14, 0, dpl, page_fault::handler, false, false);
    super::idt::set_handler(15, 0, dpl, generic::reserved_handler, false, false);
    super::idt::set_handler(16, 0, dpl, generic::x87_fp_handler, false, false);
    super::idt::set_handler(17, 0, dpl, generic::align_chk_handler, false, false);
    super::idt::set_handler(18, 0, dpl, generic::machine_chk_handler, false, false);
    super::idt::set_handler(19, 0, dpl, generic::simd_fp_handler, false, false);
    super::idt::set_handler(20, 0, dpl, generic::reserved_handler, false, false);
    super::idt::set_handler(21, 0, dpl, generic::reserved_handler, false, false);
    for i in 22..28 {
        super::idt::set_handler(i, 0, dpl, generic::reserved_handler, false, false);
    }
    super::idt::set_handler(28, 0, dpl, generic::hv_injection_handler, false, false);
    super::idt::set_handler(29, 0, dpl, generic::vmm_com_handler, false, false);
    super::idt::set_handler(30, 0, dpl, generic::security_handler, false, false);
    super::idt::set_handler(31, 0, dpl, generic::reserved_handler, false, false);
    super::idt::set_handler(255, 0, dpl, generic::spurious, true, true);
}
