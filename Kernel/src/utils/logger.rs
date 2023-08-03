// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use core::fmt::Write;

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        writeln!(
            crate::system::serial::SERIAL.lock(),
            "{} {} > {}",
            record.level(),
            record.target(),
            record.args()
        )
        .unwrap();

        let state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
        if record.metadata().level() <= log::Level::Info || state.verbose {
            let Some(terminal) = &mut state.terminal else {
                return;
            };
            writeln!(
                terminal,
                "{} {} > {}",
                record.level(),
                record.target(),
                record.args()
            )
            .unwrap();
        }
    }

    fn flush(&self) {}
}

pub static LOGGER: Logger = Logger;

pub fn init() {
    crate::system::serial::SERIAL.lock().init();

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
}
