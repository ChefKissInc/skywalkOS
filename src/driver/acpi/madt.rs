//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use alloc::vec::Vec;

use acpi::tables::madt::ic::{
    ioapic::{IoApic, Iso},
    lapic::LocalApic,
    InterruptController,
};
use log::info;

pub struct Madt {
    pub lapics: Vec<&'static LocalApic>,
    pub ioapics: Vec<&'static IoApic>,
    pub isos: Vec<&'static Iso>,
}

impl Madt {
    pub fn new(madt: &'static acpi::tables::madt::Madt) -> Self {
        // Disable PIC
        unsafe {
            amd64::io::port::Port::<u8>::new(0xA1).write(0xFF);
            amd64::io::port::Port::<u8>::new(0x21).write(0xFF);
        }

        let mut lapics = Vec::new();
        let mut ioapics = Vec::new();
        let mut isos = Vec::new();

        for ent in madt.into_iter() {
            match ent {
                InterruptController::LocalApic(lapic) => {
                    info!("Local APIC: {:#X?}", lapic);
                    lapics.push(lapic);
                }
                InterruptController::IoApic(ioapic) => {
                    info!("I/O APIC: {:#X?}", ioapic);
                    info!("I/O APIC ver: {:#X?}", ioapic.read_ver());
                    ioapics.push(ioapic);
                }
                InterruptController::Iso(iso) => {
                    info!("Interrupt Source Override: {:#X?}", iso);
                    isos.push(iso);
                }
                _ => {}
            }
        }

        Self {
            lapics,
            ioapics,
            isos,
        }
    }
}
