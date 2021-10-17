/*
 * Copyright (c) VisualDevelopment 2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

use core::fmt::Write;

pub struct SerialLogger;

impl log::Log for SerialLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        let mut serial = crate::sys::io::serial::SERIAL.lock();

        if self.enabled(record.metadata()) {
            writeln!(
                serial,
                "[{}:{}] {}",
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
