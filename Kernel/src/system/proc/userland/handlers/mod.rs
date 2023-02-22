// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use core::fmt::Write;

use crate::system::{proc::scheduler::Scheduler, RegisterState};

pub mod alloc;
pub mod device_tree;
pub mod message;
pub mod port;
pub mod provider;

pub fn kprint(state: &mut RegisterState) {
    // TODO: kill process on failure
    let s = unsafe { core::slice::from_raw_parts(state.rsi as *const u8, state.rdx as usize) };
    let s = unsafe { core::str::from_utf8_unchecked(s) };

    #[cfg(debug_assertions)]
    write!(crate::system::serial::SERIAL.lock(), "{s}").unwrap();

    let sys_state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
    if let Some(terminal) = &mut sys_state.terminal {
        write!(terminal, "{s}").unwrap();
    }
}

pub fn thread_teardown(scheduler: &mut Scheduler) {
    let id = scheduler.current_tid.unwrap();
    scheduler.threads.remove(&id);
    scheduler.tid_gen.free(id);
    scheduler.current_tid = None;

    let pid = scheduler.current_pid.unwrap();
    if !scheduler.threads.iter().any(|(_, v)| v.pid == pid) {
        scheduler.processes.remove(&pid);
        scheduler.pid_gen.free(pid);
        scheduler.current_pid = None;
    }
}
