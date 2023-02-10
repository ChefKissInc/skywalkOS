// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use crate::system::{proc::scheduler::Scheduler, RegisterState};

pub fn register(scheduler: &mut Scheduler, state: &mut RegisterState) {
    let pid = scheduler.current_pid.unwrap();
    if scheduler.providers.try_insert(state.rsi, pid).is_err() {
        todo!()
    }
}

pub fn get(scheduler: &mut Scheduler, state: &mut RegisterState) {
    if !scheduler.providers.contains_key(&state.rsi) {
        todo!()
    }
    state.rax = scheduler
        .providers
        .get(&state.rsi)
        .cloned()
        .unwrap_or_default();
}
