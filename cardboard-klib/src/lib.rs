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
pub enum KernelRequest {
    Print(&'static [u8]),
    Exit,
}
