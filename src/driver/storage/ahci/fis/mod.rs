//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#![allow(dead_code)]

use modular_bitfield::prelude::*;

pub use self::{d2h::*, data::*, dma_setup::*, h2d::*, pio_setup::*};

mod d2h;
mod data;
mod dma_setup;
mod h2d;
mod pio_setup;

#[derive(Debug, BitfieldSpecifier)]
#[bits = 8]
#[repr(u8)]
pub enum FISType {
    HostToDevice = 0x27,
    DeviceToHost = 0x34,
    // DMAACT = 0x39,
    DMASetup = 0x41,
    Data = 0x46,
    // BIST = 0x58,
    PIOSetup = 0x5F,
    SetDeviceBits = 0xA1,
}

#[bitfield(bits = 12)]
#[derive(Debug, BitfieldSpecifier)]
pub struct FISHeader {
    pub fis_type: FISType,
    pub pmport: B4,
}
