// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::{
    boxed::Box,
    collections::VecDeque,
    string::{String, ToString},
    vec::Vec,
};

use driver_core::system_call::Message;

pub mod scheduler;
pub mod userland;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ThreadState {
    Active,
    Blocked,
    Inactive,
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
    #[must_use]
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
                    + userland::USER_PHYS_VIRT_OFFSET
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

#[derive(Debug)]
pub struct Process {
    pub path: String,
    pub cwd: String,
    pub cr3: Box<userland::UserPageTableLvl4>,
    pub messages: VecDeque<Message>,
}

impl Process {
    #[must_use]
    pub fn new(proc_id: u64, path: &str, cwd: &str) -> Self {
        Self {
            path: path.to_string(),
            cwd: cwd.to_string(),
            cr3: Box::new(userland::UserPageTableLvl4::new(proc_id)),
            messages: VecDeque::new(),
        }
    }
}
