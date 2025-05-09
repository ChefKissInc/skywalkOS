// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#[bitfield(u64)]
pub struct VmCr {
    pub disable_debug_port: bool,
    pub reserve_init: bool,
    pub disable_a20: bool,
    pub locked: bool,
    pub disabled: bool,
    #[bits(59)]
    __: u64,
}

impl super::ModelSpecificReg for VmCr {
    const MSR_NUM: u32 = 0xC001_0114;
}
