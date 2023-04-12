// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use core::fmt::Write;

use tungstenkit::syscall::SystemCall;

pub struct KWriter;

impl Write for KWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe { SystemCall::kprint(s) }
        Ok(())
    }
}

pub struct KLog;

impl log::Log for KLog {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        writeln!(
            KWriter,
            "{} {} > {}",
            record.level(),
            record.target(),
            record.args()
        )
        .unwrap();
    }

    fn flush(&self) {}
}

pub static LOGGER: KLog = KLog;

pub fn init() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
}
