/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

#![no_std]
#![no_main]
#![deny(warnings, clippy::cargo, unused_extern_crates)]
#![feature(asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(const_raw_ptr_deref)]

extern crate alloc;

use alloc::boxed::Box;
use log::{debug, info};

mod sys;
mod utils;

#[no_mangle]
pub extern "sysv64" fn kernel_main(explosion: &'static kaboom::ExplosionResult) -> ! {
    sys::io::serial::SERIAL.lock().init();

    log::set_logger(&utils::logger::SERIAL_LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();

    utils::parse_tags(explosion.tags);

    // At this point, memory allocations are now possible
    info!("Copyright VisualDevelopment 2021.");
    info!("Thoust fuseth hast been igniteth!");

    assert_eq!(explosion.revision, kaboom::CURRENT_REVISION);

    let test = Box::new(5);
    debug!("test = {:#X?}", test);
    core::mem::drop(test);

    info!("Wowse! We artst sending thoust ourst greatesth welcomes!.");

    loop {
        unsafe { asm!("hlt") }
    }
}
