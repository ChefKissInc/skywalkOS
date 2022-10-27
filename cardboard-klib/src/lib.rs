#![no_std]
#![deny(
    warnings,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    unused_extern_crates,
    rust_2021_compatibility
)]
#![allow(clippy::module_name_repetitions, clippy::similar_names)]

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
    #[allow(clippy::must_use_candidate)]
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
pub enum MessageChannelEntry {
    Occupied(uuid::Uuid, u64),
    Free,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(C)]
pub struct MessageChannel {
    pub data: [MessageChannelEntry; 64],
}

impl MessageChannel {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            data: [MessageChannelEntry::Free; 64],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum KernelRequest<'a> {
    Print(&'a [u8]),
    GetMyMessageChannel,
    Exit,
    SkipMe,
}

impl<'a> KernelRequest<'a> {
    pub fn send(&self) -> KernelRequestStatus {
        let mut ret: u64;
        unsafe { core::arch::asm!("int 249", in("rdi") self as *const _, out("rax") ret) }
        ret.into()
    }
}
