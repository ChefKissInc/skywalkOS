// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

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
