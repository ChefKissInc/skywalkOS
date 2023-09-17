// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#[bitfield(u64)]
pub struct ExtendedFeatureEnableReg {
    pub syscall_ext: bool,
    #[bits(7)]
    __: u8,
    pub long_mode: bool,
    __: bool,
    pub long_mode_active: bool,
    pub no_execute: bool,
    pub secure_virtual_machine: bool,
    pub long_mode_seg_limit: bool,
    pub fast_fxsave_fxrstor: bool,
    pub translation_cache_ext: bool,
    __: bool,
    pub mcommit: bool,
    pub interruptible_wbinvd: bool,
    #[bits(45)]
    __: u64,
}

impl super::ModelSpecificReg for ExtendedFeatureEnableReg {
    const MSR_NUM: u32 = 0xC000_0080;
}
