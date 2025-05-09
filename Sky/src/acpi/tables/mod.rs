// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

pub mod hpet;
pub mod madt;
pub mod rsdp;
pub mod rsdt;
pub mod xsdt;

#[derive(Clone, Copy)]
#[repr(C, packed)]
pub struct SystemDescTableHeader {
    signature: [u8; 4],
    length: u32,
    pub revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    pub oem_revision: u32,
    creator_id: [u8; 4],
    pub creator_revision: u32,
}

impl SystemDescTableHeader {
    pub fn validate(&self) -> bool {
        let bytes = unsafe {
            core::slice::from_raw_parts((self as *const Self).cast::<u8>(), self.length())
        };
        let sum = bytes.iter().fold(0u8, |sum, &byte| sum.wrapping_add(byte));

        sum == 0
    }

    pub fn signature(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.signature).trim() }
    }

    pub const fn length(&self) -> usize {
        self.length as usize
    }

    pub fn oem_id(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.oem_id).trim() }
    }

    pub fn oem_table_id(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.oem_table_id).trim() }
    }

    pub fn creator_id(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.creator_id).trim() }
    }
}

impl core::fmt::Debug for SystemDescTableHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let rev = self.oem_revision;
        let cr_rev = self.creator_revision;
        f.debug_struct("SystemDescTableHeader")
            .field("valid", &self.validate())
            .field("signature", &self.signature())
            .field("length", &self.length())
            .field("revision", &self.revision)
            .field("oem_id", &self.oem_id())
            .field("oem_table_id", &self.oem_table_id())
            .field("oem_revision", &rev)
            .field("creator_id", &self.creator_id())
            .field("creator_revision", &cr_rev)
            .finish()
    }
}
