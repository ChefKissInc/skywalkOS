// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use core::{fmt::Write, ops::ControlFlow};

use fireworkkit::TerminationReason;

use crate::system::RegisterState;

pub mod alloc;
pub mod msg;
pub mod osdtentry;
pub mod port;

pub fn kprint(state: &RegisterState) -> ControlFlow<Option<TerminationReason>> {
    let s = unsafe { core::slice::from_raw_parts(state.rsi as *const _, state.rdx as _) };
    let Ok(s) = core::str::from_utf8(s) else {
        return ControlFlow::Break(Some(TerminationReason::MalformedBody));
    };

    write!(crate::system::serial::SERIAL.lock(), "{s}").unwrap();

    if let Some(v) = unsafe { (*crate::system::state::SYS_STATE.get()).terminal.as_mut() } {
        write!(v, "{s}").unwrap();
    }

    ControlFlow::Continue(())
}
