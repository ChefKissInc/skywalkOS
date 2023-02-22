// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use tungstenkit::syscall::OSDTEntryReqType;

use crate::system::{proc::scheduler::Scheduler, RegisterState};

pub fn get_entry_info(scheduler: &mut Scheduler, state: &mut RegisterState) {
    let sys_state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
    let dt_index = sys_state.dt_index.as_ref().unwrap().read();
    let Ok(info_type) = OSDTEntryReqType::try_from(state.rdx) else {
        todo!()
    };
    let Some(ent) = dt_index.get(&state.rsi) else {
        todo!()
    };
    let data = match info_type {
        OSDTEntryReqType::Parent => postcard::to_allocvec(&ent.lock().parent),
        OSDTEntryReqType::Children => postcard::to_allocvec(&ent.lock().children),
        OSDTEntryReqType::Properties => postcard::to_allocvec(&ent.lock().properties),
        OSDTEntryReqType::Property => {
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

    let virt = scheduler
        .current_process_mut()
        .unwrap()
        .track_kernelside_alloc(data.as_ptr() as _, data.len() as _);

    state.rdi = virt;
    state.rsi = data.len() as u64;
}
