// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

use amd64::io::port::PortIO;
use iridium_kit::syscall::{AccessSize, SystemCallStatus};

use crate::system::RegisterState;

pub fn port_in(state: &mut RegisterState) -> SystemCallStatus {
    let port = state.rsi as u16;
    let Ok(access_size) = AccessSize::try_from(state.rdx) else {
        return SystemCallStatus::MalformedData;
    };
    unsafe {
        state.rdi = match access_size {
            AccessSize::Byte => u8::read(port) as u64,
            AccessSize::Word => u16::read(port) as u64,
            AccessSize::DWord => u32::read(port) as u64,
        };
    }
    SystemCallStatus::Success
}

pub fn port_out(state: &mut RegisterState) -> SystemCallStatus {
    let port = state.rsi as u16;
    let Ok(access_size) = AccessSize::try_from(state.rcx) else {
        return SystemCallStatus::MalformedData;
    };
    unsafe {
        match access_size {
            AccessSize::Byte => u8::write(port, state.rdx as u8),
            AccessSize::Word => u16::write(port, state.rdx as u16),
            AccessSize::DWord => u32::write(port, state.rdx as u32),
        };
    }
    SystemCallStatus::Success
}
