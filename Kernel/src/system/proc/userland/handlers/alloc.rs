// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use core::ops::ControlFlow;

use fireworkkit::TerminationReason;

use crate::system::{proc::scheduler::Scheduler, RegisterState};

pub fn alloc(
    scheduler: &mut Scheduler,
    state: &mut RegisterState,
) -> ControlFlow<Option<TerminationReason>> {
    let process = scheduler.current_process_mut().unwrap();
    let (addr, pages) = process.allocate(state.rsi);

    unsafe {
        core::ptr::write_bytes(addr as *mut u8, 0, (pages * 0x1000) as _);
    }

    state.rax = addr;
    ControlFlow::Continue(())
}

pub fn free(
    scheduler: &mut Scheduler,
    state: &RegisterState,
) -> ControlFlow<Option<TerminationReason>> {
    let process = scheduler.current_process_mut().unwrap();
    if process.is_msg(state.rsi) {
        return ControlFlow::Break(Some(TerminationReason::MalformedAddress));
    }
    process.free_alloc(state.rsi);
    ControlFlow::Continue(())
}
