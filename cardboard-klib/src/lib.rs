// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#![no_std]
#![deny(warnings, clippy::cargo, unused_extern_crates)]

pub mod port;

use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

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
    pub fn as_result(self) -> Result<(), SystemCallStatus> {
        match self {
            Self::Success => Ok(()),
            _ => Err(self),
        }
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
    PortInByte,
    PortInWord,
    PortInDWord,
    PortOutByte,
    PortOutWord,
    PortOutDWord,
    RegisterIRQHandler,
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
    pub unsafe fn receive_message() -> Result<Option<(uuid::Uuid, &'static [u8])>, SystemCallStatus>
    {
        let ty: u64 = Self::ReceiveMessage.into();
        let mut ret: u64;
        let mut id_upper: u64;
        let mut id_lower: u64;
        let mut ptr: u64;
        let mut len: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            out("rax") ret,
            lateout("rdi") id_upper,
            out("rsi") id_lower,
            out("rdx") ptr,
            out("rcx") len,
        );
        let ret = SystemCallStatus::try_from(ret).unwrap();
        if matches!(ret, SystemCallStatus::DoNothing) {
            Ok(None)
        } else {
            ret.as_result()?;
            Ok(Some((
                uuid::Uuid::from_u64_pair(id_upper, id_lower),
                core::slice::from_raw_parts(ptr as *const u8, len as usize),
            )))
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn send_message(target: uuid::Uuid, s: &[u8]) -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::SendMessage.into();
        let (id_upper, id_lower) = target.as_u64_pair();
        let ptr = s.as_ptr() as u64;
        let len = s.len() as u64;
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") id_upper,
            in("rdx") id_lower,
            in("rcx") ptr,
            in("r8") len,
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
    pub unsafe fn skip() -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::Skip.into();
        let mut ret: u64;
        core::arch::asm!("int 249", in("rdi") ty, out("rax") ret);
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn register_provider(provider: uuid::Uuid) -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::RegisterProvider.into();
        let (id_upper, id_lower) = provider.as_u64_pair();
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") id_upper,
            in("rdx") id_lower,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn get_providing_process(
        provider: uuid::Uuid,
    ) -> Result<uuid::Uuid, SystemCallStatus> {
        let ty: u64 = Self::GetProvidingProcess.into();
        let (mut id_upper, mut id_lower) = provider.as_u64_pair();
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") id_upper,
            in("rdx") id_lower,
            lateout("rdi") id_upper,
            lateout("rsi") id_lower,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()?;
        Ok(uuid::Uuid::from_u64_pair(id_upper, id_lower))
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn port_in_byte(port: u16) -> Result<u8, SystemCallStatus> {
        let ty: u64 = Self::PortInByte.into();
        let mut ret: u64;
        let mut val: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") port as u64,
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
        let ty: u64 = Self::PortOutByte.into();
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") port as u64,
            in("rdx") val as u64,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn port_in_word(port: u16) -> Result<u16, SystemCallStatus> {
        let ty: u64 = Self::PortInWord.into();
        let mut ret: u64;
        let mut val: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") port as u64,
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
        let ty: u64 = Self::PortOutWord.into();
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") port as u64,
            in("rdx") val as u64,
            out("rax") ret,
        );
        SystemCallStatus::try_from(ret).unwrap().as_result()
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn port_in_dword(port: u16) -> Result<u32, SystemCallStatus> {
        let ty: u64 = Self::PortInDWord.into();
        let mut ret: u64;
        let mut val: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") port as u64,
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
        let ty: u64 = Self::PortOutDWord.into();
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") port as u64,
            in("rdx") val as u64,
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
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub enum KernelMessage {
    IRQFired(u8),
}
