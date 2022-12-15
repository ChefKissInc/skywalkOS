// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use core::fmt::Write;

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        #[cfg(debug_assertions)]
        writeln!(
            crate::sys::io::serial::SERIAL.lock(),
            "{} [{}] > {}",
            record.level(),
            record.target(),
            record.args()
        )
        .unwrap();

        let state = unsafe { crate::sys::state::SYS_STATE.get().as_mut().unwrap() };
        let verbose = state.boot_settings.verbose;
        if record.metadata().level() <= log::Level::Info || verbose {
            if let Some(terminal) = &mut state.terminal {
                writeln!(
                    terminal,
                    "{} [{}] > {}",
                    record.level(),
                    record.target(),
                    record.args()
                )
                .unwrap();
            }
        }
    }

    fn flush(&self) {}
}

pub static LOGGER: Logger = Logger;

pub fn init() {
    #[cfg(debug_assertions)]
    crate::sys::io::serial::SERIAL.lock().init();

    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
}
