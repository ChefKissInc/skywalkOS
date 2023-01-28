// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::vec::Vec;

use sulphur_dioxide::{MemoryData, MemoryEntry};

pub struct MemoryManager {
    entries: Vec<(u64, u64)>,
}

impl MemoryManager {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn allocate(&mut self, ent: (u64, u64)) {
        self.entries.push(ent);
    }

    pub fn mem_type_from_desc(
        &self,
        desc: &uefi::table::boot::MemoryDescriptor,
    ) -> Option<MemoryEntry> {
        let mut data = MemoryData {
            base: desc.phys_start,
            length: desc.page_count * 0x1000,
        };

        match desc.ty {
            uefi::table::boot::MemoryType::CONVENTIONAL => Some(MemoryEntry::Usable(data)),
            uefi::table::boot::MemoryType::LOADER_CODE
            | uefi::table::boot::MemoryType::LOADER_DATA => {
                let Some((base, size)) = self.entries.iter()
                    .find(|(base, size)| data.base <= base + size) else {
                    return Some(MemoryEntry::BootLoaderReclaimable(data));
                };
                let top = data.base + data.length;

                if top > base + size {
                    data.length -= size;
                    data.base += size;
                    Some(MemoryEntry::BootLoaderReclaimable(data))
                } else {
                    None
                }
            }
            uefi::table::boot::MemoryType::ACPI_RECLAIM => Some(MemoryEntry::ACPIReclaimable(data)),
            _ => None,
        }
    }
}
