// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#![no_std]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]

use alloc::vec::Vec;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

extern crate alloc;

#[derive(Debug, Serialize, Deserialize)]
pub struct DistributionInfo<'a> {
    pub branding: &'a str,
    pub version: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DCPersonality<'a> {
    pub match_: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DCInfo<'a> {
    pub identifier: &'a str,
    pub name: &'a str,
    pub version: &'a str,
    pub description: &'a str,
    pub personalities: HashMap<&'a str, DCPersonality<'a>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DCCache<'a> {
    pub branding: &'a str,
    pub version: &'a str,
    pub infos: Vec<DCInfo<'a>>,
    pub payloads: HashMap<&'a str, &'a [u8]>,
}
