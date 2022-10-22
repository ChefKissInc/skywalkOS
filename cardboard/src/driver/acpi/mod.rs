// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::vec::Vec;

use acpi::tables::SDTHeader;

pub mod apic;
pub mod ioapic;
pub mod madt;

#[derive(Debug)]
pub struct ACPIPlatform {
    pub version: u8,
    pub tables: Vec<&'static SDTHeader>,
}

impl ACPIPlatform {
    pub fn new(rsdp: &'static acpi::tables::rsdp::RSDP) -> Self {
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
            .map(|&v| unsafe { (v as *const SDTHeader).cast::<T>().as_ref().unwrap() })
    }
}
