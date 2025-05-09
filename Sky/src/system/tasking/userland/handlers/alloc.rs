// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use core::ops::ControlFlow;

use skykit::TerminationReason;

use crate::system::{tasking::scheduler::Scheduler, RegisterState};

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
    let addr = state.rsi;

    let process = scheduler.current_process_mut().unwrap();
    if process.is_msg(addr) {
        return ControlFlow::Break(Some(TerminationReason::MalformedAddress));
    }

    let size = state.rdx;
    if process.region_is_mapped(addr, size) {
        process.free_alloc(state.rsi);
        ControlFlow::Continue(())
    } else {
        ControlFlow::Break(Some(TerminationReason::MalformedArgument))
    }
}
