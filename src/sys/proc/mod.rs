//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use alloc::{
    boxed::Box,
    collections::VecDeque,
    string::{String, ToString},
    vec::Vec,
};

use amd64::paging::pml4::PML4;

use super::vmm::PageTableLvl4;

pub mod sched;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ThreadState {
    Active,
    Blocked,
    Inactive,
}

#[derive(Debug)]
pub struct Thread {
    pub state: ThreadState,
    pub id: usize,
    pub regs: super::RegisterState,
    pub fs_base: usize,
    pub gs_base: usize,
    pub rsp: Vec<u8>,
    pub kern_rsp: Vec<u8>,
}

impl Thread {
    pub fn new(id: usize, rip: usize) -> Self {
        let mut rsp = Vec::new();
        rsp.resize(0x2000, 0);
        let mut kern_rsp = Vec::new();
        kern_rsp.resize(0x2000, 0);
        Self {
            state: ThreadState::Inactive,
            id,
            regs: super::RegisterState {
                rip: rip as u64,
                cs: 0x08,
                rflags: 0x202,
                rsp: rsp.as_ptr() as u64 + rsp.len() as u64,
                ss: 0x10,
                ..Default::default()
            },
            fs_base: 0,
            gs_base: 0,
            rsp,
            kern_rsp,
        }
    }
}

#[derive(Debug)]
pub struct Process {
    pub id: usize,
    pub path: String,
    pub cwd: String,
    pub cr3: Box<PageTableLvl4>,
    pub threads: VecDeque<Thread>,
}

impl Process {
    pub fn new(id: usize, path: &str, cwd: &str) -> Self {
        let mut cr3 = Box::new(PageTableLvl4::new());
        unsafe {
            cr3.map_higher_half();
        }

        Self {
            id,
            path: path.to_string(),
            cwd: cwd.to_string(),
            cr3,
            threads: VecDeque::new(),
        }
    }
}
