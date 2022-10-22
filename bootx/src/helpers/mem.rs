// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::vec::Vec;

use sulphur_dioxide::mmap::{MemoryData, MemoryEntry};

#[derive(Debug)]
pub struct MemoryManager {
    entries: Vec<(u64, u64)>,
}

impl MemoryManager {
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
                let mut ret = MemoryEntry::BootLoaderReclaimable(data);

                for (base, size) in &self.entries {
                    let top = data.base + data.length;
                    if data.base <= base + size {
                        if top > base + size {
                            data.length -= size;
                            data.base += size;
                            ret = MemoryEntry::BootLoaderReclaimable(data);
                        } else {
                            ret = MemoryEntry::KernelOrModule(data);
                        }

                        break;
                    }
                }
                Some(ret)
            }
            uefi::table::boot::MemoryType::ACPI_RECLAIM => Some(MemoryEntry::ACPIReclaimable(data)),
            _ => None,
        }
    }
}
