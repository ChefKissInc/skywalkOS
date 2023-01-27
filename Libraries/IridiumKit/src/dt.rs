// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::{borrow::ToOwned, string::String, vec::Vec};

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub enum OSValue {
    Boolean(bool),
    String(String),
    USize(usize),
    U64(u64),
    U32(u32),
    U16(u16),
    U8(u8),
    Vec(Vec<Self>),
    Dictionary(HashMap<String, Self>),
}

impl From<bool> for OSValue {
    fn from(val: bool) -> Self {
        Self::Boolean(val)
    }
}

impl From<&str> for OSValue {
    fn from(val: &str) -> Self {
        Self::String(val.to_owned())
    }
}

impl From<String> for OSValue {
    fn from(val: String) -> Self {
        Self::String(val)
    }
}

impl From<usize> for OSValue {
    fn from(val: usize) -> Self {
        Self::USize(val)
    }
}

impl From<u64> for OSValue {
    fn from(val: u64) -> Self {
        Self::U64(val)
    }
}

impl From<u32> for OSValue {
    fn from(val: u32) -> Self {
        Self::U32(val)
    }
}

impl From<u16> for OSValue {
    fn from(val: u16) -> Self {
        Self::U16(val)
    }
}

impl From<u8> for OSValue {
    fn from(val: u8) -> Self {
        Self::U8(val)
    }
}

impl From<Vec<Self>> for OSValue {
    fn from(val: Vec<Self>) -> Self {
        Self::Vec(val)
    }
}

impl From<HashMap<String, Self>> for OSValue {
    fn from(val: HashMap<String, Self>) -> Self {
        Self::Dictionary(val)
    }
}

impl TryFrom<OSValue> for bool {
    type Error = ();

    fn try_from(val: OSValue) -> Result<Self, Self::Error> {
        match val {
            OSValue::Boolean(b) => Ok(b),
            _ => Err(()),
        }
    }
}

impl TryFrom<OSValue> for String {
    type Error = ();

    fn try_from(val: OSValue) -> Result<Self, Self::Error> {
        match val {
            OSValue::String(s) => Ok(s),
            _ => Err(()),
        }
    }
}

impl TryFrom<OSValue> for usize {
    type Error = ();

    fn try_from(val: OSValue) -> Result<Self, Self::Error> {
        match val {
            OSValue::USize(u) => Ok(u),
            _ => Err(()),
        }
    }
}

impl TryFrom<OSValue> for u64 {
    type Error = ();

    fn try_from(val: OSValue) -> Result<Self, Self::Error> {
        match val {
            OSValue::U64(u) => Ok(u),
            _ => Err(()),
        }
    }
}

impl TryFrom<OSValue> for u32 {
    type Error = ();

    fn try_from(val: OSValue) -> Result<Self, Self::Error> {
        match val {
            OSValue::U32(u) => Ok(u),
            _ => Err(()),
        }
    }
}

impl TryFrom<OSValue> for u16 {
    type Error = ();

    fn try_from(val: OSValue) -> Result<Self, Self::Error> {
        match val {
            OSValue::U16(u) => Ok(u),
            _ => Err(()),
        }
    }
}

impl TryFrom<OSValue> for u8 {
    type Error = ();

    fn try_from(val: OSValue) -> Result<Self, Self::Error> {
        match val {
            OSValue::U8(u) => Ok(u),
            _ => Err(()),
        }
    }
}

impl TryFrom<OSValue> for Vec<OSValue> {
    type Error = ();

    fn try_from(val: OSValue) -> Result<Self, Self::Error> {
        match val {
            OSValue::Vec(v) => Ok(v),
            _ => Err(()),
        }
    }
}

impl TryFrom<OSValue> for HashMap<String, OSValue> {
    type Error = ();

    fn try_from(val: OSValue) -> Result<Self, Self::Error> {
        match val {
            OSValue::Dictionary(d) => Ok(d),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct OSDTEntry(u64);

impl OSDTEntry {
    #[inline]
    #[must_use]
    pub const fn from_id(id: u64) -> Self {
        Self(id)
    }
}
