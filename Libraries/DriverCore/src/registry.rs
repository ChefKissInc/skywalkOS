// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::{borrow::ToOwned, string::String, vec::Vec};

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub enum BCObject {
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

impl From<bool> for BCObject {
    fn from(val: bool) -> Self {
        Self::Boolean(val)
    }
}

impl From<&str> for BCObject {
    fn from(val: &str) -> Self {
        Self::String(val.to_owned())
    }
}

impl From<String> for BCObject {
    fn from(val: String) -> Self {
        Self::String(val)
    }
}

impl From<usize> for BCObject {
    fn from(val: usize) -> Self {
        Self::USize(val)
    }
}

impl From<u64> for BCObject {
    fn from(val: u64) -> Self {
        Self::U64(val)
    }
}

impl From<u32> for BCObject {
    fn from(val: u32) -> Self {
        Self::U32(val)
    }
}

impl From<u16> for BCObject {
    fn from(val: u16) -> Self {
        Self::U16(val)
    }
}

impl From<u8> for BCObject {
    fn from(val: u8) -> Self {
        Self::U8(val)
    }
}

impl From<Vec<Self>> for BCObject {
    fn from(val: Vec<Self>) -> Self {
        Self::Vec(val)
    }
}

impl From<HashMap<String, Self>> for BCObject {
    fn from(val: HashMap<String, Self>) -> Self {
        Self::Dictionary(val)
    }
}

impl TryFrom<BCObject> for bool {
    type Error = ();

    fn try_from(val: BCObject) -> Result<Self, Self::Error> {
        match val {
            BCObject::Boolean(b) => Ok(b),
            _ => Err(()),
        }
    }
}

impl TryFrom<BCObject> for String {
    type Error = ();

    fn try_from(val: BCObject) -> Result<Self, Self::Error> {
        match val {
            BCObject::String(s) => Ok(s),
            _ => Err(()),
        }
    }
}

impl TryFrom<BCObject> for usize {
    type Error = ();

    fn try_from(val: BCObject) -> Result<Self, Self::Error> {
        match val {
            BCObject::USize(u) => Ok(u),
            _ => Err(()),
        }
    }
}

impl TryFrom<BCObject> for u64 {
    type Error = ();

    fn try_from(val: BCObject) -> Result<Self, Self::Error> {
        match val {
            BCObject::U64(u) => Ok(u),
            _ => Err(()),
        }
    }
}

impl TryFrom<BCObject> for u32 {
    type Error = ();

    fn try_from(val: BCObject) -> Result<Self, Self::Error> {
        match val {
            BCObject::U32(u) => Ok(u),
            _ => Err(()),
        }
    }
}

impl TryFrom<BCObject> for u16 {
    type Error = ();

    fn try_from(val: BCObject) -> Result<Self, Self::Error> {
        match val {
            BCObject::U16(u) => Ok(u),
            _ => Err(()),
        }
    }
}

impl TryFrom<BCObject> for u8 {
    type Error = ();

    fn try_from(val: BCObject) -> Result<Self, Self::Error> {
        match val {
            BCObject::U8(u) => Ok(u),
            _ => Err(()),
        }
    }
}

impl TryFrom<BCObject> for Vec<BCObject> {
    type Error = ();

    fn try_from(val: BCObject) -> Result<Self, Self::Error> {
        match val {
            BCObject::Vec(v) => Ok(v),
            _ => Err(()),
        }
    }
}

impl TryFrom<BCObject> for HashMap<String, BCObject> {
    type Error = ();

    fn try_from(val: BCObject) -> Result<Self, Self::Error> {
        match val {
            BCObject::Dictionary(d) => Ok(d),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct BCRegistryEntry(u64);

impl BCRegistryEntry {
    #[inline]
    #[must_use]
    pub const fn from_id(id: u64) -> Self {
        Self(id)
    }
}
