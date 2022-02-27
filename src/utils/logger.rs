//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use core::fmt::Write;

pub struct SerialLogger;

impl log::Log for SerialLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut serial = crate::sys::io::serial::SERIAL.lock();

            writeln!(
                serial,
                "[{} | {}] {}",
                record.target(),
                record.level(),
                record.args()
            )
            .unwrap();
        }
    }

    fn flush(&self) {}
}

pub static SERIAL_LOGGER: SerialLogger = SerialLogger {};
