// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

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
        trace!("Tracking allocation of {size} bytes at {addr:#X} from process {proc_id}");
        self.allocations.insert(addr, (proc_id, size));
    }

    pub fn track_msg(&mut self, id: u64, addr: u64) {
        trace!("Marking allocation at {addr:#X} as message {id}");
        self.message_allocations.insert(id, addr);
    }

    pub fn free_msg(&mut self, id: u64) {
        trace!("Freeing message {id}");
        let addr = self.message_allocations.remove(&id).unwrap();
        self.free(addr);
    }

    pub fn free(&mut self, addr: u64) {
        let (proc_id, size) = self.allocations.remove(&addr).unwrap();
        let count = (size + 0xFFF) / 0x1000;
        trace!("Freeing allocation of {size} bytes at {addr:#X} from process {proc_id}");
        let state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
        unsafe {
            state.pmm.get_mut().unwrap().lock().free(
                (addr - tungsten_kit::USER_PHYS_VIRT_OFFSET) as *mut _,
                count,
            );
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
        let state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
        let addr = unsafe {
            state
                .pmm
                .get_mut()
                .unwrap()
                .lock()
                .alloc((size + 0xFFF) / 0x1000)
                .unwrap() as u64
        };
        let virt = addr + tungsten_kit::USER_PHYS_VIRT_OFFSET;
        self.track(proc_id, virt, size);
        virt
    }
}
