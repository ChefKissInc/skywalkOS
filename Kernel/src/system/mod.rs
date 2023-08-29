// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

pub mod allocator;
pub mod exceptions;
pub mod fkext;
pub mod gdt;
mod panic;
pub mod pmm;
pub mod serial;
pub mod state;
pub mod tasking;
pub mod terminal;
pub mod tss;
pub mod vmm;

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct RegisterState {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub int_num: u64,
    pub err_code: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

impl core::fmt::Display for RegisterState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "R15: {:#018X}, R14: {:#018X}, R13: {:#018X}, R12: {:#018X}",
            self.r15, self.r14, self.r13, self.r12
        )?;
        writeln!(
            f,
            "R11: {:#018X}, R10: {:#018X}, R9:  {:#018X}, R8:  {:#018X}",
            self.r11, self.r10, self.r9, self.r8
        )?;
        writeln!(
            f,
            "RBP: {:#018X}, RDI: {:#018X}, RSI: {:#018X}, RDX: {:#018X}",
            self.rbp, self.rdi, self.rsi, self.rdx
        )?;
        writeln!(
            f,
            "RCX: {:#018X}, RBX: {:#018X}, RAX: {:#018X}, RIP: {:#018X}",
            self.rcx, self.rbx, self.rax, self.rip
        )?;
        writeln!(
            f,
            "CS:  {:#018X}, RFL: {:#018X}, RSP: {:#018X}, SS:  {:#018X}",
            self.cs, self.rflags, self.rsp, self.ss
        )?;
        write!(
            f,
            "INT: {:#018X}, ERR: {:#018X}",
            self.int_num, self.err_code
        )
    }
}
