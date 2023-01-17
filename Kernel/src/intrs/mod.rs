// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use amd64::io::port::Port;

use crate::system::{gdt::PrivilegeLevel, RegisterState};

pub mod idt;
pub mod pic;

unsafe extern "C" fn irq7_quirk(_state: &mut RegisterState) {
    let p = Port::<u8, u8>::new(0x20);
    p.write(0x0B);
    if p.read() & 0x80 != 0 {
        crate::system::state::SYS_STATE
            .get()
            .as_mut()
            .unwrap()
            .lapic
            .get_mut()
            .unwrap()
            .send_eoi();
    }
}

pub fn init_intr_quirks() {
    idt::set_handler(0x27, 0, PrivilegeLevel::Supervisor, irq7_quirk, false, true);
}
