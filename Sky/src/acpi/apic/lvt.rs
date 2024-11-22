// Copyright (c) ChefKiss 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use super::DeliveryMode;

#[bitfield(u32)]
pub struct LocalVectorTable {
    pub vector: u8,
    #[bits(3, into = DeliveryMode::into_bits_32, from = DeliveryMode::from_bits_32)]
    pub delivery_mode: DeliveryMode,
    __: bool,
    pub pending: bool,
    pub active_low: bool,
    pub remote_irr: bool,
    pub level_triggered: bool,
    pub mask: bool,
    #[bits(15)]
    __: u16,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// 2 bits
pub enum TimerMode {
    #[default]
    OneShot = 0b00,
    Periodic = 0b01,
    TscDeadline = 0b10,
}

impl TimerMode {
    const fn into_bits(self) -> u32 {
        self as _
    }

    const fn from_bits(value: u32) -> Self {
        match value {
            0b00 => Self::OneShot,
            0b01 => Self::Periodic,
            0b10 => Self::TscDeadline,
            _ => panic!("Invalid value for TimerMode"),
        }
    }
}

#[bitfield(u32)]
pub struct TimerLVT {
    pub vector: u8,
    #[bits(4)]
    __: u8,
    pub pending: bool,
    #[bits(3)]
    __: u8,
    pub mask: bool,
    #[bits(2)]
    pub mode: TimerMode,
    #[bits(13)]
    __: u16,
}
