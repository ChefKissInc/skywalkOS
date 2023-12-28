// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::string::String;
#[cfg(feature = "userspace")]
use alloc::{borrow::ToOwned, vec::Vec};

#[cfg(feature = "userspace")]
use hashbrown::HashMap;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

use crate::osvalue::OSValue;
#[cfg(feature = "userspace")]
use crate::syscall::SystemCall;

pub const OSDTENTRY_NAME_KEY: &str = "_Name";
pub const FKEXT_MATCH_KEY: &str = "_FKExtMatch";
pub const FKEXT_PROC_KEY: &str = "_FKExtProc";

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct OSDTEntry(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u64)]
pub enum OSDTEntryInfo {
    Parent,
    Children,
    Properties,
    Property,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSDTEntryProp(pub String, pub OSValue);

#[cfg(feature = "userspace")]
impl OSDTEntry {
    fn get_info(&self, ty: OSDTEntryInfo, k: Option<&str>) -> Vec<u8> {
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
                options(nostack),
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
                options(nostack),
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
        postcard::from_bytes(&self.get_info(OSDTEntryInfo::Parent, None)).unwrap()
    }

    #[must_use]
    pub fn children(&self) -> Vec<Self> {
        postcard::from_bytes(&self.get_info(OSDTEntryInfo::Children, None)).unwrap()
    }

    #[must_use]
    pub fn properties(&self) -> HashMap<String, OSValue> {
        postcard::from_bytes(&self.get_info(OSDTEntryInfo::Properties, None)).unwrap()
    }

    #[must_use]
    pub fn get_property(&self, k: &str) -> Option<OSValue> {
        postcard::from_bytes(&self.get_info(OSDTEntryInfo::Property, Some(k))).unwrap()
    }

    pub fn set_property(&self, k: &str, v: OSValue) {
        let req = postcard::to_allocvec(&OSDTEntryProp(k.to_owned(), v)).unwrap();
        unsafe {
            core::arch::asm!(
                "int 249",
                in("rdi") SystemCall::SetOSDTEntryProp as u64,
                in("rsi") self.0,
                in("rdx") req.as_ptr() as u64,
                in("rcx") req.len() as u64,
                options(nostack),
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
