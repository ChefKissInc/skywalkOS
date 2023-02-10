// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use crate::system::{proc::scheduler::Scheduler, RegisterState};

pub fn get_entry_info(scheduler: &mut Scheduler, state: &mut RegisterState) {
    let pid = scheduler.current_pid.unwrap();
    let sys_state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
    let dt_index = sys_state.dt_index.as_ref().unwrap().read();
    let Ok(info_type) = tungstenkit::syscall::OSDTEntryReqType::try_from(state.rdx) else {
        todo!()
    };
    let Some(ent) = dt_index.get(&state.rsi) else {
        todo!()
    };
    let data = match info_type {
        tungstenkit::syscall::OSDTEntryReqType::Parent => postcard::to_allocvec(&ent.lock().parent),
        tungstenkit::syscall::OSDTEntryReqType::Children => {
            postcard::to_allocvec(&ent.lock().children)
        }
        tungstenkit::syscall::OSDTEntryReqType::Properties => {
            postcard::to_allocvec(&ent.lock().properties)
        }
        tungstenkit::syscall::OSDTEntryReqType::Property => {
            let Ok(k) = core::str::from_utf8(unsafe {
                core::slice::from_raw_parts(state.rcx as *const _, state.r8 as _)
            }) else {
                todo!()
            };
            postcard::to_allocvec(&ent.lock().properties.get(k))
        }
    }
    .unwrap()
    .leak();

    let ptr = data.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET;
    let virt = ptr + tungstenkit::USER_PHYS_VIRT_OFFSET;

    scheduler
        .processes
        .get_mut(&pid)
        .unwrap()
        .track_alloc(virt, data.len() as u64, Some(false));

    state.rdi = virt;
    state.rsi = data.len() as u64;
}
