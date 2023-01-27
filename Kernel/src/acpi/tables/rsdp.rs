// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use core::mem::size_of;

use super::{rsdt::Rsdt, xsdt::Xsdt};

#[repr(C, packed)]
pub struct Rsdp {
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
pub enum RsdtType {
    Rsdt(&'static Rsdt),
    Xsdt(&'static Xsdt),
}

#[derive(Debug)]
pub struct RsdtTypeIter {
    ptr: u64,
    is_xsdt: bool,
    curr: usize,
    total: usize,
}

impl Iterator for RsdtTypeIter {
    type Item = &'static super::SdtHeader;

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
                Some((addr as *const super::SdtHeader).as_ref().unwrap())
            }
        }
    }
}

impl RsdtType {
    #[must_use]
    pub fn iter(&self) -> RsdtTypeIter {
        unsafe {
            let (is_xsdt, length, header) = match *self {
                Self::Rsdt(v) => (false, v.length(), (v as *const Rsdt).cast::<u8>()),
                Self::Xsdt(v) => (true, v.length(), (v as *const Xsdt).cast::<u8>()),
            };
            let total = (length - size_of::<super::SdtHeader>()) / if is_xsdt { 8 } else { 4 };
            let ptr = header.add(size_of::<super::SdtHeader>()) as u64;
            RsdtTypeIter {
                ptr,
                is_xsdt,
                curr: 0,
                total,
            }
        }
    }
}

impl Rsdp {
    #[must_use]
    pub fn validate(&self) -> bool {
        let bytes = unsafe {
            core::slice::from_raw_parts((self as *const Self).cast::<u8>(), self.length())
        };
        bytes.iter().fold(0u8, |sum, &byte| sum.wrapping_add(byte)) == 0
    }

    // #[must_use]
    // pub fn signature(&self) -> &str {
    //     unsafe { core::str::from_utf8_unchecked(&self.signature).trim() }
    // }

    #[must_use]
    pub fn oem_id(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.oem_id).trim() }
    }

    #[must_use]
    pub const fn length(&self) -> usize {
        if self.revision == 0 {
            20
        } else {
            self.length as usize
        }
    }

    #[must_use]
    pub fn as_type(&self) -> RsdtType {
        if self.revision == 0 {
            let addr = u64::from(self.rsdt_addr) + amd64::paging::PHYS_VIRT_OFFSET;
            unsafe { RsdtType::Rsdt((addr as *const Rsdt).as_ref().unwrap()) }
        } else {
            let addr = self.xsdt_addr + amd64::paging::PHYS_VIRT_OFFSET;
            unsafe { RsdtType::Xsdt((addr as *const Xsdt).as_ref().unwrap()) }
        }
    }
}

impl core::fmt::Debug for Rsdp {
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
