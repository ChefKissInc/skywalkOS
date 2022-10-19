// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};

use amd64::paging::pml4::PML4;

use super::vmm::PageTableLvl4;

pub mod sched;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    Active,
    Blocked,
    Inactive,
}

#[derive(Debug)]
pub struct Thread {
    pub state: ThreadState,
    pub uuid: uuid::Uuid,
    pub proc_uuid: uuid::Uuid,
    pub regs: super::RegisterState,
    pub fs_base: usize,
    pub gs_base: usize,
    pub stack: Vec<u8>,
    pub kern_stack: Vec<u8>,
}

impl Thread {
    pub fn new(proc_uuid: uuid::Uuid, rip: u64) -> Self {
        let mut stack = Vec::new();
        stack.resize(0x14000, 0);
        let mut kern_stack = Vec::new();
        kern_stack.resize(0x14000, 0);
        Self {
            state: ThreadState::Inactive,
            uuid: uuid::Uuid::new_v4(),
            proc_uuid,
            regs: super::RegisterState {
                rip,
                cs: super::gdt::SegmentSelector::new(3, super::gdt::PrivilegeLevel::User)
                    .0
                    .into(),
                rflags: 0x202,
                rsp: stack.as_ptr() as u64 - amd64::paging::PHYS_VIRT_OFFSET + stack.len() as u64,
                ss: super::gdt::SegmentSelector::new(4, super::gdt::PrivilegeLevel::User)
                    .0
                    .into(),
                ..Default::default()
            },
            fs_base: 0,
            gs_base: 0,
            stack,
            kern_stack,
        }
    }
}

#[derive(Debug)]
pub struct Process {
    pub path: String,
    pub cwd: String,
    pub cr3: Box<PageTableLvl4>,
}

impl Process {
    pub fn new(path: &str, cwd: &str) -> Self {
        let mut cr3 = Box::new(PageTableLvl4::new());
        unsafe {
            cr3.map_higher_half();
        }

        Self {
            path: path.to_string(),
            cwd: cwd.to_string(),
            cr3,
        }
    }
}
