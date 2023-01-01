// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use modular_bitfield::prelude::*;
use num_enum::IntoPrimitive;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct IOAPIC {
    header: super::ICHeader,
    pub id: u8,
    __: u8,
    pub address: u32,
    pub gsi_base: u32,
}

impl core::ops::Deref for IOAPIC {
    type Target = super::ICHeader;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}

#[derive(Debug, IntoPrimitive)]
#[repr(u32)]
pub enum IOAPICReg {
    ID,
    Ver,
    ArbID,
    IORedirTable = 0x10,
}

#[derive(Debug, BitfieldSpecifier)]
#[bits = 3]
#[repr(u8)]
pub enum DeliveryMode {
    Fixed,
    LowestPriority,
    SMI,
    NMI = 4,
    Init,
    ExtINT = 7,
}

#[bitfield(bits = 64)]
#[derive(Debug)]
#[repr(u64)]
pub struct IOAPICRedir {
    pub vector: u8,
    pub delivery_mode: DeliveryMode,
    pub logical_dest: bool,
    pub pending: bool,
    pub active_high: bool,
    pub remote_irr: bool,
    pub trigger_at_level: bool,
    pub masked: bool,
    #[skip]
    __: B39,
    pub dest: u8,
}

#[bitfield(bits = 32)]
#[derive(Debug)]
#[repr(u32)]
pub struct IOAPICVer {
    #[skip(setters)]
    pub ver: u8,
    #[skip]
    __: u8,
    #[skip(setters)]
    pub max_redir: u8,
    #[skip]
    __: u8,
}

impl IOAPIC {
    #[must_use]
    const fn base(&self, off: u8) -> *mut u32 {
        (self.address as u64 + off as u64 + amd64::paging::PHYS_VIRT_OFFSET) as *mut u32
    }

    #[must_use]
    pub fn read<T: Into<u32>>(&self, reg: T) -> u32 {
        unsafe {
            self.base(0).write_volatile(reg.into());
            self.base(0x10).read_volatile()
        }
    }

    #[must_use]
    pub fn read_redir(&self, num: u32) -> IOAPICRedir {
        let reg = IOAPICReg::IORedirTable as u32 + num * 2;
        IOAPICRedir::from(u64::from(self.read(reg)) | (u64::from(self.read(reg + 1)) << 32))
    }

    #[must_use]
    pub fn read_ver(&self) -> IOAPICVer {
        IOAPICVer::from(self.read(IOAPICReg::Ver))
    }

    pub fn write(&self, reg: u32, val: u32) {
        unsafe {
            self.base(0).write_volatile(reg);
            self.base(0x10).write_volatile(val);
        }
    }

    pub fn write_redir(&self, num: u32, redir: IOAPICRedir) {
        let reg = IOAPICReg::IORedirTable as u32 + num * 2;
        let val = u64::from(redir);
        self.write(reg, val as u32);
        self.write(reg + 1, (val >> 32) as u32);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct InterruptSourceOverride {
    header: super::ICHeader,
    pub bus: u8,
    pub irq: u8,
    pub gsi: u32,
    pub flags: amd64::spec::mps::Inti,
}

impl core::ops::Deref for InterruptSourceOverride {
    type Target = super::ICHeader;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct NMISource {
    header: super::ICHeader,
    pub flags: amd64::spec::mps::Inti,
    pub gsi: u32,
}

impl core::ops::Deref for NMISource {
    type Target = super::ICHeader;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}
