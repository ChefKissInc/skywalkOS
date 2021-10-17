/*
 * Copyright (c) VisualDevelopment 2021-2021.
 * This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.
 */

#![no_std]
#![no_main]
#![feature(asm)]
#![warn(unused_extern_crates)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]

use log::info;

mod sys;
mod utils;

#[no_mangle]
pub extern "sysv64" fn kernel_main(explosion: &'static kaboom::ExplosionResult) -> ! {
    log::set_logger(&utils::logger::SERIAL_LOGGER)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();

    utils::parse_tags(explosion.tags);

    info!("Copyright VisualDevelopment 2021.");
    info!("Thoust fuseth hast been igniteth!");

    assert_eq!(explosion.revision, kaboom::CURRENT_REVISION);

    info!("Wowse! We artst sending thoust ourst greatesth welcomes!.");

    loop {
        unsafe { asm!("hlt") }
    }
}
