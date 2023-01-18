// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

pub mod pml4;

use modular_bitfield::prelude::*;

pub const PHYS_VIRT_OFFSET: u64 = 0xFFFF_8000_0000_0000;
pub const KERNEL_VIRT_OFFSET: u64 = 0xFFFF_FFFF_8000_0000;

#[derive(Debug)]
pub struct PageTableOffsets {
    pub pml4: u64,
    pub pdp: u64,
    pub pd: u64,
    pub pt: u64,
}

impl PageTableOffsets {
    #[inline]
    #[must_use]
    pub const fn new(virtual_address: u64) -> Self {
        Self {
            pml4: (virtual_address >> 39) & 0x1FF,
            pdp: (virtual_address >> 30) & 0x1FF,
            pd: (virtual_address >> 21) & 0x1FF,
            pt: (virtual_address >> 12) & 0x1FF,
        }
    }
}

#[bitfield(bits = 64)]
#[repr(u64)]
#[derive(Debug, Default, Clone, Copy)]
pub struct PageTableEntry {
    pub present: bool,
    pub writable: bool,
    pub user: bool,
    pub pwt: bool,
    pub pcd: bool,
    #[skip(setters)]
    pub accessed: bool,
    #[skip(setters)]
    pub dirty: bool,
    pub huge_or_pat: bool,
    pub global: bool,
    pub available_to_os: B3,
    pub address: B40,
    pub available_to_os2: B11,
    pub no_execute: bool,
}

#[repr(C, align(4096))]
#[derive(Debug)]
pub struct PageTable {
    pub entries: [PageTableEntry; 512],
}

impl Default for PageTable {
    fn default() -> Self {
        Self::new()
    }
}

impl PageTable {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::new(); 512],
        }
    }
}
