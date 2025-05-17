// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use amd64::io::port::Port;

use crate::system::{gdt::PrivilegeLevel, RegisterState};

pub mod idt;
pub mod pic;

unsafe extern "sysv64" fn irq7_quirk(_state: &mut RegisterState) {
    let p = Port::<u8, u8>::new(0x20);
    p.write(0x0B);
    if p.read() & 0x80 != 0 {
        (*crate::system::state::SYS_STATE.get())
            .lapic
            .as_ref()
            .unwrap()
            .send_eoi();
    }
}

pub fn init_quirks() {
    idt::set_handler(0x27, 0, PrivilegeLevel::Supervisor, irq7_quirk, false);
}
