// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use amd64::io::port::Port;

const ICW1_ICW4: u8 = 0x01;
const ICW1_INIT: u8 = 0x10;
const ICW4_8086: u8 = 0x01;

pub struct ProgrammableIntrController {
    pub master_cmd: Port<u8, u8>,
    pub master_data: Port<u8, u8>,
    pub slave_cmd: Port<u8, u8>,
    pub slave_data: Port<u8, u8>,
}

impl ProgrammableIntrController {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            master_cmd: Port::new(0x20),
            master_data: Port::new(0x21),
            slave_cmd: Port::new(0xA0),
            slave_data: Port::new(0xA1),
        }
    }

    pub fn remap_and_disable(&self) {
        unsafe {
            self.master_cmd.write(ICW1_INIT | ICW1_ICW4);
            self.slave_cmd.write(ICW1_INIT | ICW1_ICW4);
            self.master_data.write(32);
            self.master_data.write(4); // slave PIC at IRQ2
            self.slave_data.write(2); // slave PIC cascade identity
            self.master_data.write(ICW4_8086);
            self.slave_data.write(ICW4_8086);

            // Mask all IRQs
            self.master_data.write(0xFF);
            self.slave_data.write(0xFF);
        }
    }
}
