// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use modular_bitfield::prelude::*;

#[derive(BitfieldSpecifier, Debug, Clone, Copy, PartialEq, Eq)]
#[bits = 2]
pub enum Polarity {
    ConformToBusSpec = 0,
    ActiveHigh = 0b01,
    ActiveLow = 0b11,
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy, PartialEq, Eq)]
#[bits = 2]
pub enum TriggerMode {
    ConformToBusSpec = 0,
    EdgeTriggered = 0b01,
    LevelTriggered = 0b11,
}

#[bitfield(bits = 16)]
#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub struct Inti {
    #[skip(setters)]
    pub polarity: Polarity,
    #[skip(setters)]
    pub trigger_mode: TriggerMode,
    #[skip]
    __: B12,
}
