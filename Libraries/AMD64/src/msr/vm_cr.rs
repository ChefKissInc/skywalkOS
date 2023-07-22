// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use modular_bitfield::prelude::*;

#[bitfield(bits = 64)]
#[derive(BitfieldSpecifier, Debug, Default, Clone, Copy)]
#[repr(u64)]
pub struct VmCr {
    pub disable_debug_port: bool,
    pub reserve_init: bool,
    pub disable_a20: bool,
    pub locked: bool,
    pub disabled: bool,
    #[skip]
    __: B59,
}

impl super::ModelSpecificReg for VmCr {
    const MSR_NUM: u32 = 0xC001_0114;
}
