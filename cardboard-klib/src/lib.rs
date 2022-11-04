#![no_std]
#![deny(warnings, clippy::cargo, unused_extern_crates)]

use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, IntoPrimitive)]
#[must_use]
#[repr(u64)]
pub enum KernelRequestStatus {
    Success,
    InvalidRequest,
    MalformedData,
    UnknownRequest,
    Unimplemented,
    Failure,
    #[num_enum(catch_all)]
    Other(u64),
}

impl KernelRequestStatus {
    /// # Panics
    ///
    /// Panics if self not Success
    pub fn unwrap(self) -> u64 {
        match self {
            Self::Success => 0,
            Self::Other(v) => v,
            v => panic!(
                "called `KernelRequestStatusCode::unwrap()` on an `{:#X?}` value",
                v
            ),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(C)]
pub enum MessageChannelEntry<'a> {
    Occupied {
        source_process: uuid::Uuid,
        data: &'a [u8],
    },
    Unoccupied,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(C)]
pub struct MessageChannel<'a> {
    pub data: [MessageChannelEntry<'a>; 64],
}

impl<'a> MessageChannel<'a> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            data: [MessageChannelEntry::Unoccupied; 64],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum KernelRequest<'a> {
    Print(&'a [u8]),
    AcquireMsgChannelRef,
    Exit,
    ScheduleNext,
}

impl<'a> KernelRequest<'a> {
    pub fn send(&self) -> KernelRequestStatus {
        let mut ret: u64;
        unsafe { core::arch::asm!("int 249", in("rdi") self as *const _, out("rax") ret) }
        ret.into()
    }
}
