//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use acpi::tables::SDTHeader;
use hashbrown::HashMap;
use log::debug;

pub mod apic;
pub mod ioapic;
pub mod madt;

#[derive(Debug)]
pub struct ACPIPlatform {
    pub version: u8,
    pub tables: HashMap<&'static str, &'static SDTHeader>,
}

impl ACPIPlatform {
    pub fn new(rsdp: &'static acpi::tables::rsdp::RSDP) -> Self {
        let mut tables = HashMap::new();

        for ent in rsdp.as_type().iter() {
            if !ent.validate() {
                debug!("Invalid table: {:X?}", ent);
                continue;
            }

            debug!(
                "Table: {:#X?}",
                tables.try_insert(ent.signature(), ent).unwrap()
            );
        }

        Self {
            version: rsdp.revision,
            tables,
        }
    }

    pub fn find<T>(&self, signature: &str) -> Option<&'static T> {
        self.tables
            .iter()
            .find(|(&a, _)| a == signature)
            .map(|(_, &v)| unsafe { (v as *const _ as *const T).as_ref().unwrap() })
    }
}
