// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#![no_std]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![allow(clippy::missing_safety_doc)]

use alloc::{string::String, vec::Vec};

use hashbrown::HashMap;

extern crate alloc;

#[cfg(target_arch = "x86_64")]
pub mod port;
pub mod registry;
#[cfg(target_arch = "x86_64")]
pub mod syscall;

use serde::{Deserialize, Serialize};

pub const USER_PHYS_VIRT_OFFSET: u64 = 0xC000_0000;

#[derive(Debug, Serialize, Deserialize)]
pub struct DCInfo<'a> {
    pub identifier: &'a str,
    pub name: &'a str,
    pub version: &'a str,
    pub description: &'a str,
    pub personalities: HashMap<&'a str, HashMap<String, registry::BCObject>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DCCache<'a> {
    #[serde(borrow)]
    pub infos: Vec<DCInfo<'a>>,
    pub payloads: HashMap<&'a str, &'a [u8]>,
}
