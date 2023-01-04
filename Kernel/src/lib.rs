// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#![no_std]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]

pub mod port;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

pub const USER_PHYS_VIRT_OFFSET: u64 = 0xC0000000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[must_use]
#[repr(u64)]
pub enum SystemCallStatus {
    Success,
    InvalidRequest,
    MalformedData,
    UnknownRequest,
    Unimplemented,
    Failure,
    DoNothing,
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
    #[must_use]
    pub const fn new(id: u64, proc_id: u64, data: &'static [u8]) -> Self {
        Self { id, proc_id, data }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u64)]
pub enum SystemCall {
    KPrint,
    ReceiveMessage,
    SendMessage,
    Exit,
    Skip,
    RegisterProvider,
    GetProvidingProcess,
    PortIn,
    PortOut,
    RegisterIRQHandler,
    Allocate,
    Free,
    Ack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u64)]
pub enum AccessSize {
    Byte,
    Word,
    DWord,
}

impl SystemCall {
    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn kprint(s: &str) -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::KPrint.into();
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") s.as_ptr() as u64,
            in("rdx") s.len() as u64,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn receive_message() -> Result<Option<Message>, SystemCallStatus> {
        let ty: u64 = Self::ReceiveMessage.into();
        let mut ret: u64;
        let mut id: u64;
        let mut proc_id: u64;
        let mut ptr: u64;
        let mut len: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            out("rax") ret,
            lateout("rdi") id,
            out("rsi") proc_id,
            out("rdx") ptr,
            out("rcx") len,
        );
        let ret = SystemCallStatus::try_from(ret).unwrap();
        if matches!(ret, SystemCallStatus::DoNothing) {
            Ok(None)
        } else {
            ret.as_result()?;
            Ok(Some(Message {
                id,
                proc_id,
                data: core::slice::from_raw_parts(ptr as *const u8, len as usize),
            }))
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn send_message(target: u64, s: &[u8]) -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::SendMessage.into();
        let ptr = s.as_ptr() as u64;
        let len = s.len() as u64;
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") target,
            in("rdx") ptr,
            in("rcx") len,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn exit() -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::Exit.into();
        let mut ret: u64;
        core::arch::asm!("int 249", in("rdi") ty, out("rax") ret);
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn skip() {
        let ty: u64 = Self::Skip.into();
        core::arch::asm!("int 249", in("rdi") ty);
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn register_provider(provider: u64) -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::RegisterProvider.into();
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") provider,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn get_providing_process(provider: u64) -> Result<u64, SystemCallStatus> {
        let ty: u64 = Self::GetProvidingProcess.into();
        let mut id;
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") provider,
            lateout("rdi") id,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()?;
        Ok(id)
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn port_in_byte(port: u16) -> Result<u8, SystemCallStatus> {
        let mut ret: u64;
        let mut val: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortIn as u64,
            in("rsi") port as u64,
            in("rdx") AccessSize::Byte as u64,
            lateout("rdi") val,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()?;
        Ok(val as u8)
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn port_out_byte(port: u16, val: u8) -> Result<(), SystemCallStatus> {
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortOut as u64,
            in("rsi") port as u64,
            in("rdx") val as u64,
            in("rcx") AccessSize::Byte as u64,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn port_in_word(port: u16) -> Result<u16, SystemCallStatus> {
        let mut ret: u64;
        let mut val: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortIn as u64,
            in("rsi") port as u64,
            in("rdx") AccessSize::Word as u64,
            lateout("rdi") val,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()?;
        Ok(val as u16)
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn port_out_word(port: u16, val: u16) -> Result<(), SystemCallStatus> {
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortOut as u64,
            in("rsi") port as u64,
            in("rdx") val as u64,
            in("rcx") AccessSize::Word as u64,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn port_in_dword(port: u16) -> Result<u32, SystemCallStatus> {
        let mut ret: u64;
        let mut val: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortIn as u64,
            in("rsi") port as u64,
            in("rdx") AccessSize::DWord as u64,
            lateout("rdi") val,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()?;
        Ok(val as u32)
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn port_out_dword(port: u16, val: u32) -> Result<(), SystemCallStatus> {
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") Self::PortOut as u64,
            in("rsi") port as u64,
            in("rdx") val as u64,
            in("rcx") AccessSize::DWord as u64,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn register_irq_handler(irq: u8) -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::RegisterIRQHandler.into();
        let mut ret: u64;
        core::arch::asm!("int 249", in("rdi") ty, in("rsi") irq as u64, out("rax") ret);
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn allocate(size: u64) -> Result<*mut u8, SystemCallStatus> {
        let ty: u64 = Self::Allocate.into();
        let mut ret: u64;
        let mut ptr: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") size,
            lateout("rdi") ptr,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()?;
        Ok(ptr as *mut u8)
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn free(ptr: *mut u8) -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::Free.into();
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") ptr as u64,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn ack(id: u64) -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::Ack.into();
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") id,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub enum KernelMessage {
    IRQFired(u8),
}
