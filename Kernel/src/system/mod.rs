// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

pub mod allocator;
pub mod exc;
pub mod gdt;
mod panic;
pub mod pmm;
pub mod proc;
pub mod serial;
pub mod state;
pub mod terminal;
pub mod tkext;
pub mod tss;
pub mod vmm;

#[derive(Default, Clone, Copy)]
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
            "R15: 0x{:>016X}, R14: 0x{:>016X}, R13: 0x{:>016X}, R12: 0x{:>016X}",
            self.r15, self.r14, self.r13, self.r12
        )?;
        writeln!(
            f,
            "R11: 0x{:>016X}, R10: 0x{:>016X}, R9:  0x{:>016X}, R8:  0x{:>016X}",
            self.r11, self.r10, self.r9, self.r8
        )?;
        writeln!(
            f,
            "RBP: 0x{:>016X}, RDI: 0x{:>016X}, RSI: 0x{:>016X}, RDX: 0x{:>016X}",
            self.rbp, self.rdi, self.rsi, self.rdx
        )?;
        writeln!(
            f,
            "RCX: 0x{:>016X}, RBX: 0x{:>016X}, RAX: 0x{:>016X}",
            self.rcx, self.rbx, self.rax
        )?;
        writeln!(
            f,
            "INT: 0x{:>016X}, ERR: 0x{:>016X}, RIP: 0x{:>016X}",
            self.int_num, self.err_code, self.rip
        )?;
        writeln!(
            f,
            "CS:  0x{:>016X}, RFL: 0x{:>016X}, RSP: 0x{:>016X}",
            self.cs, self.rflags, self.rsp
        )?;
        write!(f, "SS:  0x{:>016X}", self.ss)
    }
}
