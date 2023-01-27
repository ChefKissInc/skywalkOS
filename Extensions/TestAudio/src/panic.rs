// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

use iridium_kit::syscall::SystemCall;

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    error!("{info}");
    unsafe { SystemCall::exit() };
}
