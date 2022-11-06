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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum KernelRequest<'a> {
    Print(&'a [u8]),
    AcquireMsgChannelRef,
    SendMessage(uuid::Uuid, &'a [u8]),
    Exit,
    ScheduleNext,
}

impl<'a> KernelRequest<'a> {
    /// # Safety
    ///
    /// The caller must ensure that this operation has no unsafe side effects.
    pub unsafe fn send(&self) -> KernelRequestStatus {
        let mut ret: u64;
        core::arch::asm!("int 249", in("rdi") self as *const _, out("rax") ret);
        ret.into()
    }
}
