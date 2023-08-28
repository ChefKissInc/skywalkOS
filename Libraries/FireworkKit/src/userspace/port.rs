// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use crate::syscall::{AccessSize, SystemCall};

macro_rules! PortIOSystemCallIn {
    ($out:tt, $port:expr, $size:expr) => {{
        let mut val: Self;
        core::arch::asm!(
            "int 249",
            in("rdi") SystemCall::PortIn as u64,
            in("rsi") $port,
            in("rdx") $size as u64,
            out($out) val,
            options(nomem, nostack),
        );
        val
    }};
}

macro_rules! PortIOSystemCallOut {
    ($in_:tt, $port:expr, $value:expr, $size:expr) => {
        core::arch::asm!(
            "int 249",
            in("rdi") SystemCall::PortOut as u64,
            in("rsi") $port,
            in("rdx") $size as u64,
            in($in_) $value,
            options(nomem, nostack),
        )
    };
}

pub trait PortIO: Sized {
    unsafe fn read(port: u16) -> Self;
    unsafe fn write(port: u16, value: Self);
}

impl PortIO for u8 {
    unsafe fn read(port: u16) -> Self {
        PortIOSystemCallIn!("al", port, AccessSize::Byte)
    }

    unsafe fn write(port: u16, value: Self) {
        PortIOSystemCallOut!("cl", port, value, AccessSize::Byte);
    }
}

impl PortIO for u16 {
    unsafe fn read(port: u16) -> Self {
        PortIOSystemCallIn!("ax", port, AccessSize::Word)
    }

    unsafe fn write(port: u16, value: Self) {
        PortIOSystemCallOut!("cx", port, value, AccessSize::Word);
    }
}

impl PortIO for u32 {
    unsafe fn read(port: u16) -> Self {
        PortIOSystemCallIn!("eax", port, AccessSize::DWord)
    }

    unsafe fn write(port: u16, value: Self) {
        PortIOSystemCallOut!("ecx", port, value, AccessSize::DWord);
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
