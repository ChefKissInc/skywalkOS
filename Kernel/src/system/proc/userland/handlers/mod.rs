// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use core::fmt::Write;

use iridium_kit::syscall::SystemCallStatus;

use crate::system::{proc::scheduler::Scheduler, RegisterState};

pub mod alloc;
pub mod device_tree;
pub mod message;
pub mod port;
pub mod provider;

pub fn kprint(state: &mut RegisterState) -> SystemCallStatus {
    let s = unsafe { core::slice::from_raw_parts(state.rsi as *const u8, state.rdx as usize) };
    let s = unsafe { core::str::from_utf8_unchecked(s) };

    #[cfg(debug_assertions)]
    write!(crate::system::serial::SERIAL.lock(), "{s}").unwrap();

    let sys_state = unsafe { crate::system::state::SYS_STATE.get().as_mut().unwrap() };
    if let Some(terminal) = &mut sys_state.terminal {
        write!(terminal, "{s}").unwrap();
    }

    SystemCallStatus::Success
}

pub fn process_teardown(scheduler: &mut Scheduler) {
    let id = scheduler.current_thread_id.unwrap();
    let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
    let index = scheduler.thread_ids.iter().position(|v| *v == id).unwrap();
    scheduler.threads.remove(&id);
    scheduler.thread_ids.remove(index);
    scheduler.thread_id_gen.free(id);
    scheduler.current_thread_id = None;

    let sys_state = unsafe { crate::system::state::SYS_STATE.get().as_mut().unwrap() };
    if !scheduler.threads.iter().any(|(_, v)| v.proc_id == proc_id) {
        sys_state
            .user_allocations
            .get_mut()
            .unwrap()
            .lock()
            .free_proc(proc_id);
        scheduler.processes.remove(&proc_id);
        scheduler.proc_id_gen.free(proc_id);
    }
}
