// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::{borrow::ToOwned, string::String, vec::Vec};

use hashbrown::HashMap;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

use crate::{osvalue::OSValue, syscall::SystemCall};

pub const OSDTENTRY_NAME_KEY: &str = "_Name";
pub const TKEXT_MATCH_KEY: &str = "_TKExtMatch";
pub const TKEXT_PROC_KEY: &str = "_TKExtProc";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetOSDTEntryPropReq(pub String, pub OSValue);

impl OSDTEntry {
    fn get_info(&self, ty: GetOSDTEntryReqType, k: Option<&str>) -> Vec<u8> {
        let (mut ptr, mut len): (u64, u64);
        unsafe {
            core::arch::asm!(
                "int 249",
                in("rdi") SystemCall::GetOSDTEntryInfo as u64,
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
    pub fn new_child(&self, name: Option<&str>) -> Self {
        let mut id: u64;
        unsafe {
            core::arch::asm!(
                "int 249",
                in("rdi") SystemCall::NewOSDTEntry as u64,
                in("rsi") self.0,
                out("rax") id,
                options(nostack, preserves_flags),
            );
        }
        let ret: Self = id.into();
        if let Some(name) = name {
            ret.set_property(OSDTENTRY_NAME_KEY, name.into());
        }
        ret
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

    pub fn set_property(&self, k: &str, v: OSValue) {
        let req = postcard::to_allocvec(&SetOSDTEntryPropReq(k.to_owned(), v)).unwrap();
        unsafe {
            core::arch::asm!(
                "int 249",
                in("rdi") SystemCall::SetOSDTEntryProp as u64,
                in("rsi") self.0,
                in("rdx") req.as_ptr() as u64,
                in("rcx") req.len() as u64,
                options(nostack, preserves_flags),
            );
        }
    }
}

impl From<u64> for OSDTEntry {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<OSDTEntry> for u64 {
    fn from(val: OSDTEntry) -> Self {
        val.0
    }
}

impl From<&OSDTEntry> for u64 {
    fn from(val: &OSDTEntry) -> Self {
        val.0
    }
}
