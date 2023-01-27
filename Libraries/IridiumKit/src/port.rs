// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

pub trait PortIO: Sized {
    unsafe fn read(port: u16) -> Self;
    unsafe fn write(port: u16, value: Self);
}

impl PortIO for u8 {
    unsafe fn read(port: u16) -> Self {
        super::syscall::SystemCall::port_in_byte(port).unwrap()
    }

    unsafe fn write(port: u16, value: Self) {
        super::syscall::SystemCall::port_out_byte(port, value).unwrap();
    }
}

impl PortIO for u16 {
    unsafe fn read(port: u16) -> Self {
        super::syscall::SystemCall::port_in_word(port).unwrap()
    }

    unsafe fn write(port: u16, value: Self) {
        super::syscall::SystemCall::port_out_word(port, value).unwrap();
    }
}

impl PortIO for u32 {
    unsafe fn read(port: u16) -> Self {
        super::syscall::SystemCall::port_in_dword(port).unwrap()
    }

    unsafe fn write(port: u16, value: Self) {
        super::syscall::SystemCall::port_out_dword(port, value).unwrap();
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
