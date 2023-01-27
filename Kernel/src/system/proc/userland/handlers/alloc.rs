// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

use amd64::paging::{pml4::PML4, PageTableEntry};
use iridium_kit::syscall::SystemCallStatus;

use crate::system::{proc::scheduler::Scheduler, RegisterState};

pub fn alloc(scheduler: &mut Scheduler, state: &mut RegisterState) -> SystemCallStatus {
    let size = state.rsi;
    let proc_id = scheduler.current_thread_mut().unwrap().proc_id;

    let sys_state = unsafe { crate::system::state::SYS_STATE.get().as_mut().unwrap() };
    let addr = sys_state
        .user_allocations
        .get_mut()
        .unwrap()
        .lock()
        .allocate(proc_id, size);

    unsafe {
        let process = scheduler.processes.get_mut(&proc_id).unwrap();
        process.cr3.map_pages(
            addr,
            addr - iridium_kit::USER_PHYS_VIRT_OFFSET,
            (size + 0xFFF) / 0x1000,
            PageTableEntry::new()
                .with_writable(true)
                .with_present(true)
                .with_user(true),
        );

        core::ptr::write_bytes(addr as *mut u8, 0, ((size + 0xFFF) / 0x1000 * 0x1000) as _);
    }

    state.rdi = addr;
    SystemCallStatus::Success
}

pub fn free(scheduler: &mut Scheduler, state: &mut RegisterState) -> SystemCallStatus {
    let addr = state.rsi;
    let sys_state = unsafe { crate::system::state::SYS_STATE.get().as_mut().unwrap() };
    let size = sys_state
        .user_allocations
        .get_mut()
        .unwrap()
        .lock()
        .allocations
        .get(&addr)
        .unwrap()
        .1;
    sys_state
        .user_allocations
        .get_mut()
        .unwrap()
        .lock()
        .free(addr);

    let proc_id = scheduler.current_thread_mut().unwrap().proc_id;
    let process = scheduler.processes.get_mut(&proc_id).unwrap();
    unsafe {
        process.cr3.unmap_pages(addr, (size + 0xFFF) / 0x1000);
    }

    SystemCallStatus::Success
}
