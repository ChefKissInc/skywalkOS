// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use core::fmt::Write;

use driver_core::system_call::SystemCall;

pub struct KWriter;

impl Write for KWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            SystemCall::kprint(s).unwrap();
        }
        Ok(())
    }
}

pub struct KLog;

impl log::Log for KLog {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        writeln!(KWriter, "{}: {}", record.target(), record.args()).unwrap();
    }

    fn flush(&self) {}
}

pub static LOGGER: KLog = KLog;

pub fn init() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
}
