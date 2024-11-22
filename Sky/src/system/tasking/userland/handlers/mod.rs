// Copyright (c) ChefKiss 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use core::{fmt::Write, ops::ControlFlow};

use skykit::TerminationReason;

use crate::system::{tasking::scheduler::Scheduler, RegisterState};

pub mod alloc;
pub mod msg;
pub mod os_dt_entry;
pub mod port;

pub fn kprint(
    scheduler: &Scheduler,
    state: &RegisterState,
) -> ControlFlow<Option<TerminationReason>> {
    let addr = state.rsi;
    let size = state.rdx;

    if !scheduler
        .current_process()
        .unwrap()
        .region_is_valid(addr, size)
    {
        return ControlFlow::Break(Some(TerminationReason::MalformedAddress));
    }

    let s = unsafe { core::slice::from_raw_parts(addr as *const _, size as _) };
    let Ok(s) = core::str::from_utf8(s) else {
        return ControlFlow::Break(Some(TerminationReason::MalformedBody));
    };

    write!(crate::system::serial::SERIAL.lock(), "{s}").unwrap();

    if let Some(v) = unsafe { (*crate::system::state::SYS_STATE.get()).terminal.as_mut() } {
        write!(v, "{s}").unwrap();
    }

    ControlFlow::Continue(())
}
