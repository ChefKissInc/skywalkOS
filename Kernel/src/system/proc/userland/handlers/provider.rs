// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use tungstenkit::syscall::SystemCallStatus;

use crate::system::{proc::scheduler::Scheduler, RegisterState};

pub fn register(scheduler: &mut Scheduler, state: &mut RegisterState) -> SystemCallStatus {
    let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
    if scheduler.providers.try_insert(state.rsi, proc_id).is_err() {
        SystemCallStatus::InvalidRequest
    } else {
        SystemCallStatus::Success
    }
}

pub fn get_for_process(scheduler: &mut Scheduler, state: &mut RegisterState) -> SystemCallStatus {
    if !scheduler.providers.contains_key(&state.rsi) {
        return SystemCallStatus::MalformedData;
    }
    state.rdi = if let Some(&proc_id) = scheduler.providers.get(&state.rsi) {
        proc_id
    } else {
        0
    };
    SystemCallStatus::Success
}
