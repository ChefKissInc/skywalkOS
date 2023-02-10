// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::boxed::Box;
use core::mem::size_of;

use amd64::paging::pml4::PML4;

#[repr(C)]
pub struct UserPML4(amd64::paging::PageTable, u64);

impl UserPML4 {
    #[inline]
    pub const fn new(pid: u64) -> Self {
        Self(amd64::paging::PageTable::new(), pid)
    }
}

impl PML4 for UserPML4 {
    const VIRT_OFF: u64 = amd64::paging::PHYS_VIRT_OFFSET;

    fn get_entry(&mut self, offset: u64) -> &mut amd64::paging::PageTableEntry {
        &mut self.0.entries[offset as usize]
    }

    fn alloc_entry(&self) -> u64 {
        let phys = Box::leak(Box::new(Self::new(self.1))) as *mut _ as u64
            - amd64::paging::PHYS_VIRT_OFFSET;

        let scheduler = unsafe {
            (*crate::system::state::SYS_STATE.get())
                .scheduler
                .as_mut()
                .unwrap()
                .get_mut()
        };
        scheduler.processes.get_mut(&self.1).unwrap().track_alloc(
            phys + tungstenkit::USER_PHYS_VIRT_OFFSET,
            size_of::<Self>() as _,
            None,
        );
        phys
    }
}
