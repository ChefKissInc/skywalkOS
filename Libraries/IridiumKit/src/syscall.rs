// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::vec::Vec;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u64)]
pub enum SystemCallStatus {
    Success,
    InvalidRequest,
    MalformedData,
    UnknownRequest,
    NotFound,
}

impl SystemCallStatus {
    pub const fn as_result(self) -> Result<(), Self> {
        match self {
            Self::Success => Ok(()),
            _ => Err(self),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Message {
    pub id: u64,
    pub proc_id: u64,
    pub data: &'static [u8],
}

impl Message {
    #[inline]
    #[must_use]
    pub const fn new(id: u64, proc_id: u64, data: &'static [u8]) -> Self {
        Self { id, proc_id, data }
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
    GetProviderForProcess,
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
pub enum OSDTEntryInfoType {
    Parent,
    PropertyNamed,
}

impl SystemCall {
    pub unsafe fn kprint(s: &str) -> Result<(), SystemCallStatus> {
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::KPrint as u64,
            in("rsi") s.as_ptr() as u64,
            in("rdx") s.len() as u64,
            out("rax") ret,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    pub unsafe fn receive_message() -> Result<Option<Message>, SystemCallStatus> {
        let mut ret: u64;
        let mut id: u64;
        let mut proc_id: u64;
        let mut ptr: u64;
        let mut len: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::ReceiveMessage as u64,
            out("rax") ret,
            lateout("rdi") id,
            out("rsi") proc_id,
            out("rdx") ptr,
            out("rcx") len,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()?;
        if id == 0 {
            return Ok(None);
        }
        Ok(Some(Message {
            id,
            proc_id,
            data: core::slice::from_raw_parts(ptr as *const u8, len as usize),
        }))
    }

    pub unsafe fn send_message(target: u64, s: &[u8]) -> Result<(), SystemCallStatus> {
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::SendMessage as u64,
            in("rsi") target,
            in("rdx") s.as_ptr() as u64,
            in("rcx") s.len() as u64,
            out("rax") ret,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    pub unsafe fn exit() -> ! {
        core::arch::asm!("int 249", in("rdi") Self::Exit as u64, options(nostack, preserves_flags, noreturn));
    }

    pub unsafe fn skip() {
        core::arch::asm!("int 249", in("rdi") Self::Yield as u64, options(nostack, preserves_flags));
    }

    pub unsafe fn register_provider(provider: u64) -> Result<(), SystemCallStatus> {
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::RegisterProvider as u64,
            in("rsi") provider,
            out("rax") ret,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    pub unsafe fn get_providing_process(provider: u64) -> Result<Option<u64>, SystemCallStatus> {
        let (mut ret, mut id): (u64, u64);
        core::arch::asm!(
            "int 249",
            in("rdi") Self::GetProviderForProcess as u64,
            in("rsi") provider,
            out("rax") ret,
            lateout("rdi") id,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()?;
        if id == 0 {
            return Ok(None);
        }
        Ok(Some(id))
    }

    pub unsafe fn port_in_byte(port: u16) -> Result<u8, SystemCallStatus> {
        let (mut ret, mut val): (u64, u8);
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortIn as u64,
            in("rsi") port,
            in("rdx") AccessSize::Byte as u64,
            out("rax") ret,
            lateout("dil") val,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()?;
        Ok(val)
    }

    pub unsafe fn port_out_byte(port: u16, val: u8) -> Result<(), SystemCallStatus> {
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortOut as u64,
            in("rsi") port,
            in("dl") val,
            in("rcx") AccessSize::Byte as u64,
            out("rax") ret,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    pub unsafe fn port_in_word(port: u16) -> Result<u16, SystemCallStatus> {
        let (mut ret, mut val): (u64, u16);
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortIn as u64,
            in("rsi") port,
            in("rdx") AccessSize::Word as u64,
            out("rax") ret,
            lateout("rdi") val,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()?;
        Ok(val)
    }

    pub unsafe fn port_out_word(port: u16, val: u16) -> Result<(), SystemCallStatus> {
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortOut as u64,
            in("rsi") port,
            in("rdx") val,
            in("rcx") AccessSize::Word as u64,
            out("rax") ret,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    pub unsafe fn port_in_dword(port: u16) -> Result<u32, SystemCallStatus> {
        let (mut ret, mut val): (u64, u32);
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortIn as u64,
            in("rsi") port,
            in("rdx") AccessSize::DWord as u64,
            out("rax") ret,
            lateout("edi") val,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()?;
        Ok(val)
    }

    pub unsafe fn port_out_dword(port: u16, val: u32) -> Result<(), SystemCallStatus> {
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortOut as u64,
            in("rsi") port,
            in("rdx") val,
            in("rcx") AccessSize::DWord as u64,
            out("rax") ret,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    pub unsafe fn register_irq_handler(irq: u8) -> Result<(), SystemCallStatus> {
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::RegisterIRQHandler as u64,
            in("sil") irq,
            out("rax") ret,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    pub unsafe fn allocate(size: u64) -> Result<*mut u8, SystemCallStatus> {
        let (mut ret, mut ptr): (u64, u64);
        core::arch::asm!(
            "int 249",
            in("rdi") Self::Allocate as u64,
            in("rsi") size,
            out("rax") ret,
            lateout("rdi") ptr,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()?;
        Ok(ptr as *mut u8)
    }

    pub unsafe fn free(ptr: *mut u8) -> Result<(), SystemCallStatus> {
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::Free as u64,
            in("rsi") ptr as u64,
            out("rax") ret,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    pub unsafe fn ack_message(id: u64) -> Result<(), SystemCallStatus> {
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::AckMessage as u64,
            in("rsi") id,
            out("rax") ret,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    pub unsafe fn get_dt_entry_info(
        id: u64,
        ty: OSDTEntryInfoType,
        k: Option<&str>,
    ) -> Result<Vec<u8>, SystemCallStatus> {
        let mut ret: u64;
        let mut ptr: u64;
        let mut len: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::GetDTEntryInfo as u64,
            in("rsi") id,
            in("rdx") ty as u64,
            in("rcx") k.map_or(0, |s| s.as_ptr() as u64),
            in("r8") k.map_or(0, |s| s.len() as u64),
            out("rax") ret,
            lateout("rdi") ptr,
            lateout("rsi") len,
            options(nostack, preserves_flags),
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()?;
        Ok(Vec::from_raw_parts(ptr as *mut u8, len as _, len as _))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub enum KernelMessage {
    IRQFired(u8),
}
