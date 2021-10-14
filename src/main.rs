#![no_std]
#![no_main]
#![feature(asm)]
#![warn(unused_extern_crates)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]

use core::fmt::Write;

use log::info;

mod sys;
mod utils;

struct SerialLogger;

impl log::Log for SerialLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Trace
    }

    fn log(&self, record: &log::Record) {
        let mut serial = sys::io::serial::SERIAL.lock();

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

static SERIAL_LOGGER: SerialLogger = SerialLogger;

#[no_mangle]
pub extern "sysv64" fn kernel_main(explosion: &'static kaboom::ExplosionResult) -> ! {
    log::set_logger(&SERIAL_LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();

    info!("Copyright VisualDevelopment 2021.");
    info!("Thoust fuseth hast been igniteth!");

    assert_eq!(explosion.revision, kaboom::CURRENT_REVISION);

    info!("Wowse! We artst sending thoust ourst greatesth welcomes!.");

    loop {
        unsafe { asm!("hlt") }
    }
}
