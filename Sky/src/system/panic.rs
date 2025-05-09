// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use unwinding::abi::{UnwindContext, UnwindReasonCode, _Unwind_Backtrace, _Unwind_GetIP};

struct CallbackData<'a> {
    counter: usize,
    kern_symbols: &'a [skyliftkit::KernSymbol],
}

extern "C" fn callback(
    unwind_ctx: &UnwindContext<'_>,
    arg: *mut core::ffi::c_void,
) -> UnwindReasonCode {
    let data = unsafe { &mut *arg.cast::<CallbackData>() };
    data.counter += 1;

    let ip = _Unwind_GetIP(unwind_ctx) as u64;
    data.kern_symbols
        .iter()
        .find(|v| ip >= v.start && ip < v.end)
        .map_or_else(
            || {
                error!("{:>4}: {ip:#018X}+{:#06X} -> ???", data.counter, 0);
            },
            |symbol| {
                error!(
                    "{:>4}: {:#018X}+{:#06X} -> {}",
                    data.counter,
                    symbol.start,
                    ip - symbol.start,
                    rustc_demangle::demangle(symbol.name)
                );
            },
        );

    UnwindReasonCode::NO_REASON
}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    crate::cli!();
    while super::serial::SERIAL.is_locked() {
        unsafe { super::serial::SERIAL.force_unlock() }
    }
    let state = unsafe { &mut *super::state::SYS_STATE.get() };

    if state.in_panic.load(core::sync::atomic::Ordering::Relaxed) {
        error!("Panicked while panicking!");

        crate::hlt_loop!();
    }
    state
        .in_panic
        .store(true, core::sync::atomic::Ordering::Relaxed);

    error!("{info}");
    error!("Backtrace:");
    let mut data = CallbackData {
        counter: 0,
        kern_symbols: state.kern_symbols.as_ref().unwrap(),
    };
    _Unwind_Backtrace(callback, core::ptr::addr_of_mut!(data).cast());

    if let Some(ctx) = state.interrupt_context {
        data.counter = 0;
        error!("In interrupt:");
        error!("    {ctx}");
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
                        error!("{:>4}: {ip:#018X}+{:#06X} -> ???", data.counter, 0);
                    },
                    |symbol| {
                        error!(
                            "{:>4}: {:#018X}+{:#06X} -> {}",
                            data.counter,
                            symbol.start,
                            ip - symbol.start,
                            rustc_demangle::demangle(symbol.name)
                        );
                    },
                );
        }
    }

    crate::hlt_loop!();
}
