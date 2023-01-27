// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use modular_bitfield::prelude::*;

#[bitfield(bits = 64)]
#[derive(BitfieldSpecifier, Debug, Default, Clone, Copy)]
#[repr(u64)]
pub struct ExtendedFeatureEnableReg {
    pub syscall_ext: bool,
    #[skip]
    __: B7,
    pub long_mode: bool,
    #[skip]
    __: B1,
    pub long_mode_active: bool,
    pub no_execute: bool,
    pub secure_virtual_machine: bool,
    pub long_mode_seg_limit: bool,
    pub fast_fxsave_fxrstor: bool,
    pub translation_cache_ext: bool,
    #[skip]
    __: B1,
    pub mcommit: bool,
    pub interruptible_wbinvd: bool,
    #[skip]
    __: B45,
}

impl super::ModelSpecificReg for ExtendedFeatureEnableReg {
    const MSR_NUM: u32 = 0xC000_0080;
}
