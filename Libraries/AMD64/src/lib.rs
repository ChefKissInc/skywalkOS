// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

#![no_std]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![allow(clippy::missing_safety_doc, clippy::multiple_crate_versions)]

pub mod cpuid;
pub mod io;
pub mod msr;
pub mod paging;
pub mod spec;
