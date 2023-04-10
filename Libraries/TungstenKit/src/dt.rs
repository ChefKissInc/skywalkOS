// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::{borrow::ToOwned, string::String, vec::Vec};

use hashbrown::HashMap;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "x86_64")]
use crate::syscall::SystemCall;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct OSDTEntry(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u64)]
pub enum GetOSDTEntryReqType {
    Parent,
    Children,
    Properties,
    Property,
}
#[cfg(target_arch = "x86_64")]
impl OSDTEntry {
    fn get_info(&self, ty: GetOSDTEntryReqType, k: Option<&str>) -> Vec<u8> {
        let (mut ptr, mut len): (u64, u64);
        unsafe {
            core::arch::asm!(
                "int 249",
                in("rdi") SystemCall::GetDTEntryInfo as u64,
                in("rsi") self.0,
                in("rdx") ty as u64,
                in("rcx") k.map_or(0, |s| s.as_ptr() as u64),
                in("r8") k.map_or(0, |s| s.len() as u64),
                out("rax") ptr,
                lateout("rdi") len,
                options(nostack, preserves_flags),
            );
            Vec::from_raw_parts(ptr as *mut u8, len as _, len as _)
        }
    }

    #[must_use]
    pub fn new_child(&self) -> Self {
        let mut id: u64;
        unsafe {
            core::arch::asm!(
                "int 249",
                in("rdi") SystemCall::NewDTEntry as u64,
                in("rsi") self.0,
                out("rax") id,
                options(nostack, preserves_flags),
            );
        }
        id.into()
    }

    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        postcard::from_bytes(&self.get_info(GetOSDTEntryReqType::Parent, None)).unwrap()
    }

    #[must_use]
    pub fn children(&self) -> Vec<Self> {
        postcard::from_bytes(&self.get_info(GetOSDTEntryReqType::Children, None)).unwrap()
    }

    #[must_use]
    pub fn properties(&self) -> HashMap<String, OSValue> {
        postcard::from_bytes(&self.get_info(GetOSDTEntryReqType::Properties, None)).unwrap()
    }

    #[must_use]
    pub fn get_property(&self, k: &str) -> Option<OSValue> {
        postcard::from_bytes(&self.get_info(GetOSDTEntryReqType::Property, Some(k))).unwrap()
    }
}

#[cfg(target_arch = "x86_64")]
impl From<u64> for OSDTEntry {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[cfg(target_arch = "x86_64")]
impl From<OSDTEntry> for u64 {
    fn from(val: OSDTEntry) -> Self {
        val.0
    }
}

#[cfg(target_arch = "x86_64")]
impl From<&OSDTEntry> for u64 {
    fn from(val: &OSDTEntry) -> Self {
        val.0
    }
}
