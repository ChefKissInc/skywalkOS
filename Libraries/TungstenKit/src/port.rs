// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use num_enum::TryFromPrimitive;

use crate::syscall::SystemCall;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u64)]
pub enum AccessSize {
    Byte,
    Word,
    DWord,
}

pub trait PortIO: Sized {
    unsafe fn read(port: u16) -> Self;
    unsafe fn write(port: u16, value: Self);
}

impl PortIO for u8 {
    unsafe fn read(port: u16) -> Self {
        let mut val: Self;
        core::arch::asm!(
            "int 249",
            in("rdi") SystemCall::PortIn as u64,
            in("rsi") port,
            in("rdx") AccessSize::Byte as u64,
            out("al") val,
            options(nostack, preserves_flags),
        );
        val
    }

    unsafe fn write(port: u16, value: Self) {
        core::arch::asm!(
            "int 249",
            in("rdi") SystemCall::PortOut as u64,
            in("rsi") port,
            in("dl") value,
            in("rcx") AccessSize::Byte as u64,
            options(nostack, preserves_flags),
        );
    }
}

impl PortIO for u16 {
    unsafe fn read(port: u16) -> Self {
        let mut val: Self;
        core::arch::asm!(
            "int 249",
            in("rdi") SystemCall::PortIn as u64,
            in("rsi") port,
            in("rdx") AccessSize::Word as u64,
            out("rax") val,
            options(nostack, preserves_flags),
        );
        val
    }

    unsafe fn write(port: u16, value: Self) {
        core::arch::asm!(
            "int 249",
            in("rdi") SystemCall::PortOut as u64,
            in("rsi") port,
            in("rdx") value,
            in("rax") AccessSize::Word as u64,
            options(nostack, preserves_flags),
        );
    }
}

impl PortIO for u32 {
    unsafe fn read(port: u16) -> Self {
        let mut val: Self;
        core::arch::asm!(
            "int 249",
            in("rdi") SystemCall::PortIn as u64,
            in("rsi") port,
            in("rdx") AccessSize::DWord as u64,
            out("rax") val,
            options(nostack, preserves_flags),
        );
        val
    }

    unsafe fn write(port: u16, value: Self) {
        core::arch::asm!(
            "int 249",
            in("rdi") SystemCall::PortOut as u64,
            in("rsi") port,
            in("rdx") value,
            in("rcx") AccessSize::DWord as u64,
            options(nostack, preserves_flags),
        );
    }
}

#[derive(Clone, Copy)]
pub struct Port<T: PortIO, R: From<T> + Into<T>> {
    port: u16,
    __: core::marker::PhantomData<T>,
    ___: core::marker::PhantomData<R>,
}

impl<T: PortIO, R: From<T> + Into<T>> Port<T, R> {
    #[inline]
    #[must_use]
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            __: core::marker::PhantomData,
            ___: core::marker::PhantomData,
        }
    }

    #[must_use]
    pub unsafe fn read(&self) -> R {
        T::read(self.port).into()
    }

    #[must_use]
    pub unsafe fn read_off<A: Into<u16>, R2: From<T> + Into<T>>(&self, off: A) -> R2 {
        T::read(self.port + off.into()).into()
    }

    pub unsafe fn write(&self, value: R) {
        T::write(self.port, value.into());
    }

    pub unsafe fn write_off<A: Into<u16>, R2: From<T> + Into<T>>(&self, value: R2, off: A) {
        T::write(self.port + off.into(), value.into());
    }
}
