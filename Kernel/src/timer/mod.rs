// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

pub mod hpet;
pub mod pit;

pub trait Timer {
    fn sleep(&self, ms: u64);
}
