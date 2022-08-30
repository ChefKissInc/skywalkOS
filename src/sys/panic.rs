// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use core::arch::asm;

use log::error;
use unwinding::abi::{UnwindContext, UnwindReasonCode, _Unwind_Backtrace, _Unwind_GetIP};

struct CallbackData<'a> {
    counter: usize,
    kern_symbols: &'a [sulphur_dioxide::kern_sym::KernSymbol],
}

extern "C" fn callback(
    unwind_ctx: &mut UnwindContext<'_>,
    arg: *mut core::ffi::c_void,
) -> UnwindReasonCode {
    let data = unsafe { &mut *arg.cast::<CallbackData>() };
    data.counter += 1;

    let ip = _Unwind_GetIP(unwind_ctx) as u64;
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
            super::io::serial::SERIAL.force_unlock();
        }
    }

    info.location().map_or_else(
        || {
            error!("Oops. Your system crashed... A panic has occurred at an unknown location.");
        },
        |loc| {
            error!(
                "Oops. Your system crashed... A panic has occurred at {}@{}:{}.",
                loc.file(),
                loc.line(),
                loc.column()
            );
        },
    );

    info.message().map_or_else(
        || {
            error!("No message provided. Payload: {:#X?}", info.payload());
        },
        |args| {
            args.as_str().map_or_else(
                || {
                    error!("The arguments are: {:#X?}", args);
                },
                |s| {
                    error!("The message is: {}.", s);
                },
            );
        },
    );

    error!("Backtrace:");

    let mut data = CallbackData {
        counter: 0,
        kern_symbols: unsafe {
            (*super::state::SYS_STATE.get())
                .kern_symbols
                .assume_init_mut()
        },
    };
    _Unwind_Backtrace(callback, core::ptr::addr_of_mut!(data).cast());

    loop {
        unsafe { asm!("hlt") };
    }
}
