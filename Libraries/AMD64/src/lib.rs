// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

#![no_std]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![allow(clippy::missing_safety_doc)]

pub mod cpuid;
pub mod io;
pub mod msr;
pub mod paging;
pub mod spec;
