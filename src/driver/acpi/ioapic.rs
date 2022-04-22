//! Copyright (c) VisualDevelopment 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives licence.

use acpi::tables::madt::ic::ioapic;
use amd64::spec::mps::{Polarity, TriggerMode};
use log::debug;

pub struct IoApic;

impl IoApic {
    pub fn new() -> Self {
        let madt = unsafe { (&*crate::sys::state::SYS_STATE.madt.get()).get().unwrap() };

        for iso in &madt.isos {
            let ioapic = Self::find_for_gsi(iso.gsi).unwrap();
            let redir = ioapic
                .read_redir(iso.gsi - ioapic.gsi_base)
                .with_vector(iso.irq)
                .with_active_high(iso.flags.polarity() == Polarity::ActiveHigh)
                .with_trigger_at_level(iso.flags.trigger_mode() == TriggerMode::LevelTriggered)
                .with_masked(true);
            debug!("{:?}", redir);
            ioapic.write_redir(iso.gsi - ioapic.gsi_base, redir);
        }

        Self
    }

    pub fn find_for_gsi(gsi: u32) -> Option<&'static ioapic::IoApic> {
        unsafe { (&*crate::sys::state::SYS_STATE.madt.get()).get()? }
            .ioapics
            .iter()
            .find(|ioapic| {
                gsi >= ioapic.gsi_base
                    && gsi < (ioapic.gsi_base + ioapic.read_ver().max_redir() as u32)
            })
            .copied()
    }
}
