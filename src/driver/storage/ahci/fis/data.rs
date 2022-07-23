//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use core::mem::size_of;

use modular_bitfield::prelude::*;

#[bitfield(bits = 32)]
#[derive(Debug)]
#[repr(u32)]
pub struct FISData {
    header: super::FISHeader,
    #[skip]
    __: B20,
}

impl FISData {
    pub fn data_ptr(&mut self) -> *mut u8 {
        (self as *mut _ as usize + size_of::<Self>()) as *mut _
    }
}
