// Copyright (c) ChefKiss 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#![allow(dead_code)]

pub mod regs;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum AddressSpaceID {
    SystemMemory = 0,
    SystemIo,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Address {
    pub addr_space_id: AddressSpaceID,
    pub reg_bit_width: u8,
    pub reg_bit_off: u8,
    __: u8,
    pub address: u64,
}

#[bitfield(u32)]
pub struct EventTimerBlockID {
    pub hw_revision: u8,
    #[bits(5)]
    pub comparator_cnt: u8,
    pub counter_size: bool,
    __: bool,
    pub legacy_replacement: bool,
    pub pci_vendor_id: u16,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
/// 4 bits
pub enum PageProtection {
    NoGuarantee = 0,
    Protected4Kb,
    Protected64Kb,
}

impl PageProtection {
    const fn into_bits(self) -> u8 {
        self as _
    }

    const fn from_bits(value: u8) -> Self {
        match value {
            0 => Self::NoGuarantee,
            1 => Self::Protected4Kb,
            2 => Self::Protected64Kb,
            _ => panic!("Invalid value for PageProtection"),
        }
    }
}

#[bitfield(u8)]
pub struct PageProtectionAttributes {
    #[bits(4)]
    pub page_protection: PageProtection,
    #[bits(4)]
    __: u8,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Hpet {
    header: super::SystemDescTableHeader,
    pub evnt_timer_block: EventTimerBlockID,
    pub address: Address,
    pub hpet_num: u8,
    pub min_tick: u16,
    pub page_prot_attr: PageProtectionAttributes,
}

impl Hpet {
    pub fn read_reg<T: Into<u64>>(&self, reg: T) -> u64 {
        unsafe {
            ((self.address.address + amd64::paging::PHYS_VIRT_OFFSET + reg.into()) as *const u64)
                .read_volatile()
        }
    }

    pub fn write_reg<T: Into<u64>>(&self, reg: T, value: u64) {
        unsafe {
            ((self.address.address + amd64::paging::PHYS_VIRT_OFFSET + reg.into()) as *mut u64)
                .write_volatile(value);
        }
    }

    pub fn counter_value(&self) -> u64 {
        self.read_reg(regs::HPETReg::MainCounterValue)
    }

    pub fn set_counter_value(&self, value: u64) {
        self.write_reg(regs::HPETReg::MainCounterValue, value);
    }

    pub fn capabilities(&self) -> regs::GeneralCapabilities {
        self.read_reg(regs::HPETReg::GeneralCapabilities).into()
    }

    pub fn set_config(&self, value: regs::GeneralConfiguration) {
        self.write_reg(regs::HPETReg::GeneralConfiguration, value.into());
    }
}

impl core::ops::Deref for Hpet {
    type Target = super::SystemDescTableHeader;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}
