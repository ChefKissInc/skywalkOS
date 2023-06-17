// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Message {
    pub id: u64,
    pub pid: u64,
    pub data: &'static [u8],
}

impl Message {
    #[inline]
    #[must_use]
    pub const fn new(id: u64, pid: u64, data: &'static [u8]) -> Self {
        Self { id, pid, data }
    }

    #[must_use]
    pub unsafe fn receive() -> Self {
        let (mut id, mut pid): (u64, u64);
        let (mut ptr, mut len): (u64, u64);
        core::arch::asm!(
            "int 249",
            in("rdi") SystemCall::ReceiveMessage as u64,
            out("rax") id,
            lateout("rdi") pid,
            out("rsi") ptr,
            out("rdx") len,
            options(nostack, preserves_flags),
        );
        Self {
            id,
            pid,
            data: core::slice::from_raw_parts(ptr as *const u8, len as _),
        }
    }

    pub unsafe fn send(self) {
        core::arch::asm!(
            "int 249",
            in("rdi") SystemCall::SendMessage as u64,
            in("rsi") self.pid,
            in("rdx") self.data.as_ptr() as u64,
            in("rcx") self.data.len() as u64,
            options(nostack, preserves_flags),
        );
    }

    pub unsafe fn ack(self) {
        core::arch::asm!(
            "int 249",
            in("rdi") SystemCall::AckMessage as u64,
            in("rsi") self.id,
            options(nostack, preserves_flags),
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u64)]
pub enum AccessSize {
    Byte,
    Word,
    DWord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u64)]
pub enum SystemCall {
    KPrint,
    ReceiveMessage,
    SendMessage,
    Quit,
    Yield,
    PortIn,
    PortOut,
    RegisterIRQHandler,
    Allocate,
    Free,
    AckMessage,
    NewOSDTEntry,
    GetOSDTEntryInfo,
    SetOSDTEntryProp,
}

#[cfg(feature = "userspace")]
impl SystemCall {
    pub unsafe fn kprint(s: &str) {
        core::arch::asm!(
            "int 249",
            in("rdi") Self::KPrint as u64,
            in("rsi") s.as_ptr() as u64,
            in("rdx") s.len() as u64,
            options(nostack, preserves_flags),
        );
    }

    pub unsafe fn quit() -> ! {
        core::arch::asm!("int 249", in("rdi") Self::Quit as u64, options(nostack, preserves_flags, noreturn));
    }

    pub unsafe fn r#yield() {
        core::arch::asm!("int 249", in("rdi") Self::Yield as u64, options(nostack, preserves_flags));
    }

    pub unsafe fn register_irq_handler(irq: u8) {
        core::arch::asm!(
            "int 249",
            in("rdi") Self::RegisterIRQHandler as u64,
            in("sil") irq,
            options(nostack, preserves_flags),
        );
    }

    #[must_use]
    pub unsafe fn allocate(size: u64) -> *mut u8 {
        let mut ptr: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::Allocate as u64,
            in("rsi") size,
            out("rax") ptr,
            options(nostack, preserves_flags),
        );
        ptr as *mut u8
    }

    pub unsafe fn free(ptr: *mut u8) {
        core::arch::asm!(
            "int 249",
            in("rdi") Self::Free as u64,
            in("rsi") ptr as u64,
            options(nostack, preserves_flags),
        );
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub enum KernelMessage {
    IRQFired(u8),
}
