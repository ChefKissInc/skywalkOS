// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::vec::Vec;

pub mod apic;
pub mod ioapic;
pub mod madt;
pub mod tables;

pub struct Acpi {
    pub version: u8,
    pub tables: Vec<&'static tables::SDTHeader>,
}

impl Acpi {
    #[inline]
    #[must_use]
    pub fn new(rsdp: &'static tables::rsdp::RootSystemDescPtr) -> Self {
        let mut tables = Vec::new();

        for ent in rsdp.as_type().iter() {
            if !ent.validate() {
                debug!("Invalid table: {ent:X?}");
                continue;
            }

            debug!("Table: {ent:#X?}");
            tables.push(ent);
        }

        Self {
            version: rsdp.revision,
            tables,
        }
    }

    pub fn find<T>(&self, signature: &str) -> Option<&'static T> {
        self.tables
            .iter()
            .find(|&a| a.signature() == signature)
            .map(|&v| unsafe { &*(v as *const tables::SDTHeader).cast::<T>() })
    }
}

pub fn get_hpet(state: &mut crate::system::state::SystemState) -> super::timer::hpet::Hpet {
    let acpi = state.acpi.as_ref().unwrap();
    let pml4 = state.pml4.as_mut().unwrap();

    acpi.find("HPET")
        .map(|v| unsafe {
            let addr = v as *const _ as u64;
            pml4.map_mmio(
                addr,
                addr - amd64::paging::PHYS_VIRT_OFFSET,
                1,
                amd64::paging::PageTableEntry::new()
                    .with_present(true)
                    .with_writable(true),
            );
            super::timer::hpet::Hpet::new(v)
        })
        .unwrap()
}
