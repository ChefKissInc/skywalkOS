// Copyright (c) ChefKiss Inc 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

pub mod hpet;
pub mod pit;

pub trait Timer {
    fn sleep(&self, ms: u64);
}
