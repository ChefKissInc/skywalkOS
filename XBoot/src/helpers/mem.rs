// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::vec::Vec;

use sulphur_dioxide::{MemoryData, MemoryEntry};
use uefi::table::boot::{MemoryDescriptor, MemoryType};

pub struct MemoryManager {
    entries: Vec<(u64, u64)>,
}

impl MemoryManager {
    #[inline]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn allocate(&mut self, ent: (u64, u64)) {
        self.entries.push(ent);
    }

    pub fn mem_type_from_desc(&self, desc: &MemoryDescriptor) -> Option<MemoryEntry> {
        let data = MemoryData::new(desc.phys_start, desc.page_count * 0x1000);

        match desc.ty {
            MemoryType::CONVENTIONAL => Some(MemoryEntry::Usable(data)),
            MemoryType::LOADER_CODE | MemoryType::LOADER_DATA => {
                let Some((base, size)) = self
                    .entries
                    .iter()
                    .find(|(base, size)| data.base <= base + size)
                else {
                    return Some(MemoryEntry::BootLoaderReclaimable(data));
                };
                let top = data.base + data.length;

                if top > base + size {
                    Some(MemoryEntry::BootLoaderReclaimable(MemoryData::new(
                        data.base + size,
                        data.length - size,
                    )))
                } else {
                    None
                }
            }
            MemoryType::ACPI_RECLAIM => Some(MemoryEntry::ACPIReclaimable(data)),
            _ => None,
        }
    }
}
