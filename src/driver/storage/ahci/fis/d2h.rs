//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use modular_bitfield::prelude::*;

#[bitfield(bits = 160)]
#[derive(Debug)]
pub struct FISDeviceToHost {
    header: super::FISHeader,
    #[skip]
    __: B2,
    pub interrupt: bool,
    #[skip]
    __: B1,
    pub status: u8,
    pub error: u8,
    pub lba0: u8,
    pub lba1: u8,
    pub lba2: u8,
    pub device: u8,
    pub lba3: u8,
    pub lba4: u8,
    pub lba5: u8,
    #[skip]
    __: u8,
    pub count_low: u8,
    pub count_high: u8,
    #[skip]
    __: B48,
}
