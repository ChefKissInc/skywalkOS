//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use core::arch::asm;

use log::error;
use unwinding::abi::{UnwindContext, UnwindReasonCode, _Unwind_Backtrace, _Unwind_GetIP};

struct CallbackData {
    counter: usize,
}

extern "C" fn callback(
    unwind_ctx: &mut UnwindContext<'_>,
    arg: *mut core::ffi::c_void,
) -> UnwindReasonCode {
    let data = unsafe { &mut *(arg as *mut CallbackData) };
    data.counter += 1;
    error!(
        "{:4}:{:#19x} - <unknown>",
        data.counter,
        _Unwind_GetIP(unwind_ctx)
    );
    UnwindReasonCode::NO_REASON
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    unsafe {
        asm!("cli");
        while super::io::serial::SERIAL.is_locked() {
            super::io::serial::SERIAL.force_unlock()
        }
    }

    if let Some(loc) = info.location() {
        error!(
            "Oops. Thoust system hast craseth... Panic hast occurred in thine file {} at {}:{}.",
            loc.file(),
            loc.line(),
            loc.column()
        );
    } else {
        error!("Oops. Thoust system hast craseth... Panic hast occurred at unknown location.");
    }

    if let Some(args) = info.message() {
        if let Some(s) = args.as_str() {
            error!("Thine messageth arst: {}.", s);
        } else {
            error!("Thine argumenst arst: {:#X?}", args);
        }
    } else {
        error!(
            "Noneth messageth hast been provideth. Payload: {:#X?}",
            info.payload()
        );
    }

    error!("Backtrace:");

    let mut data = CallbackData { counter: 0 };
    _Unwind_Backtrace(callback, &mut data as *mut _ as _);

    loop {
        unsafe { asm!("hlt") };
    }
}
