// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#[bitfield(u64)]
pub struct APICBase {
    __: u8,
    pub bsp: bool,
    __: bool,
    pub x2apic_enabled: bool,
    pub apic_global_enable: bool,
    #[bits(52)]
    pub apic_base: u64,
}

impl super::ModelSpecificReg for APICBase {
    const MSR_NUM: u32 = 0x1B;
}
