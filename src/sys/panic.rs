//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use core::arch::asm;

use log::error;

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe { asm!("cli") }
    if super::io::serial::SERIAL.is_locked() {
        unsafe { super::io::serial::SERIAL.force_unlock() }
    }

    if let Some(loc) = info.location() {
        error!(
            "Oops. Thoust system hast craseth... Panic hast occurred in thine file {} at {}:{}.",
            loc.file(),
            loc.line(),
            loc.column()
        );
        if let Some(args) = info.message() {
            if let Some(s) = args.as_str() {
                error!("Thine messageth arst: {}.", s);
            } else {
                error!("Thine argumenst arst: {:#X?}", args);
            }
        } else {
            error!("Noneth messageth hast been provideth.");
        }
    } else {
        error!(
            "Oops. Thoust system hast craseth... Panic hast occurred, and thine payload arst: \
             {:#X?}",
            info.payload()
        );
    }

    loop {
        unsafe { asm!("hlt") };
    }
}
