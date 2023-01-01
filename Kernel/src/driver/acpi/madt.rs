// Copyright (c) ChefKiss Inc 2021-2023.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use alloc::vec::Vec;

use acpi::tables::madt::ic::{
    ioapic::{InterruptSourceOverride, IOAPIC},
    proc_lapic::ProcessorLocalAPIC,
    InterruptController,
};

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
        let mut lapic_addr = if madt.local_ic_addr() != 0 {
            madt.local_ic_addr()
        } else {
            0xFEE0_0000
        };

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

pub fn setup(state: &mut crate::sys::state::SystemState) {
    let acpi = state.acpi.get_mut().unwrap();
    state
        .madt
        .call_once(|| spin::Mutex::new(MADTData::new(acpi.find("APIC").unwrap())));
}
