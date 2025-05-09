// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#[derive(Debug, Default, Clone, Copy)]
#[repr(u8)]
/// 3 bits
pub enum PATEntry {
    #[default]
    Uncacheable = 0x0,
    WriteCombining = 0x1,
    WriteThrough = 0x4,
    WriteProtected = 0x5,
    WriteBack = 0x6,
    Uncached = 0x7,
}

impl PATEntry {
    const fn into_bits(self) -> u64 {
        self as _
    }

    const fn from_bits(value: u64) -> Self {
        match value {
            0x0 => Self::Uncacheable,
            0x1 => Self::WriteCombining,
            0x4 => Self::WriteThrough,
            0x5 => Self::WriteProtected,
            0x6 => Self::WriteBack,
            0x7 => Self::Uncached,
            _ => panic!("Invalid PAT value"),
        }
    }
}

#[bitfield(u64)]
pub struct PageAttributeTable {
    #[bits(3)]
    pub pat0: PATEntry,
    #[bits(5)]
    __: u8,
    #[bits(3)]
    pub pat1: PATEntry,
    #[bits(5)]
    __: u8,
    #[bits(3)]
    pub pat2: PATEntry,
    #[bits(5)]
    __: u8,
    #[bits(3)]
    pub pat3: PATEntry,
    #[bits(5)]
    __: u8,
    #[bits(3)]
    pub pat4: PATEntry,
    #[bits(5)]
    __: u8,
    #[bits(3)]
    pub pat5: PATEntry,
    #[bits(5)]
    __: u8,
    #[bits(3)]
    pub pat6: PATEntry,
    #[bits(5)]
    __: u8,
    #[bits(3)]
    pub pat7: PATEntry,
    #[bits(5)]
    __: u8,
}

impl super::ModelSpecificReg for PageAttributeTable {
    const MSR_NUM: u32 = 0x277;
}
