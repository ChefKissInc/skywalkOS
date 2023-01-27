// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use core::mem::size_of;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MCFG {
    header: super::SDTHeader,
    __: u64,
}

impl core::ops::Deref for MCFG {
    type Target = super::SDTHeader;

    fn deref(&self) -> &Self::Target {
        &self.header
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct MCFGEntry {
    pub base: u64,
    pub segment: u16,
    pub bus_start: u8,
    pub bus_end: u8,
    __: u32,
}

impl MCFG {
    #[must_use]
    pub fn entries(&self) -> &'static [MCFGEntry] {
        let len = (self.length() - size_of::<Self>()) / size_of::<MCFGEntry>();
        unsafe {
            let data = ((self as *const _ as usize) + size_of::<Self>()) as *const MCFGEntry;
            core::slice::from_raw_parts(data, len)
        }
    }
}
