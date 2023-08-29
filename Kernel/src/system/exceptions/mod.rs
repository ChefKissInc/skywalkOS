// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

macro_rules! exception_msg {
    ($name:expr, $msg:expr, $regs:expr) => {
        while crate::system::serial::SERIAL.is_locked() {
            crate::system::serial::SERIAL.force_unlock()
        }

        let sys_state = &mut *crate::system::state::SYS_STATE.get();

        if $regs.cs.trailing_zeros() >= 2 {
            sys_state.interrupt_context = Some(*$regs);
            panic!("Received {} exception: {}", $name, $msg);
        } else {
            use core::fmt::Write;
            let mut scheduler = sys_state.scheduler.as_ref().unwrap().lock();
            let cur_proc = scheduler.current_process().unwrap();
            let image_base = cur_proc.image_base;
            let proc_path = &cur_proc.path;
            writeln!(
                crate::system::serial::SERIAL.lock(),
                "Received {} exception in user-land: {}",
                $name,
                $msg
            )
            .unwrap();
            writeln!(crate::system::serial::SERIAL.lock(), "{}", $regs).unwrap();
            writeln!(
                crate::system::serial::SERIAL.lock(),
                "Image Base: {image_base:#018X}"
            )
            .unwrap();
            writeln!(
                crate::system::serial::SERIAL.lock(),
                "Process Path: {proc_path}"
            )
            .unwrap();

            if sys_state.verbose {
                if let Some(v) = sys_state.terminal.as_mut() {
                    writeln!(v, "Received {} exception in user-land: {}", $name, $msg).unwrap();
                    writeln!(v, "{}", $regs).unwrap();
                    writeln!(v, "Image Base: {image_base:#018X}").unwrap();
                    writeln!(v, "Process Path: {proc_path}").unwrap();
                }
            }

            scheduler.process_teardown();
            scheduler.schedule($regs);
        }
    };
}

macro_rules! generic_exception {
    ($ident:ident, $name:expr) => {
        pub unsafe extern "sysv64" fn $ident(regs: &mut crate::system::RegisterState) {
            exception_msg!($name, "<No Additional Information>", regs);
        }
    };
}

use exception_msg;
use generic_exception;

use crate::system::gdt::PrivilegeLevel;

mod gdt;
mod generic;
mod page_fault;

pub fn init() {
    debug!("Initialising.");
    let dpl = PrivilegeLevel::Supervisor;
    crate::interrupts::idt::set_handler(0, 0, dpl, generic::div_by_zero, false, true);
    crate::interrupts::idt::set_handler(1, 0, dpl, generic::debug, false, true);
    crate::interrupts::idt::set_handler(2, 0, dpl, generic::nmi, false, false);
    crate::interrupts::idt::set_handler(3, 0, dpl, generic::breakpoint, false, true);
    crate::interrupts::idt::set_handler(4, 0, dpl, generic::overflow, false, true);
    crate::interrupts::idt::set_handler(5, 0, dpl, generic::bound_range, false, true);
    crate::interrupts::idt::set_handler(6, 0, dpl, generic::invalid_opcode, false, true);
    crate::interrupts::idt::set_handler(7, 0, dpl, generic::dev_unavailable, false, true);
    crate::interrupts::idt::set_handler(8, 1, dpl, generic::double_fault, false, false);
    crate::interrupts::idt::set_handler(9, 0, dpl, generic::coproc_segment_overrun, false, false);
    crate::interrupts::idt::set_handler(10, 0, dpl, gdt::invalid_tss, false, false);
    crate::interrupts::idt::set_handler(11, 0, dpl, gdt::segment_not_present, false, false);
    crate::interrupts::idt::set_handler(12, 0, dpl, gdt::stack_exception, false, true);
    crate::interrupts::idt::set_handler(13, 0, dpl, gdt::general_protection_fault, false, true);
    crate::interrupts::idt::set_handler(14, 0, dpl, page_fault::page_fault, false, true);
    crate::interrupts::idt::set_handler(15, 0, dpl, generic::reserved, false, false);
    crate::interrupts::idt::set_handler(16, 0, dpl, generic::x87_fp, false, true);
    crate::interrupts::idt::set_handler(17, 0, dpl, generic::align_check, false, true);
    crate::interrupts::idt::set_handler(18, 0, dpl, generic::machine_check, false, false);
    crate::interrupts::idt::set_handler(19, 0, dpl, generic::simd_fp, false, true);
    crate::interrupts::idt::set_handler(20, 0, dpl, generic::reserved, false, false);
    crate::interrupts::idt::set_handler(21, 0, dpl, generic::reserved, false, false);
    for i in 22..28 {
        crate::interrupts::idt::set_handler(i, 0, dpl, generic::reserved, false, false);
    }
    crate::interrupts::idt::set_handler(28, 0, dpl, generic::hv_injection, false, true);
    crate::interrupts::idt::set_handler(29, 0, dpl, generic::vmm_communication, false, true);
    crate::interrupts::idt::set_handler(30, 0, dpl, generic::security, false, true);
    crate::interrupts::idt::set_handler(31, 0, dpl, generic::reserved, false, false);
    crate::interrupts::idt::set_handler(255, 0, dpl, generic::spurious, true, true);
}
