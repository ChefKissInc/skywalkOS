// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::boxed::Box;

use amd64::paging::{PageTable, PageTableFlags, PAGE_SIZE};

use crate::system::proc::AllocationType;

#[derive(Debug)]
#[repr(C)]
pub struct UserPML4(PageTable<{ amd64::paging::PHYS_VIRT_OFFSET }>, u64);

impl UserPML4 {
    #[inline]
    pub const fn new(pid: u64) -> Self {
        Self(amd64::paging::PageTable::new(), pid)
    }

    fn alloc_entry(pid: u64) -> u64 {
        let sys_state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
        let phys = Box::leak(Box::new(PageTable::<0>::new())) as *mut _ as u64
            - amd64::paging::PHYS_VIRT_OFFSET;

        let scheduler = sys_state.scheduler.as_mut().unwrap().get_mut();
        scheduler.processes.get_mut(&pid).unwrap().track_alloc(
            phys + fireworkkit::USER_PHYS_VIRT_OFFSET,
            PAGE_SIZE,
            AllocationType::Kernel,
        );

        phys
    }

    pub unsafe fn set_cr3(&mut self) {
        self.0.set_cr3();
    }

    pub unsafe fn map(&mut self, virt: u64, phys: u64, count: u64, flags: PageTableFlags) {
        let pid = self.1;
        self.0
            .map(&move || Self::alloc_entry(pid), virt, phys, count, flags);
    }

    pub unsafe fn unmap(&mut self, virt: u64, count: u64) {
        self.0.unmap(virt, count);
    }

    pub unsafe fn map_higher_half(&mut self) {
        let pid = self.1;
        self.0.map_higher_half(&move || Self::alloc_entry(pid));
    }
}
