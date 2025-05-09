// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use crate::{syscall::SystemCall, userspace::logger::KWriter};
use core::{
    fmt::Write,
    sync::atomic::{AtomicBool, Ordering},
};

static IN_PANIC: AtomicBool = AtomicBool::new(false);

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    if IN_PANIC.swap(true, Ordering::AcqRel) {
        let _ = writeln!(KWriter, "double panic");
    } else {
        writeln!(KWriter, "{info}").unwrap();
    }
    unsafe { SystemCall::quit() }
}
