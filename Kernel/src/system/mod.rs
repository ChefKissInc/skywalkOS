// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

pub mod allocator;
pub mod exc;
pub mod gdt;
mod panic;
pub mod pmm;
pub mod proc;
#[cfg(debug_assertions)]
pub mod serial;
pub mod state;
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
