//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use modular_bitfield::prelude::*;

#[bitfield(bits = 224)]
#[derive(Debug)]
pub struct FISDMASetup {
    header: super::FISHeader,
    #[skip]
    __: B1,
    pub device_to_host: bool,
    pub interrupt: bool,
    pub auto_activate: bool,
    #[skip]
    __: u16,
    pub buffer_id: u64,
    #[skip]
    __: u32,
    pub buffer_offset: u32,
    pub transfer_count: u32,
    #[skip]
    __: u32,
}
