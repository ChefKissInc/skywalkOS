// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use crate::system::{proc::scheduler::Scheduler, RegisterState};

pub fn alloc(scheduler: &mut Scheduler, state: &mut RegisterState) {
    let size = state.rsi;
    let pid = scheduler.current_pid.unwrap();
    let process = scheduler.processes.get_mut(&pid).unwrap();
    let addr = process.allocate(size);

    unsafe {
        core::ptr::write_bytes(addr as *mut u8, 0, ((size + 0xFFF) / 0x1000 * 0x1000) as _);
    }

    state.rdi = addr;
}

pub fn free(scheduler: &mut Scheduler, state: &mut RegisterState) {
    let addr = state.rsi;

    let pid = scheduler.current_pid.unwrap();
    let process = scheduler.processes.get_mut(&pid).unwrap();
    process.free_alloc(addr);
}
