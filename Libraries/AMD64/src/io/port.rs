// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

pub trait PortIO: Sized {
    unsafe fn read(port: u16) -> Self;
    unsafe fn write(port: u16, value: Self);
}

impl PortIO for u8 {
    unsafe fn read(port: u16) -> Self {
        let ret: Self;
        core::arch::asm!("in al, dx", out("al") ret, in("dx") port, options(nostack, preserves_flags));
        ret
    }

    unsafe fn write(port: u16, value: Self) {
        core::arch::asm!("out dx, al", in("dx") port, in("al") value, options(nostack, preserves_flags));
    }
}

impl PortIO for u16 {
    unsafe fn read(port: u16) -> Self {
        let ret: Self;
        core::arch::asm!("in ax, dx", out("ax") ret, in("dx") port, options(nostack, preserves_flags));
        ret
    }

    unsafe fn write(port: u16, value: Self) {
        core::arch::asm!("out dx, ax", in("dx") port, in("ax") value, options(nostack, preserves_flags));
    }
}

impl PortIO for u32 {
    unsafe fn read(port: u16) -> Self {
        let ret: Self;
        core::arch::asm!("in eax, dx", out("eax") ret, in("dx") port, options(nostack, preserves_flags));
        ret
    }

    unsafe fn write(port: u16, value: Self) {
        core::arch::asm!("out dx, eax", in("dx") port, in("eax") value, options(nostack, preserves_flags));
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
    pub const fn new(port: u16) -> Self {
        Self {
            port,
            __: core::marker::PhantomData,
            ___: core::marker::PhantomData,
        }
    }

    pub unsafe fn read(&self) -> R {
        T::read(self.port).into()
    }

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
