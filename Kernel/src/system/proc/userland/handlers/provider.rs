// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use core::ops::ControlFlow;

use crate::system::{proc::scheduler::Scheduler, RegisterState};

pub fn register(scheduler: &mut Scheduler, state: &mut RegisterState) -> ControlFlow<bool> {
    let pid = scheduler.current_pid.unwrap();
    if scheduler.providers.try_insert(state.rsi, pid).is_err() {
        return ControlFlow::Break(true);
    }

    ControlFlow::Continue(())
}

pub fn get(scheduler: &mut Scheduler, state: &mut RegisterState) -> ControlFlow<bool> {
    if !scheduler.providers.contains_key(&state.rsi) {
        return ControlFlow::Break(true);
    }

    state.rax = scheduler
        .providers
        .get(&state.rsi)
        .cloned()
        .unwrap_or_default();
    ControlFlow::Continue(())
}
