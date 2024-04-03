// Copyright (c) ChefKiss Inc 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#[bitfield(u32)]
pub struct ProcessorLAPICFlags {
    pub enabled: bool,
    pub online_capable: bool,
    #[bits(30)]
    __: u32,
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
    pub flags: amd64::spec::mps::INTI,
    pub lint: u8,
}

impl core::ops::Deref for LocalAPICNMI {
    type Target = super::ICHeader;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}
