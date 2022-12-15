// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

pub mod hpet;
pub mod pit;

pub trait Timer {
    fn sleep(&self, ms: u64);
}
