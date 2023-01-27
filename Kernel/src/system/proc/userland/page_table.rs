// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

use alloc::boxed::Box;

use amd64::paging::pml4::PML4;

// Track allocations in order to free them on process exit.
#[repr(transparent)]
pub struct UserPML4(amd64::paging::PageTable);

impl UserPML4 {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(amd64::paging::PageTable::new())
    }
}

impl PML4 for UserPML4 {
    const VIRT_OFF: u64 = amd64::paging::PHYS_VIRT_OFFSET;

    fn get_entry(&mut self, offset: u64) -> &mut amd64::paging::PageTableEntry {
        &mut self.0.entries[offset as usize]
    }

    fn alloc_entry(&self) -> u64 {
        let phys = Box::leak(Box::new(amd64::paging::PageTable::new())) as *mut _ as u64
            - amd64::paging::PHYS_VIRT_OFFSET;
        let state = unsafe { crate::system::state::SYS_STATE.get().as_mut().unwrap() };
        let scheduler = state.scheduler.get_mut().unwrap().get_mut();
        state.user_allocations.get_mut().unwrap().lock().track(
            scheduler.current_thread_id.unwrap(),
            phys + iridium_kit::USER_PHYS_VIRT_OFFSET,
            4096,
        );
        phys
    }
}
