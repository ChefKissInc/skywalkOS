//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use modular_bitfield::prelude::*;

#[bitfield(bits = 32)]
#[derive(Debug, BitfieldSpecifier, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub struct LocalVectorTable {
    pub vector: u8,
    pub delivery_mode: super::DeliveryMode,
    #[skip]
    __: bool,
    #[skip(setters)]
    pub pending: bool,
    pub active_low: bool,
    #[skip(setters)]
    pub remote_irr: bool,
    pub level_triggered: bool,
    pub mask: bool,
    #[skip]
    __: B15,
}

#[derive(Debug, BitfieldSpecifier, Default, Clone, Copy, PartialEq, Eq)]
#[bits = 32]
#[repr(u32)]
pub enum TimerDivide {
    #[default]
    By2 = 0b0000,
    By4 = 0b0001,
    By8 = 0b0010,
    By16 = 0b0011,
    By32 = 0b1000,
    By64 = 0b1001,
    By128 = 0b1010,
    By1 = 0b1011,
}

#[derive(Debug, BitfieldSpecifier, Default, Clone, Copy, PartialEq, Eq)]
#[bits = 2]
#[repr(u8)]
pub enum TimerMode {
    #[default]
    OneShot = 0b00,
    Periodic = 0b01,
    TscDeadline = 0b10,
}

#[bitfield(bits = 32)]
#[derive(Debug, BitfieldSpecifier, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub struct TimerLVT {
    pub vector: u8,
    #[skip]
    __: B4,
    #[skip(setters)]
    pub pending: bool,
    #[skip]
    __: B3,
    pub mask: bool,
    pub mode: TimerMode,
    #[skip]
    __: B13,
}
