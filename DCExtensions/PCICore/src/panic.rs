// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use driver_core::system_call::SystemCall;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    error!("{info}");
    unsafe { SystemCall::exit() };
}
