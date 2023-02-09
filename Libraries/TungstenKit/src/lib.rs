// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

#![no_std]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![allow(clippy::missing_safety_doc)]

use alloc::{string::String, vec::Vec};

use hashbrown::HashMap;

extern crate alloc;

pub mod dt;
#[cfg(target_arch = "x86_64")]
pub mod port;
#[cfg(target_arch = "x86_64")]
pub mod syscall;

use serde::{Deserialize, Serialize};

pub const USER_PHYS_VIRT_OFFSET: u64 = 0xC000_0000;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TKInfo {
    pub identifier: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub matching_props: HashMap<String, dt::OSValue>,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TKCache(pub Vec<(TKInfo, Vec<u8>)>);
