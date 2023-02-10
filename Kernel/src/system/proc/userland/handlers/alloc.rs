// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use tungstenkit::syscall::SystemCallStatus;

use crate::system::{proc::scheduler::Scheduler, RegisterState};

pub fn alloc(scheduler: &mut Scheduler, state: &mut RegisterState) -> SystemCallStatus {
    let size = state.rsi;
    let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
    let process = scheduler.processes.get_mut(&proc_id).unwrap();
    let addr = process.allocate(size);

    unsafe {
        core::ptr::write_bytes(addr as *mut u8, 0, ((size + 0xFFF) / 0x1000 * 0x1000) as _);
    }

    state.rdi = addr;
    SystemCallStatus::Success
}

pub fn free(scheduler: &mut Scheduler, state: &mut RegisterState) -> SystemCallStatus {
    let addr = state.rsi;

    let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
    let process = scheduler.processes.get_mut(&proc_id).unwrap();
    process.free_alloc(addr);

    SystemCallStatus::Success
}
