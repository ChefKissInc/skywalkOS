// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use alloc::vec::Vec;

use amd64::paging::PageTableEntry;

use super::tables::madt::ic::{
    ioapic::{InputOutputAPIC, IntrSourceOverride},
    proc_lapic::ProcessorLocalAPIC,
    InterruptController,
};

pub struct MADTData {
    pub proc_lapics: Vec<&'static ProcessorLocalAPIC>,
    pub ioapics: Vec<&'static InputOutputAPIC>,
    pub isos: Vec<&'static IntrSourceOverride>,
    pub lapic_addr: u64,
}

impl MADTData {
    #[inline]
    pub fn new(madt: &'static super::tables::madt::MultipleAPICDescTable) -> Self {
        if madt.flags.pcat_compat() {
            crate::intrs::pic::ProgrammableIntrController::new().remap_and_disable();
        }

        let mut proc_lapics = Vec::new();
        let mut ioapics = Vec::new();
        let mut isos = Vec::new();
        let mut lapic_addr = if madt.local_ic_addr() == 0 {
            0xFEE0_0000
        } else {
            madt.local_ic_addr()
        };

        for ent in madt.as_iter() {
            match ent {
                InterruptController::ProcessorLocalAPIC(lapic) => {
                    debug!("{lapic:#X?}");
                    proc_lapics.push(lapic);
                }
                InterruptController::InputOutputAPIC(ioapic) => {
                    debug!("{ioapic:#X?}: {:#X?}", ioapic.read_ver(),);
                    unsafe {
                        (*crate::system::state::SYS_STATE.get())
                            .pml4
                            .as_mut()
                            .unwrap()
                            .map_mmio(
                                u64::from(ioapic.address) + amd64::paging::PHYS_VIRT_OFFSET,
                                u64::from(ioapic.address),
                                1,
                                PageTableEntry::new().with_present(true).with_writable(true),
                            );
                    }
                    ioapics.push(ioapic);
                }
                InterruptController::IntrSourceOverride(iso) => {
                    debug!("{iso:#X?}");
                    isos.push(iso);
                }
                InterruptController::LocalAPICAddrOverride(a) => {
                    debug!("{a:#X?}");
                    lapic_addr = a.addr;
                }
                v => debug!("Ignoring {v:X?}"),
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

pub fn setup(state: &mut crate::system::state::SystemState) {
    let acpi = state.acpi.as_mut().unwrap();
    state.madt = Some(MADTData::new(acpi.find("APIC").unwrap()).into());
}
