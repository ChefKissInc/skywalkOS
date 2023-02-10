// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::{boxed::Box, collections::VecDeque, string::String, vec::Vec};

use amd64::paging::{pml4::PML4, PageTableEntry};
use hashbrown::HashMap;
use tungstenkit::syscall::Message;

pub mod scheduler;
pub mod userland;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ThreadState {
    Active,
    Inactive,
    Suspended,
}

#[derive(Debug)]
pub struct Thread {
    pub state: ThreadState,
    pub proc_id: u64,
    pub regs: super::RegisterState,
    pub fs_base: usize,
    pub gs_base: usize,
    pub stack: Vec<u8>,
}

impl Thread {
    #[inline]
    pub fn new(proc_id: u64, rip: u64) -> Self {
        let stack = vec![0; 0x14000];
        Self {
            state: ThreadState::Inactive,
            proc_id,
            regs: super::RegisterState {
                rip,
                cs: super::gdt::SegmentSelector::new(3, super::gdt::PrivilegeLevel::User)
                    .0
                    .into(),
                rflags: 0x202,
                rsp: stack.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET
                    + tungstenkit::USER_PHYS_VIRT_OFFSET
                    + stack.len() as u64,
                ss: super::gdt::SegmentSelector::new(4, super::gdt::PrivilegeLevel::User)
                    .0
                    .into(),
                ..Default::default()
            },
            fs_base: 0,
            gs_base: 0,
            stack,
        }
    }
}

pub struct Process {
    pub proc_id: u64,
    pub path: String,
    pub cr3: Box<userland::page_table::UserPML4>,
    pub messages: VecDeque<Message>,
    pub allocations: HashMap<u64, (u64, bool)>,
    pub message_allocations: HashMap<u64, u64>,
}

impl Process {
    #[inline]
    pub fn new(proc_id: u64, path: String) -> Self {
        Self {
            proc_id,
            path,
            cr3: Box::new(userland::page_table::UserPML4::new(proc_id)),
            messages: VecDeque::new(),
            allocations: HashMap::new(),
            message_allocations: HashMap::new(),
        }
    }

    pub fn track_alloc(&mut self, addr: u64, size: u64, writable: Option<bool>) {
        trace!(
            "Tracking allocation of {size} bytes at {addr:#X} from process {}",
            self.proc_id
        );
        self.allocations.insert(addr, (size, writable.is_some()));
        if let Some(writable) = writable {
            unsafe {
                self.cr3.map_pages(
                    addr,
                    addr - tungstenkit::USER_PHYS_VIRT_OFFSET,
                    (size + 0xFFF) / 0x1000,
                    PageTableEntry::new()
                        .with_present(true)
                        .with_writable(writable)
                        .with_user(true),
                );
            }
        }
    }

    pub fn free_alloc(&mut self, addr: u64) {
        let (size, mapped) = self.allocations.remove(&addr).unwrap();
        trace!(
            "Freeing allocation of {size} bytes at {addr:#X} from process {}",
            self.proc_id
        );

        let page_count = (size + 0xFFF) / 0x1000;

        unsafe {
            (*crate::system::state::SYS_STATE.get())
                .pmm
                .as_mut()
                .unwrap()
                .lock()
                .free(
                    (addr - tungstenkit::USER_PHYS_VIRT_OFFSET) as *mut _,
                    page_count,
                );
        }

        if mapped {
            unsafe { self.cr3.unmap_pages(addr, page_count) }
        }
    }

    pub fn track_msg(&mut self, id: u64, addr: u64) {
        trace!("Marking allocation at {addr:#X} as message {id}");
        self.message_allocations.insert(id, addr);
    }

    pub fn free_msg(&mut self, id: u64) {
        trace!("Freeing message {id}");
        let addr = self.message_allocations.remove(&id).unwrap();
        self.free_alloc(addr);
    }

    pub fn allocate(&mut self, size: u64) -> u64 {
        let state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
        let addr = unsafe {
            state
                .pmm
                .as_mut()
                .unwrap()
                .lock()
                .alloc((size + 0xFFF) / 0x1000)
                .unwrap() as u64
        };
        let virt = addr + tungstenkit::USER_PHYS_VIRT_OFFSET;
        self.track_alloc(virt, size, Some(true));
        virt
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        let addrs: Vec<_> = self.allocations.keys().cloned().collect();
        for addr in addrs {
            self.free_alloc(addr);
        }
    }
}
