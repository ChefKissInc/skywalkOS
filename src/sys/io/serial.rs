/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

pub struct SerialWriter(amd64::io::serial::SerialPort);

impl SerialWriter {
    pub const fn new(port: amd64::io::serial::SerialPort) -> Self {
        Self { 0: port }
    }

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
    spin::Mutex::new(SerialWriter::new(amd64::io::serial::SerialPort::new(0x3F8)));
