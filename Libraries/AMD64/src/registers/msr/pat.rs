// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use modular_bitfield::prelude::*;

#[derive(BitfieldSpecifier, Debug, Default, Clone, Copy)]
#[bits = 3]
#[repr(u8)]
pub enum PATEntry {
    #[default]
    Uncacheable = 0x0,
    WriteCombining = 0x1,
    WriteThrough = 0x4,
    WriteProtected = 0x5,
    WriteBack = 0x6,
    Uncached = 0x7,
}

#[bitfield(bits = 64)]
#[derive(BitfieldSpecifier, Debug, Default, Clone, Copy)]
#[repr(u64)]
pub struct PageAttributeTable {
    pub pat0: PATEntry,
    #[skip]
    __: B5,
    pub pat1: PATEntry,
    #[skip]
    __: B5,
    pub pat2: PATEntry,
    #[skip]
    __: B5,
    pub pat3: PATEntry,
    #[skip]
    __: B5,
    pub pat4: PATEntry,
    #[skip]
    __: B5,
    pub pat5: PATEntry,
    #[skip]
    __: B5,
    pub pat6: PATEntry,
    #[skip]
    __: B5,
    pub pat7: PATEntry,
    #[skip]
    __: B5,
}

impl super::ModelSpecificReg for PageAttributeTable {
    const MSR_NUM: u32 = 0x277;
}
