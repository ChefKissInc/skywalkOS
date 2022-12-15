// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use modular_bitfield::prelude::*;

pub mod regs;

#[derive(Debug, BitfieldSpecifier, Clone, Copy)]
#[bits = 8]
#[repr(u8)]
pub enum AddressSpaceID {
    SystemMemory = 0,
    SystemIo,
}

#[bitfield(bits = 96)]
#[derive(Debug, Clone, Copy)]
pub struct Address {
    #[skip(setters)]
    pub addr_space_id: AddressSpaceID,
    #[skip(setters)]
    pub reg_bit_width: u8,
    #[skip(setters)]
    pub reg_bit_off: u8,
    #[skip]
    __: u8,
    #[skip(setters)]
    pub address: u64,
}

#[bitfield(bits = 32)]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub struct EventTimerBlockID {
    #[skip(setters)]
    pub hw_revision: u8,
    #[skip(setters)]
    pub comparator_cnt: B5,
    #[skip(setters)]
    pub counter_size: bool,
    #[skip]
    __: bool,
    #[skip(setters)]
    pub legacy_replacement: bool,
    #[skip(setters)]
    pub pci_vendor_id: u16,
}

#[derive(Debug, BitfieldSpecifier, Clone, Copy)]
#[bits = 4]
#[repr(u8)]
pub enum PageProtection {
    NoGuarantee = 0,
    Protected4Kb,
    Protected64Kb,
}

#[bitfield(bits = 8)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub struct PageProtectionAttributes {
    #[skip(setters)]
    pub page_protection: PageProtection,
    #[skip]
    __: B4,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct HPET {
    header: super::SDTHeader,
    pub evnt_timer_block: EventTimerBlockID,
    pub address: Address,
    pub hpet_num: u8,
    pub min_tick: u16,
    pub page_prot_attr: PageProtectionAttributes,
}

impl HPET {
    #[must_use]
    pub fn read_reg<T: Into<u64>>(&self, reg: T) -> u64 {
        unsafe {
            ((self.address.address() + amd64::paging::PHYS_VIRT_OFFSET + reg.into()) as *const u64)
                .read_volatile()
        }
    }

    pub fn write_reg<T: Into<u64>>(&self, reg: T, value: u64) {
        unsafe {
            ((self.address.address() + amd64::paging::PHYS_VIRT_OFFSET + reg.into()) as *mut u64)
                .write_volatile(value);
        }
    }

    #[must_use]
    pub fn counter_value(&self) -> u64 {
        self.read_reg(regs::HPETReg::MainCounterValue)
    }

    pub fn set_counter_value(&self, value: u64) {
        self.write_reg(regs::HPETReg::MainCounterValue, value);
    }

    #[must_use]
    pub fn capabilities(&self) -> regs::GeneralCapabilities {
        self.read_reg(regs::HPETReg::GeneralCapabilities).into()
    }

    #[must_use]
    pub fn config(&self) -> regs::GeneralConfiguration {
        self.read_reg(regs::HPETReg::GeneralConfiguration).into()
    }

    pub fn set_config(&self, value: regs::GeneralConfiguration) {
        self.write_reg(regs::HPETReg::GeneralConfiguration, value.into());
    }
}

impl core::ops::Deref for HPET {
    type Target = super::SDTHeader;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}
