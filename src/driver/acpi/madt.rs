//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use alloc::vec::Vec;

use acpi::tables::madt::ic::{
    ioapic::{IoApic, Iso},
    lapic::LocalApic,
    InterruptController,
};
use log::debug;

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
                    debug!("Found Local APIC: {:#X?}", lapic);
                    lapics.push(lapic);
                }
                InterruptController::IoApic(ioapic) => {
                    debug!(
                        "Found I/O APIC with ver {:#X?}: {:#X?}",
                        ioapic.read_ver(),
                        ioapic,
                    );
                    ioapics.push(ioapic);
                }
                InterruptController::Iso(iso) => {
                    debug!("Found Interrupt Source Override: {:#X?}", iso);
                    isos.push(iso);
                }
                rest => debug!("Dunno how to handle this: {:#X?}", rest),
            }
        }

        Self {
            lapics,
            ioapics,
            isos,
        }
    }
}
