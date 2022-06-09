//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use alloc::vec::Vec;

use acpi::tables::madt::ic::{
    ioapic::{InterruptSourceOverride, IOAPIC},
    proc_lapic::ProcessorLocalAPIC,
    InterruptController,
};
use log::trace;

pub struct MADTData {
    pub proc_lapics: Vec<&'static ProcessorLocalAPIC>,
    pub ioapics: Vec<&'static IOAPIC>,
    pub isos: Vec<&'static InterruptSourceOverride>,
    pub lapic_addr: u64,
}

impl MADTData {
    pub fn new(madt: &'static acpi::tables::madt::MADT) -> Self {
        // Disable PIC
        if madt.flags.pcat_compat() {
            crate::driver::intrs::pic::ProgrammableInterruptController::new().remap_and_disable();
        }

        let mut proc_lapics = Vec::new();
        let mut ioapics = Vec::new();
        let mut isos = Vec::new();
        let mut lapic_addr = madt.local_ic_addr();

        for ent in madt.into_iter() {
            match ent {
                InterruptController::ProcessorLocalAPIC(lapic) => {
                    trace!("Found Local APIC: {:#X?}", lapic);
                    proc_lapics.push(lapic);
                }
                InterruptController::InputOutputAPIC(ioapic) => {
                    trace!(
                        "Found I/O APIC with ver {:#X?}: {:#X?}",
                        ioapic.read_ver(),
                        ioapic,
                    );
                    ioapics.push(ioapic);
                }
                InterruptController::InterruptSourceOverride(iso) => {
                    trace!("Found Interrupt Source Override: {:#X?}", iso);
                    isos.push(iso);
                }
                InterruptController::LocalAPICAddrOverride(a) => {
                    trace!("Found Local APIC Address Override: {:#X?}", a);
                    lapic_addr = a.addr;
                }
                rest => trace!("Ignoring {:X?}", rest),
            }
        }

        Self {
            proc_lapics,
            ioapics,
            isos,
            lapic_addr,
        }
    }
}
