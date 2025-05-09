// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use num_enum::IntoPrimitive;

#[repr(u64)]
#[derive(IntoPrimitive)]
pub enum HPETReg {
    GeneralCapabilities = 0x000,
    GeneralConfiguration = 0x010,
    GeneralInterruptStatus = 0x020,
    MainCounterValue = 0x0F0,
    TimerNConfig = 0x100,
    TimerNComparator = 0x108,
    TimerNFSBInterruptRoute,
}

#[bitfield(u64)]
pub struct GeneralCapabilities {
    pub rev_id: u8,
    #[bits(5)]
    pub num_timers: u8,
    pub main_cnt_64bit: bool,
    __: bool,
    pub legacy_replacement: bool,
    pub vendor_id: u16,
    pub clk_period: u32,
}

#[bitfield(u64)]
pub struct GeneralConfiguration {
    pub main_cnt_enable: bool,
    pub legacy_replacement: bool,
    #[bits(62)]
    __: u64,
}

#[bitfield(u64)]
pub struct GeneralInterruptStatus {
    pub tmr_intr_active: bool,
    #[bits(63)]
    __: u64,
}

#[bitfield(u64)]
pub struct TimerCfgAndCapability {
    __: bool,
    pub level_triggered: bool,
    pub intr_enable: bool,
    pub periodic: bool,
    pub periodic_supported: bool,
    pub is_64_bit: bool,
    pub timer_accumulator: bool,
    __: bool,
    pub force_32bit: bool,
    #[bits(5)]
    pub ioapic_intr_route: u8,
    pub fsb_intr: bool,
    pub fsb_intr_supported: bool,
    __: u16,
    pub intr_route: u32,
}
