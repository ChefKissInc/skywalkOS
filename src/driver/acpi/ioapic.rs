//! Copyright (c) ChefKiss Inc 2021-2022.
//! This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use acpi::tables::madt::ic::ioapic::{self, IOAPICRedir};
use amd64::spec::mps::{Polarity, TriggerMode};
use log::debug;

pub fn wire_legacy_irq(irq: u8, masked: bool) {
    let madt = unsafe { (*crate::sys::state::SYS_STATE.get()).madt.assume_init_mut() };
    madt.isos.iter().find(|v| v.irq == irq).map_or_else(
        || {
            let ioapic = find_for_gsi(0).unwrap();
            debug!("Setting up legacy irq {} on I/O APIC {}", irq, ioapic.id);
            ioapic.write_redir(
                irq as _,
                IOAPICRedir::new()
                    .with_vector(irq + 0x20)
                    .with_masked(masked),
            );
        },
        |v| {
            let ioapic = find_for_gsi(v.gsi).unwrap_or_else(|| find_for_gsi(0).unwrap());
            let gsi = v.gsi;
            debug!(
                "Setting up legacy irq {} on I/O APIC {} at gsi {}",
                irq, ioapic.id, gsi
            );
            ioapic.write_redir(
                v.gsi - ioapic.gsi_base,
                IOAPICRedir::new()
                    .with_vector(irq + 0x20)
                    .with_active_high(v.flags.polarity() == Polarity::ActiveHigh)
                    .with_trigger_at_level(v.flags.trigger_mode() == TriggerMode::LevelTriggered)
                    .with_masked(masked),
            );
        },
    )
}

pub fn find_for_gsi(gsi: u32) -> Option<&'static ioapic::IOAPIC> {
    unsafe { (*crate::sys::state::SYS_STATE.get()).madt.assume_init_mut() }
        .ioapics
        .iter()
        .find(|ioapic| {
            gsi >= ioapic.gsi_base && gsi < (ioapic.gsi_base + ioapic.read_ver().max_redir() as u32)
        })
        .copied()
}
