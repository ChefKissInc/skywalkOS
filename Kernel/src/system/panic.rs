// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use unwinding::abi::{UnwindContext, UnwindReasonCode, _Unwind_Backtrace, _Unwind_GetIP};

struct CallbackData<'a> {
    counter: usize,
    kern_symbols: &'a [sulphur_dioxide::KernSymbol],
}

extern "C" fn callback(
    unwind_ctx: &mut UnwindContext<'_>,
    arg: *mut core::ffi::c_void,
) -> UnwindReasonCode {
    let data = unsafe { arg.cast::<CallbackData>().as_mut().unwrap() };
    data.counter += 1;

    let ip = _Unwind_GetIP(unwind_ctx) as u64;
    data.kern_symbols
        .iter()
        .find(|v| ip >= v.start && ip < v.end)
        .map_or_else(
            || {
                error!("{:>4}: {ip:>#19X}+{0:>#04X} -> ???", data.counter);
            },
            |symbol| {
                rustc_demangle::try_demangle(symbol.name).map_or_else(
                    |_| {
                        error!(
                            "{:>4}: {:>#19X}+{:>#04X} -> {}",
                            data.counter,
                            symbol.start,
                            ip - symbol.start,
                            symbol.name
                        );
                    },
                    |demangled| {
                        error!(
                            "{:>4}: {:>#19X}+{:>#04X} -> {demangled:#}",
                            data.counter,
                            symbol.start,
                            ip - symbol.start,
                        );
                    },
                );
            },
        );

    UnwindReasonCode::NO_REASON
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::cli!();
    #[cfg(debug_assertions)]
    while super::serial::SERIAL.is_locked() {
        unsafe { super::serial::SERIAL.force_unlock() }
    }
    let state = unsafe { super::state::SYS_STATE.get().as_mut().unwrap() };

    if state.in_panic {
        error!("Panicked while panicking!");

        crate::hlt_loop!();
    }
    state.in_panic = true;

    error!("{info}");
    error!("Backtrace:");
    let mut data = CallbackData {
        counter: 0,
        kern_symbols: state.kern_symbols.get_mut().unwrap(),
    };
    _Unwind_Backtrace(callback, core::ptr::addr_of_mut!(data).cast());

    if let Some(ctx) = state.interrupt_context {
        data.counter = 0;
        error!("In interrupt:");
        error!("    {ctx:#X?}");
        error!("Interrupt backtrace:");
        let mut rbp = ctx.rbp;
        loop {
            if rbp == 0 {
                error!("End of backtrace.");
                break;
            }
            let ip = unsafe { *((rbp + 8) as *const u64) };
            rbp = unsafe { *(rbp as *const u64) };

            data.counter += 1;

            data.kern_symbols
                .iter()
                .find(|v| ip >= v.start && ip < v.end)
                .map_or_else(
                    || {
                        error!("{:>4}: {:>#19X}+{:>#04X} -> ???", data.counter, ip, 0);
                    },
                    |symbol| {
                        rustc_demangle::try_demangle(symbol.name).map_or_else(
                            |_| {
                                error!(
                                    "{:>4}: {:>#19X}+{:>#04X} -> {}",
                                    data.counter,
                                    symbol.start,
                                    ip - symbol.start,
                                    symbol.name
                                );
                            },
                            |demangled| {
                                error!(
                                    "{:>4}: {:>#19X}+{:>#04X} -> {:#}",
                                    data.counter,
                                    symbol.start,
                                    ip - symbol.start,
                                    demangled
                                );
                            },
                        );
                    },
                );
        }
    }

    crate::hlt_loop!();
}
