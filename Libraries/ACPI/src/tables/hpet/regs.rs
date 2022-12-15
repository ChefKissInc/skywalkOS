// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use modular_bitfield::prelude::*;
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

#[bitfield(bits = 64)]
#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub struct GeneralCapabilities {
    #[skip(setters)]
    pub rev_id: u8,
    #[skip(setters)]
    pub num_timers: B5,
    #[skip(setters)]
    pub main_cnt_64bit: bool,
    #[skip]
    __: bool,
    #[skip(setters)]
    pub legacy_replacement: bool,
    #[skip(setters)]
    pub vendor_id: u16,
    #[skip(setters)]
    pub clk_period: u32,
}

#[bitfield(bits = 64)]
#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub struct GeneralConfiguration {
    pub main_cnt_enable: bool,
    pub legacy_replacement: bool,
    #[skip]
    __: B62,
}

#[bitfield(bits = 64)]
#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub struct GeneralInterruptStatus {
    pub tmr_intr_active: bool,
    #[skip]
    __: B63,
}

#[bitfield(bits = 64)]
#[derive(Debug, Clone, Copy)]
#[repr(u64)]
pub struct TimerCfgAndCapability {
    #[skip]
    __: bool,
    pub level_triggered: bool,
    pub intr_enable: bool,
    pub periodic: bool,
    #[skip(setters)]
    pub periodic_supported: bool,
    #[skip(setters)]
    pub is_64_bit: bool,
    pub timer_accumulator: bool,
    #[skip]
    __: bool,
    pub force_32bit: bool,
    pub ioapic_intr_route: B5,
    pub fsb_intr: bool,
    #[skip(setters)]
    pub fsb_intr_supported: bool,
    #[skip]
    __: u16,
    #[skip(setters)]
    pub intr_route: u32,
}
