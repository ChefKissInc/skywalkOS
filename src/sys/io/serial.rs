/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

pub struct SerialWriter(amd64::io::SerialPort<u8>);

impl SerialWriter {
    pub const fn new(port: amd64::io::SerialPort<u8>) -> Self {
        Self { 0: port }
    }
}

impl core::fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            unsafe { self.0.write(c as u8) };
        }

        Ok(())
    }
}

pub static SERIAL: spin::Mutex<SerialWriter> =
    spin::Mutex::new(SerialWriter::new(amd64::io::SerialPort::<u8>::new(0x3F8)));
