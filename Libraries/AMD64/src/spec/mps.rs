// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 2 bits
pub enum Polarity {
    ConformToBusSpec = 0b00,
    ActiveHigh = 0b01,
    ActiveLow = 0b11,
}

impl Polarity {
    const fn into_bits(self) -> u16 {
        self as _
    }

    const fn from_bits(value: u16) -> Self {
        match value {
            0b00 => Self::ConformToBusSpec,
            0b01 => Self::ActiveHigh,
            0b11 => Self::ActiveLow,
            _ => panic!("Invalid MPS Polarity"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 2 bits
pub enum TriggerMode {
    ConformToBusSpec = 0b00,
    EdgeTriggered = 0b01,
    LevelTriggered = 0b11,
}

impl TriggerMode {
    const fn into_bits(self) -> u16 {
        self as _
    }

    const fn from_bits(value: u16) -> Self {
        match value {
            0b00 => Self::ConformToBusSpec,
            0b01 => Self::EdgeTriggered,
            0b11 => Self::LevelTriggered,
            _ => panic!("Invalid MPS TriggerMode"),
        }
    }
}

#[bitfield(u16)]
pub struct INTI {
    #[bits(2)]
    pub polarity: Polarity,
    #[bits(2)]
    pub trigger_mode: TriggerMode,
    #[bits(12)]
    __: u16,
}
