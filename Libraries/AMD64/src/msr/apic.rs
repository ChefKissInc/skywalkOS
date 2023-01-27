// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use modular_bitfield::prelude::*;

#[bitfield(bits = 64)]
#[derive(BitfieldSpecifier, Debug, Default, Clone, Copy)]
#[repr(u64)]
pub struct APICBase {
    #[skip]
    __: u8,
    #[skip(setters)]
    pub bsp: bool,
    #[skip]
    __: bool,
    pub x2apic_enabled: bool,
    pub apic_global_enable: bool,
    pub apic_base: B52,
}

impl super::ModelSpecificReg for APICBase {
    const MSR_NUM: u32 = 0x1B;
}
