#![no_std]
#![deny(warnings, clippy::cargo, unused_extern_crates)]

pub mod port;

use num_enum::{FromPrimitive, IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[must_use]
#[repr(u64)]
pub enum SystemCallStatus {
    InvalidRequest = 1,
    MalformedData,
    UnknownRequest,
    Unimplemented,
    Failure,
    DoNothing,
    #[num_enum(catch_all)]
    Other(u64),
}

impl SystemCallStatus {
    /// # Panics
    ///
    /// Panics if self not Success
    pub fn unwrap(self) -> u64 {
        match self {
            Self::Other(v) => v,
            v => panic!("called `SystemCallStatus::unwrap()` on an `{v:#X?}` value"),
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

        if ret == 0 {
            return Ok(());
        }

        Err(SystemCallStatus::from(ret))
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn receive_message() -> Result<Option<(uuid::Uuid, &'static [u8])>, SystemCallStatus>
    {
        let ty: u64 = Self::ReceiveMessage.into();
        let mut ret: u64;
        let mut uuid_hi: u64;
        let mut uuid_lo: u64;
        let mut ptr: u64;
        let mut len: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            out("rax") ret,
            lateout("rdi") uuid_hi,
            out("rsi") uuid_lo,
            out("rdx") ptr,
            out("rcx") len,
        );

        match SystemCallStatus::from(ret) {
            SystemCallStatus::DoNothing => Ok(None),
            SystemCallStatus::Other(0) => Ok(Some((
                uuid::Uuid::from_u64_pair(uuid_hi, uuid_lo),
                core::slice::from_raw_parts(ptr as *const u8, len as usize),
            ))),
            v => Err(v),
        }
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn send_message(target: uuid::Uuid, s: &[u8]) -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::SendMessage.into();
        let (uuid_hi, uuid_lo) = target.as_u64_pair();
        let ptr = s.as_ptr() as u64;
        let len = s.len() as u64;
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") uuid_hi,
            in("rdx") uuid_lo,
            in("rcx") ptr,
            in("r8") len,
            out("rax") ret,
        );

        if ret == 0 {
            return Ok(());
        }

        Err(ret.into())
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn exit() -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::Exit.into();
        let mut ret: u64;
        core::arch::asm!("int 249", in("rdi") ty, out("rax") ret);
        if ret == 0 {
            return Ok(());
        }

        Err(ret.into())
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn skip() -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::Skip.into();
        let mut ret: u64;
        core::arch::asm!("int 249", in("rdi") ty, out("rax") ret);
        if ret == 0 {
            return Ok(());
        }

        Err(ret.into())
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn register_provider(provider: uuid::Uuid) -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::RegisterProvider.into();
        let (uuid_hi, uuid_lo) = provider.as_u64_pair();
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") uuid_hi,
            in("rdx") uuid_lo,
            out("rax") ret,
        );

        if ret == 0 {
            return Ok(());
        }

        Err(ret.into())
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn get_providing_process(
        provider: uuid::Uuid,
    ) -> Result<uuid::Uuid, SystemCallStatus> {
        let ty: u64 = Self::GetProvidingProcess.into();
        let (mut uuid_hi, mut uuid_lo) = provider.as_u64_pair();
        let mut ret: u64;
        core::arch::asm!(
            "int 249",
            in("rdi") ty,
            in("rsi") uuid_hi,
            in("rdx") uuid_lo,
            lateout("rdi") uuid_hi,
            lateout("rsi") uuid_lo,
            out("rax") ret,
        );

        if ret == 0 {
            return Ok(uuid::Uuid::from_u64_pair(uuid_hi, uuid_lo));
        }

        Err(ret.into())
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

        if ret == 0 {
            return Ok(val as u8);
        }

        Err(ret.into())
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

        if ret == 0 {
            return Ok(());
        }

        Err(ret.into())
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

        if ret == 0 {
            return Ok(val as u16);
        }

        Err(ret.into())
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

        if ret == 0 {
            return Ok(());
        }

        Err(ret.into())
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

        if ret == 0 {
            return Ok(val as u32);
        }

        Err(ret.into())
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

        if ret == 0 {
            return Ok(());
        }

        Err(ret.into())
    }

    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn register_irq_handler(irq: u8) -> Result<(), SystemCallStatus> {
        let ty: u64 = Self::RegisterIRQHandler.into();
        let mut ret: u64;
        core::arch::asm!("int 249", in("rdi") ty, in("rsi") irq as u64, out("rax") ret);

        if ret == 0 {
            return Ok(());
        }

        Err(ret.into())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(C)]
pub enum KernelMessage {
    IRQFired(u8),
}
