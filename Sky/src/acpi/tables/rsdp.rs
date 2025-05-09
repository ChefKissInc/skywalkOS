// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use core::mem::size_of;

use super::{rsdt::RootSystemDescTable, xsdt::ExtendedSystemDescTable};

#[repr(C, packed)]
pub struct RootSystemDescPtr {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    pub revision: u8,
    rsdt_addr: u32,
    length: u32,
    xsdt_addr: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

#[derive(Debug)]
pub enum RSDTType {
    RootSystemDescTable(&'static RootSystemDescTable),
    ExtendedSystemDescTable(&'static ExtendedSystemDescTable),
}

#[derive(Debug)]
pub struct RSDTTypeIter {
    ptr: u64,
    is_xsdt: bool,
    curr: usize,
    total: usize,
}

impl Iterator for RSDTTypeIter {
    type Item = &'static super::SystemDescTableHeader;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.total {
            None
        } else {
            unsafe {
                let addr = if self.is_xsdt {
                    (self.ptr as *const u64).add(self.curr).read_unaligned()
                } else {
                    u64::from((self.ptr as *const u32).add(self.curr).read_unaligned())
                } + amd64::paging::PHYS_VIRT_OFFSET;
                self.curr += 1;
                Some(&*(addr as *const super::SystemDescTableHeader))
            }
        }
    }
}

impl RSDTType {
    pub fn iter(&self) -> RSDTTypeIter {
        unsafe {
            let (is_xsdt, length, header) = match *self {
                Self::RootSystemDescTable(v) => (
                    false,
                    v.length(),
                    (v as *const RootSystemDescTable).cast::<u8>(),
                ),
                Self::ExtendedSystemDescTable(v) => (
                    true,
                    v.length(),
                    (v as *const ExtendedSystemDescTable).cast::<u8>(),
                ),
            };
            let total =
                (length - size_of::<super::SystemDescTableHeader>()) / if is_xsdt { 8 } else { 4 };
            let ptr = header.add(size_of::<super::SystemDescTableHeader>()) as u64;
            RSDTTypeIter {
                ptr,
                is_xsdt,
                curr: 0,
                total,
            }
        }
    }
}

impl RootSystemDescPtr {
    pub fn validate(&self) -> bool {
        let bytes = unsafe {
            core::slice::from_raw_parts((self as *const Self).cast::<u8>(), self.length())
        };
        bytes.iter().fold(0u8, |sum, &byte| sum.wrapping_add(byte)) == 0
    }

    pub fn oem_id(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.oem_id).trim() }
    }

    pub const fn length(&self) -> usize {
        if self.revision == 0 {
            20
        } else {
            self.length as usize
        }
    }

    pub fn as_type(&self) -> RSDTType {
        if self.revision == 0 {
            let addr = u64::from(self.rsdt_addr) + amd64::paging::PHYS_VIRT_OFFSET;
            unsafe { RSDTType::RootSystemDescTable(&*(addr as *const RootSystemDescTable)) }
        } else {
            let addr = self.xsdt_addr + amd64::paging::PHYS_VIRT_OFFSET;
            unsafe { RSDTType::ExtendedSystemDescTable(&*(addr as *const ExtendedSystemDescTable)) }
        }
    }
}

impl core::fmt::Debug for RootSystemDescPtr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RSDP")
            .field("valid", &self.validate())
            .field("oem_id", &self.oem_id())
            .field("revision", &self.revision)
            .field("length", &self.length())
            .field("type", &self.as_type())
            .finish()
    }
}
