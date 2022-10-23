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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum KernelRequest<'a> {
    Print(&'a [u8]),
    Exit,
}

impl<'a> KernelRequest<'a> {
    pub fn send(&self) {
        unsafe {
            core::arch::asm!("int 249", in("rdi") self as *const _);
        }
    }
}
