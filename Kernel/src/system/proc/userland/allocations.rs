// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::vec::Vec;

use hashbrown::HashMap;

pub struct UserAllocationTracker {
    pub allocations: HashMap<u64, (u64, u64)>,
    pub message_allocations: HashMap<u64, u64>,
}

impl UserAllocationTracker {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            message_allocations: HashMap::new(),
        }
    }

    pub fn track(&mut self, proc_id: u64, addr: u64, size: u64) {
        trace!(
            "Tracking allocation of {} bytes at {:#X} from process {}",
            size,
            addr,
            proc_id
        );
        self.allocations.insert(addr, (proc_id, size));
    }

    pub fn track_msg(&mut self, id: u64, addr: u64) {
        trace!("Marking allocation at {:#X} as message {}", addr, id);
        self.message_allocations.insert(id, addr);
    }

    pub fn free_msg(&mut self, id: u64) {
        trace!("Freeing message {}", id);
        let addr = self.message_allocations.remove(&id).unwrap();
        self.free(addr);
    }

    pub fn free(&mut self, addr: u64) {
        let (proc_id, size) = self.allocations.remove(&addr).unwrap();
        let count = (size + 0xFFF) / 0x1000;
        trace!(
            "Freeing allocation of {} pages at {:#X} from process {}",
            count,
            addr,
            proc_id
        );
        let state = unsafe { crate::system::state::SYS_STATE.get().as_mut().unwrap() };
        unsafe {
            state
                .pmm
                .get_mut()
                .unwrap()
                .lock()
                .free((addr - driver_core::USER_PHYS_VIRT_OFFSET) as *mut _, count);
        }
    }

    pub fn free_proc(&mut self, proc_id: u64) {
        for addr in self
            .allocations
            .iter()
            .filter(|(_, (p, _))| *p == proc_id)
            .map(|(k, _)| *k)
            .collect::<Vec<_>>()
        {
            self.free(addr);
        }
    }

    #[must_use]
    pub fn allocate(&mut self, proc_id: u64, size: u64) -> u64 {
        let state = unsafe { crate::system::state::SYS_STATE.get().as_mut().unwrap() };
        let addr = unsafe {
            state
                .pmm
                .get_mut()
                .unwrap()
                .lock()
                .alloc((size + 0xFFF) / 0x1000)
                .unwrap() as u64
        };
        let virt = addr + driver_core::USER_PHYS_VIRT_OFFSET;
        self.track(proc_id, virt, size);
        virt
    }
}