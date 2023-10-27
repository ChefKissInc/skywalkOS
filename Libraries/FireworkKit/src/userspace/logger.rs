// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use core::fmt::Write;

use crate::syscall::SystemCall;

pub struct KWriter;

impl Write for KWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            core::arch::asm!(
                "int 249",
                in("rdi") SystemCall::KPrint as u64,
                in("rsi") s.as_ptr() as u64,
                in("rdx") s.len() as u64,
                options(nostack),
            );
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
