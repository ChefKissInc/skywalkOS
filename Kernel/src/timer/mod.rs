// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

pub mod hpet;
pub mod pit;

pub trait Timer {
    fn sleep(&self, ms: u64);
}
