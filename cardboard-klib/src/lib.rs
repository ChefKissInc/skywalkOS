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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, FromPrimitive, IntoPrimitive)]
#[must_use]
#[repr(u64)]
pub enum KernelRequestStatusCode {
    Success = 0,
    InvalidRequest,
    MalformedData,
    UnknownRequest,
    Unimplemented,
    #[num_enum(default)]
    Failure = !0u64,
}

impl KernelRequestStatusCode {
    /// # Panics
    ///
    /// Panics if self not Success
    pub fn unwrap(self) {
        assert!(
            self == Self::Success,
            "called unwrap on an error value: {:?}",
            self
        );
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
#[repr(C)]
pub enum Message<T> {
    Some(T),
    None,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
#[repr(C)]
pub struct MessageChannel<T> {
    pub data: [Message<T>; 20],
}

impl<T: Copy> MessageChannel<T> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            data: [Message::None; 20],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum KernelRequest<'a> {
    Print(&'a [u8]),
    RegisterMessageChannel(u64),
    Exit,
    SkipMe,
}

impl<'a> KernelRequest<'a> {
    pub fn send(&self) -> KernelRequestStatusCode {
        let mut ret: u64;
        unsafe {
            core::arch::asm!("int 249", in("rdi") self as *const _, lateout("rax") ret);
        }
        ret.into()
    }
}
