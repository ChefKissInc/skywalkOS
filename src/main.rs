#![no_std]
#![no_main]
#![feature(asm)]
#![warn(unused_extern_crates)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]

mod system;

use core::fmt::Write;

#[no_mangle]
pub extern "sysv64" fn kernel_main(explosion: &'static kaboom::ExplosionResult) -> ! {
    let mut serial = system::io::serial::SERIAL.lock();

    writeln!(serial, "Fuse ignition begun.").unwrap();
    writeln!(serial, "Bootloader data: {:X?}", explosion).unwrap();

    assert_eq!(explosion.revision, kaboom::CURRENT_REVISION);

    writeln!(serial, "Fuse initialization complete.").unwrap();

    loop {
        unsafe { asm!("hlt") }
    }
}
