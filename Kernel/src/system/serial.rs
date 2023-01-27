// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

pub struct SerialWriter(amd64::io::serial::SerialPort);

impl SerialWriter {
    pub fn init(&self) {
        self.0.init();
    }
}

impl core::fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            self.0.transmit(c);
        }

        Ok(())
    }
}

pub static SERIAL: spin::Mutex<SerialWriter> =
    spin::Mutex::new(SerialWriter(amd64::io::serial::SerialPort::new(0x3F8)));
