// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use modular_bitfield::prelude::*;

#[bitfield(bits = 32)]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub struct ProcessorLAPICFlags {
    #[skip(setters)]
    pub enabled: bool,
    #[skip(setters)]
    pub online_capable: bool,
    #[skip]
    __: B30,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ProcessorLocalAPIC {
    header: super::ICHeader,
    pub acpi_uid: u8,
    pub apic_id: u8,
    pub flags: ProcessorLAPICFlags,
}

impl core::ops::Deref for ProcessorLocalAPIC {
    type Target = super::ICHeader;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct LocalAPICAddrOverride {
    header: super::ICHeader,
    __: u16,
    pub addr: u64,
}

impl core::ops::Deref for LocalAPICAddrOverride {
    type Target = super::ICHeader;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct LocalAPICNMI {
    header: super::ICHeader,
    pub acpi_proc_id: u8,
    pub flags: amd64::spec::mps::Inti,
    pub lint: u8,
}

impl core::ops::Deref for LocalAPICNMI {
    type Target = super::ICHeader;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}
