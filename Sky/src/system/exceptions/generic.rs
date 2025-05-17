// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

super::generic_exception!(div_by_zero, "division by zero");
super::generic_exception!(debug, "debug");
super::generic_exception!(nmi, "non-maskable interrupt");
super::generic_exception!(breakpoint, "breakpoint");
super::generic_exception!(overflow, "overflow");
super::generic_exception!(bound_range, "bound range exceeded");
super::generic_exception!(invalid_opcode, "invalid opcode");
super::generic_exception!(dev_unavailable, "device unavailable");
super::generic_exception!(double_fault, "double fault");
super::generic_exception!(coproc_segment_overrun, "coprocessor segment overrun");

pub unsafe extern "sysv64" fn reserved(regs: &mut crate::system::RegisterState) {
    super::handle_exception(
        "reserved",
        "This should NEVER happen! Make an issue and attach the serial output.",
        regs,
    );
}

super::generic_exception!(x87_fp, "x87 floating-point");
super::generic_exception!(align_check, "alignment check");
super::generic_exception!(machine_check, "machine check");
super::generic_exception!(simd_fp, "SIMD floating-point");
super::generic_exception!(hv_injection, "hypervisor injection");
super::generic_exception!(vmm_communication, "VMM communication");
super::generic_exception!(security, "security");

pub unsafe extern "sysv64" fn spurious(_regs: &mut crate::system::RegisterState) {
    while crate::system::serial::SERIAL.is_locked() {
        crate::system::serial::SERIAL.force_unlock();
    }

    warn!("Received spurious interrupt.");
}
