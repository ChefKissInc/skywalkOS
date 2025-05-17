// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

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

    if let Some(ctx) = state.interrupt_context {
        error!("In interrupt:");
        error!("    {ctx}");
    }

    crate::hlt_loop!();
}
