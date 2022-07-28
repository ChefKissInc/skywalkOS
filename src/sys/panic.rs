//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use core::arch::asm;

use log::error;
use unwinding::abi::{UnwindContext, UnwindReasonCode, _Unwind_Backtrace, _Unwind_GetIP};

struct CallbackData<'a> {
    counter: usize,
    kern_symbols: &'a [sulphur_dioxide::symbol::KernSymbol],
}

extern "C" fn callback(
    unwind_ctx: &mut UnwindContext<'_>,
    arg: *mut core::ffi::c_void,
) -> UnwindReasonCode {
    let data = unsafe { &mut *(arg as *mut CallbackData) };
    data.counter += 1;

    let ip = _Unwind_GetIP(unwind_ctx);
    if let Some(symbol) = data
        .kern_symbols
        .iter()
        .find(|v| ip >= v.start && ip < v.end)
    {
        if let Ok(demangled) = rustc_demangle::try_demangle(symbol.name) {
            error!(
                "{:>4}: {:>#19X}+{:>#04X} -> {:#}",
                data.counter,
                symbol.start,
                ip - symbol.start,
                demangled
            );
        } else {
            error!(
                "{:>4}: {:>#19X}+{:>#04X} -> {}",
                data.counter,
                symbol.start,
                ip - symbol.start,
                symbol.name
            );
        }
    } else {
        error!("{:>4}: {:>#19X}+{:>#04X} -> ???", data.counter, ip, 0);
    }

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

    let mut data = CallbackData {
        counter: 0,
        kern_symbols: unsafe {
            (*super::state::SYS_STATE.get())
                .kern_symbols
                .assume_init_mut()
        },
    };
    _Unwind_Backtrace(callback, &mut data as *mut _ as _);

    loop {
        unsafe { asm!("hlt") };
    }
}
