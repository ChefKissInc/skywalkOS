// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::vec::Vec;

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

    pub unsafe fn receive() -> Self {
        let (mut id, mut pid): (u64, u64);
        let (mut ptr, mut len): (u64, u64);
        core::arch::asm!(
            "int 249",
            in("rdi") SystemCall::ReceiveMessage as u64,
            lateout("rdi") id,
            out("rsi") pid,
            out("rdx") ptr,
            out("rcx") len,
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
pub enum SystemCall {
    KPrint,
    ReceiveMessage,
    SendMessage,
    Exit,
    Yield,
    RegisterProvider,
    GetProvidingProcess,
    PortIn,
    PortOut,
    RegisterIRQHandler,
    Allocate,
    Free,
    AckMessage,
    GetDTEntryInfo,
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
pub enum OSDTEntryReqType {
    Parent,
    Children,
    Properties,
    Property,
}

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

    pub unsafe fn exit() -> ! {
        core::arch::asm!("int 249", in("rdi") Self::Exit as u64, options(nostack, preserves_flags, noreturn));
    }

    pub unsafe fn skip() {
        core::arch::asm!("int 249", in("rdi") Self::Yield as u64, options(nostack, preserves_flags));
    }

    pub unsafe fn register_provider(provider: u64) {
        core::arch::asm!(
            "int 249",
            in("rdi") Self::RegisterProvider as u64,
            in("rsi") provider,
            options(nostack, preserves_flags),
        );
    }

    pub unsafe fn get_providing_process(provider: u64) -> Option<u64> {
        let mut id: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::GetProvidingProcess as u64,
            in("rsi") provider,
            out("rax") id,
            options(nostack, preserves_flags),
        );
        if id == 0 {
            None
        } else {
            Some(id)
        }
    }

    pub unsafe fn port_in_byte(port: u16) -> u8 {
        let mut val: u8;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortIn as u64,
            in("rsi") port,
            in("rdx") AccessSize::Byte as u64,
            lateout("dil") val, // TODO: switch to rax
            options(nostack, preserves_flags),
        );
        val
    }

    pub unsafe fn port_out_byte(port: u16, val: u8) {
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortOut as u64,
            in("rsi") port,
            in("dl") val,
            in("rcx") AccessSize::Byte as u64,
            options(nostack, preserves_flags),
        );
    }

    pub unsafe fn port_in_word(port: u16) -> u16 {
        let mut val: u16;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortIn as u64,
            in("rsi") port,
            in("rdx") AccessSize::Word as u64,
            lateout("rdi") val, // TODO: switch to rax
            options(nostack, preserves_flags),
        );
        val
    }

    pub unsafe fn port_out_word(port: u16, val: u16) {
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortOut as u64,
            in("rsi") port,
            in("rdx") val,
            in("rcx") AccessSize::Word as u64,
            options(nostack, preserves_flags),
        );
    }

    pub unsafe fn port_in_dword(port: u16) -> u32 {
        let mut val: u32;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortIn as u64,
            in("rsi") port,
            in("rdx") AccessSize::DWord as u64,
            lateout("edi") val, // TODO: switch to rax
            options(nostack, preserves_flags),
        );
        val
    }

    pub unsafe fn port_out_dword(port: u16, val: u32) {
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortOut as u64,
            in("rsi") port,
            in("rdx") val,
            in("rcx") AccessSize::DWord as u64,
            options(nostack, preserves_flags),
        );
    }

    pub unsafe fn register_irq_handler(irq: u8) {
        core::arch::asm!(
            "int 249",
            in("rdi") Self::RegisterIRQHandler as u64,
            in("sil") irq,
            options(nostack, preserves_flags),
        );
    }

    pub unsafe fn allocate(size: u64) -> *mut u8 {
        let mut ptr: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::Allocate as u64,
            in("rsi") size,
            lateout("rdi") ptr, // TODO: switch to rax
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

    pub unsafe fn get_dt_entry_info(id: u64, ty: OSDTEntryReqType, k: Option<&str>) -> Vec<u8> {
        let (mut ptr, mut len): (u64, u64);
        core::arch::asm!(
            "int 249",
            in("rdi") Self::GetDTEntryInfo as u64,
            in("rsi") id,
            in("rdx") ty as u64,
            in("rcx") k.map_or(0, |s| s.as_ptr() as u64),
            in("r8") k.map_or(0, |s| s.len() as u64),
            lateout("rdi") ptr, // TODO: switch to rax
            lateout("rsi") len, // TODO: switch to rdi
            options(nostack, preserves_flags),
        );
        Vec::from_raw_parts(ptr as *mut u8, len as _, len as _)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub enum KernelMessage {
    IRQFired(u8),
}
