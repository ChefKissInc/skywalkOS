//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use core::{cell::UnsafeCell, fmt::Write};

use kaboom::tags::SpecialisedSettings;

use crate::sys::terminal::Terminal;

pub struct FuseLogger {
    pub terminal: UnsafeCell<spin::Once<&'static mut Terminal>>,
}

unsafe impl Sync for FuseLogger {}

impl FuseLogger {
    pub const fn new() -> Self {
        Self {
            terminal: UnsafeCell::new(spin::Once::new()),
        }
    }
}

impl log::Log for FuseLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let mut serial = crate::sys::io::serial::SERIAL.lock();

        writeln!(
            serial,
            "{}//{}: {}",
            record.level(),
            record.target(),
            record.args()
        )
        .unwrap();

        unsafe {
            let verbose = crate::sys::state::SYS_STATE
                .boot_settings
                .get()
                .unwrap_or(&SpecialisedSettings { verbose: false })
                .verbose;
            if record.metadata().level() <= log::Level::Info || verbose {
                if let Some(terminal) = (&mut *self.terminal.get()).get_mut() {
                    writeln!(
                        terminal,
                        "{}//{}: {}",
                        record.level(),
                        record.target(),
                        record.args()
                    )
                    .unwrap();
                }
            }
        }
    }

    fn flush(&self) {}
}

pub static LOGGER: FuseLogger = FuseLogger::new();
