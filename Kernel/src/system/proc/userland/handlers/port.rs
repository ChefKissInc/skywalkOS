// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use core::ops::ControlFlow;

use amd64::io::port::PortIO;
use tungstenkit::{syscall::AccessSize, ExitReason};

use crate::system::RegisterState;

pub fn port_in(state: &mut RegisterState) -> ControlFlow<Option<ExitReason>> {
    let port = state.rsi as u16;
    let Ok(access_size) = AccessSize::try_from(state.rdx) else {
        return ControlFlow::Break(Some(ExitReason::InvalidArgument));
    };
    unsafe {
        state.rax = match access_size {
            AccessSize::Byte => u64::from(u8::read(port)),
            AccessSize::Word => u64::from(u16::read(port)),
            AccessSize::DWord => u64::from(u32::read(port)),
        };
    }
    ControlFlow::Continue(())
}

pub fn port_out(state: &mut RegisterState) -> ControlFlow<Option<ExitReason>> {
    let port = state.rsi as u16;
    let Ok(access_size) = AccessSize::try_from(state.rcx) else {
        return ControlFlow::Break(Some(ExitReason::InvalidArgument));
    };
    unsafe {
        match access_size {
            AccessSize::Byte => u8::write(port, state.rdx as u8),
            AccessSize::Word => u16::write(port, state.rdx as u16),
            AccessSize::DWord => u32::write(port, state.rdx as u32),
        };
    }
    ControlFlow::Continue(())
}
