// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use core::{any::type_name, mem::size_of};

use super::{rsdt::RSDT, xsdt::XSDT};

#[repr(C, packed)]
pub struct RSDP {
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
    Rsdt(&'static RSDT),
    Xsdt(&'static XSDT),
}

#[derive(Debug)]
pub struct RSDTTypeIter {
    ptr: u64,
    is_xsdt: bool,
    curr: usize,
    total: usize,
}

impl Iterator for RSDTTypeIter {
    type Item = &'static super::SDTHeader;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.total {
            None
        } else {
            unsafe {
                let addr = if self.is_xsdt {
                    *(self.ptr as *const u64).add(self.curr)
                } else {
                    u64::from(*(self.ptr as *const u32).add(self.curr))
                };
                self.curr += 1;
                ((addr + amd64::paging::PHYS_VIRT_OFFSET) as *const super::SDTHeader).as_ref()
            }
        }
    }
}

impl RSDTType {
    #[must_use]
    pub fn iter(&self) -> RSDTTypeIter {
        unsafe {
            let (is_xsdt, length, header) = match *self {
                Self::Rsdt(v) => (false, v.length(), (v as *const RSDT).cast::<u8>()),
                Self::Xsdt(v) => (true, v.length(), (v as *const XSDT).cast::<u8>()),
            };
            let total = (length - size_of::<super::SDTHeader>()) / if is_xsdt { 8 } else { 4 };
            let ptr = header.add(size_of::<super::SDTHeader>()) as u64;
            RSDTTypeIter {
                ptr,
                is_xsdt,
                curr: 0,
                total,
            }
        }
    }
}

impl RSDP {
    #[must_use]
    pub fn validate(&self) -> bool {
        let sum = unsafe {
            let bytes =
                core::slice::from_raw_parts((self as *const Self).cast::<u8>(), self.length());
            bytes.iter().fold(0u8, |sum, &byte| sum.wrapping_add(byte))
        };

        sum == 0
    }

    #[must_use]
    pub fn signature(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.signature).trim() }
    }

    #[must_use]
    pub fn oem_id(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.oem_id).trim() }
    }

    /// If ACPI 1.0, return fixed size, else return length field
    #[must_use]
    pub const fn length(&self) -> usize {
        if self.revision == 0 {
            20
        } else {
            self.length as usize
        }
    }

    #[must_use]
    pub const fn as_type(&self) -> RSDTType {
        // This is fine.
        unsafe {
            if self.revision == 0 {
                let addr = self.rsdt_addr as u64 + amd64::paging::PHYS_VIRT_OFFSET;
                RSDTType::Rsdt(&*(addr as *const RSDT))
            } else {
                let addr = self.xsdt_addr + amd64::paging::PHYS_VIRT_OFFSET;
                RSDTType::Xsdt(&*(addr as *const XSDT))
            }
        }
    }
}

impl core::fmt::Debug for RSDP {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct(type_name::<Self>())
            .field("valid", &self.validate())
            .field("oem_id", &self.oem_id())
            .field("revision", &self.revision)
            .field("length", &self.length())
            .field("type", &self.as_type())
            .finish()
    }
}
