// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use amd64::paging::{pml4::PML4, PageTableEntry};
use tungstenkit::syscall::SystemCallStatus;

use crate::system::{proc::scheduler::Scheduler, RegisterState};

pub fn get_entry_info(scheduler: &mut Scheduler, state: &mut RegisterState) -> SystemCallStatus {
    let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
    let sys_state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
    let dt_index = sys_state.dt_index.as_ref().unwrap().lock();
    let Some(dt_entry) = dt_index.get(&state.rsi) else {
        return SystemCallStatus::MalformedData;
    };
    let Ok(info_type) = tungstenkit::syscall::OSDTEntryInfoType::try_from(state.rdx) else {
        return SystemCallStatus::MalformedData;
    };
    let data = match info_type {
        tungstenkit::syscall::OSDTEntryInfoType::Parent => postcard::to_allocvec(&dt_entry.parent),
        tungstenkit::syscall::OSDTEntryInfoType::PropertyNamed => {
            let Ok(k) = core::str::from_utf8(unsafe {
                core::slice::from_raw_parts(state.rcx as *const u8, state.r8 as usize)
            }) else {
                return SystemCallStatus::MalformedData;
            };
            postcard::to_allocvec(&dt_entry.properties.get(k))
        }
    }
    .unwrap()
    .leak();
    let ptr = data.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET;
    let virt = ptr + tungstenkit::USER_PHYS_VIRT_OFFSET;
    let count = (data.len() as u64 + 0xFFF) / 0x1000;
    let mut usr_allocs = sys_state.usr_allocs.as_ref().unwrap().lock();
    usr_allocs.track(proc_id, virt, data.len() as u64);

    unsafe {
        let process = scheduler.processes.get_mut(&proc_id).unwrap();
        process.cr3.map_pages(
            virt,
            ptr,
            count,
            PageTableEntry::new().with_present(true).with_user(true),
        );
    }
    state.rdi = virt;
    state.rsi = data.len() as u64;
    SystemCallStatus::Success
}
