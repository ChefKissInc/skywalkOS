// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#![no_std]
#![deny(warnings, clippy::nursery, unused_extern_crates)]
#![allow(clippy::missing_safety_doc)]

pub mod cpuid;
pub mod io;
pub mod msr;
pub mod paging;
pub mod spec;

#[macro_use]
extern crate bitfield_struct;
