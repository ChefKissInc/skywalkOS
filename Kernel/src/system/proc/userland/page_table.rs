// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::boxed::Box;
use core::mem::size_of;

#[repr(C)]
pub struct UserPML4(amd64::paging::PageTable, u64);

impl UserPML4 {
    const VIRT_OFF: u64 = amd64::paging::PHYS_VIRT_OFFSET;

    #[inline]
    pub const fn new(pid: u64) -> Self {
        Self(amd64::paging::PageTable::new(), pid)
    }

    fn alloc_entry(pid: u64) -> u64 {
        let phys = Box::leak(Box::new(amd64::paging::PageTable::new())) as *mut _ as u64
            - amd64::paging::PHYS_VIRT_OFFSET;

        let scheduler = unsafe {
            (*crate::system::state::SYS_STATE.get())
                .scheduler
                .as_ref()
                .unwrap()
                .get_mut()
        };
        scheduler.processes.get_mut(&pid).unwrap().track_alloc(
            phys + fireworkkit::USER_PHYS_VIRT_OFFSET,
            size_of::<amd64::paging::PageTable>() as _,
            None,
        );
        phys
    }

    pub unsafe fn set(&mut self) {
        self.0.set(Self::VIRT_OFF);
    }

    pub unsafe fn map_pages(
        &mut self,
        virt: u64,
        phys: u64,
        count: u64,
        flags: amd64::paging::PageTableEntry,
    ) {
        let pid = self.1;
        self.0.map_pages(
            &move || Self::alloc_entry(pid),
            virt,
            Self::VIRT_OFF,
            phys,
            count,
            flags,
        );
    }

    pub unsafe fn unmap_pages(&mut self, virt: u64, count: u64) {
        self.0.unmap_pages(virt, Self::VIRT_OFF, count);
    }

    pub unsafe fn map_higher_half(&mut self) {
        let pid = self.1;
        self.0
            .map_higher_half(&move || Self::alloc_entry(pid), Self::VIRT_OFF);
    }
}
