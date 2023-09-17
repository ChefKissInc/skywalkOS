// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::vec::Vec;

use amd64::paging::PageTableFlags;

use self::tables::hpet::Hpet;

pub mod apic;
pub mod ioapic;
pub mod madt;
pub mod tables;

pub struct ACPIState {
    pub version: u8,
    pub tables: Vec<&'static tables::SDTHeader>,
}

impl ACPIState {
    #[inline]
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

pub fn get_hpet(state: &crate::system::state::SystemState) -> super::timer::hpet::Hpet {
    let acpi = state.acpi.as_ref().unwrap();
    let pml4 = state.pml4.as_ref().unwrap();

    acpi.find("HPET")
        .map(|v: &Hpet| unsafe {
            pml4.lock().map_mmio(
                v.address.address + amd64::paging::PHYS_VIRT_OFFSET,
                v.address.address,
                1,
                PageTableFlags::new_present().with_writable(true),
            );
            super::timer::hpet::Hpet::new(v)
        })
        .unwrap()
}
