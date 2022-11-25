// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

// use alloc::vec::Vec;

use amd64::paging::{pml4::PML4, PageTableEntry};
use hashbrown::HashMap;

pub struct UserAllocationTracker {
    pub allocations: HashMap<u64, (uuid::Uuid, u64)>,
    // message_allocations: HashMap<uuid::Uuid, u64>,
}

impl UserAllocationTracker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            // message_allocations: HashMap::new(),
        }
    }

    pub fn track(&mut self, proc_id: uuid::Uuid, addr: u64, count: u64) {
        trace!(
            "Tracking allocation of {} pages at {:#X} from process {}",
            count,
            addr,
            proc_id
        );
        self.allocations.insert(addr, (proc_id, count));
    }

    pub fn free(&mut self, addr: u64) {
        let (proc_id, count) = self.allocations.remove(&addr).unwrap();
        trace!(
            "Freeing allocation of {} pages at {:#X} from process {}",
            count,
            addr,
            proc_id
        );
        let state = unsafe { crate::sys::state::SYS_STATE.get().as_mut().unwrap() };
        unsafe {
            state
                .pmm
                .get_mut()
                .unwrap()
                .lock()
                .free((addr - super::USER_PHYS_VIRT_OFFSET) as *mut _, count)
        }
    }

    // pub fn free_proc(&mut self, proc_id: uuid::Uuid) {
    //     for addr in self
    //         .allocations
    //         .iter()
    //         .filter(|(_, (p, _))| *p == proc_id)
    //         .map(|(k, _)| *k)
    //         .collect::<Vec<_>>()
    //     {
    //         self.free(addr);
    //     }
    // }

    #[must_use]
    pub fn allocate(
        &mut self,
        proc_id: uuid::Uuid,
        cr3: &mut super::UserPageTableLvl4,
        size: u64,
    ) -> u64 {
        let state = unsafe { crate::sys::state::SYS_STATE.get().as_mut().unwrap() };
        let count = (size + 0xFFF) / 0x1000;
        let addr = unsafe { state.pmm.get_mut().unwrap().lock().alloc(count).unwrap() as u64 };
        let virt = addr + super::USER_PHYS_VIRT_OFFSET;
        self.track(proc_id, virt, count);
        unsafe {
            cr3.map_pages(
                virt,
                addr,
                count,
                PageTableEntry::new()
                    .with_user(true)
                    .with_writable(true)
                    .with_present(true),
            );
        }
        virt
    }
}
