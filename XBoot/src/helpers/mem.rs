// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::vec::Vec;

use amd64::paging::PAGE_SIZE;
use sulphur_dioxide::{MemoryData, MemoryEntry};
use uefi::table::boot::{MemoryDescriptor, MemoryType};

pub struct MemoryManager {
    entries: Vec<MemoryData>,
}

impl MemoryManager {
    #[inline]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn allocate(&mut self, ent: (u64, u64)) {
        self.entries.push(MemoryData::new(ent.0, ent.1));
    }

    pub fn mem_type_from_desc(&self, desc: &MemoryDescriptor) -> Option<MemoryEntry> {
        let data = MemoryData::new(desc.phys_start, desc.page_count * PAGE_SIZE);

        match desc.ty {
            MemoryType::CONVENTIONAL => Some(MemoryEntry::Usable(data)),
            MemoryType::LOADER_CODE | MemoryType::LOADER_DATA => {
                let Some(reserved) = self.entries.iter().find(|v| data.base <= v.base + v.length)
                else {
                    return Some(MemoryEntry::BootLoaderReclaimable(data));
                };

                if data.base + data.length > reserved.base + reserved.length {
                    return Some(MemoryEntry::BootLoaderReclaimable(MemoryData::new(
                        data.base + reserved.length,
                        data.length - reserved.length,
                    )));
                }

                None
            }
            MemoryType::ACPI_RECLAIM => Some(MemoryEntry::ACPIReclaimable(data)),
            _ => None,
        }
    }
}
