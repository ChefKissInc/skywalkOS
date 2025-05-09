// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use crate::syscall::SystemCall;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    error!("{info}");
    unsafe { SystemCall::quit() }
}
