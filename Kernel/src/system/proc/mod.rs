// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::{
    boxed::Box,
    collections::VecDeque,
    string::{String, ToString},
    vec::Vec,
};

use tungsten_kit::syscall::Message;

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
                    + tungsten_kit::USER_PHYS_VIRT_OFFSET
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
    pub path: String,
    pub cwd: String,
    pub cr3: Box<userland::page_table::UserPML4>,
    pub messages: VecDeque<Message>,
}

impl Process {
    #[inline]
    #[must_use]
    pub fn new(proc_id: u64, path: &str, cwd: &str) -> Self {
        Self {
            path: path.to_string(),
            cwd: cwd.to_string(),
            cr3: Box::new(userland::page_table::UserPML4::new(proc_id)),
            messages: VecDeque::new(),
        }
    }
}
