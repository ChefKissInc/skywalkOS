//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use core::fmt::Write;

pub struct FuseLogger;

impl log::Log for FuseLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let mut serial = crate::sys::io::serial::SERIAL.lock();

        writeln!(
            serial,
            "[{}] {}: {}",
            record.level(),
            record.target(),
            record.args()
        )
        .unwrap();

        unsafe {
            let verbose = (*crate::sys::state::SYS_STATE.get()).boot_settings.verbose;
            if record.metadata().level() <= log::Level::Info || verbose {
                if let Some(terminal) = &mut (*crate::sys::state::SYS_STATE.get()).terminal {
                    writeln!(
                        terminal,
                        "[{}] {}: {}",
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

pub static LOGGER: FuseLogger = FuseLogger;
