// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use amd64::spec::mps::{Polarity, TriggerMode};

use super::tables::madt::ic::ioapic::{IOAPICRedir, InputOutputAPIC};

pub fn wire_legacy_irq(irq: u8, masked: bool) {
    let state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
    let madt = state.madt.as_ref().unwrap().lock();
    madt.isos.iter().find(|v| v.irq == irq).map_or_else(
        || {
            let ioapic = find_for_gsi(&madt, 0).unwrap();
            trace!("Routing legacy irq {irq} to I/O APIC {}", ioapic.id);
            ioapic.write_redir(
                u32::from(irq),
                IOAPICRedir::new()
                    .with_vector(irq + 0x20)
                    .with_masked(masked),
            );
        },
        |v| {
            let ioapic =
                find_for_gsi(&madt, v.gsi).unwrap_or_else(|| find_for_gsi(&madt, 0).unwrap());
            let gsi = v.gsi;
            trace!(
                "Routing legacy irq {irq} to I/O APIC {} at gsi {gsi}",
                ioapic.id
            );
            let flags = v.flags;
            ioapic.write_redir(
                v.gsi - ioapic.gsi_base,
                IOAPICRedir::new()
                    .with_vector(irq + 0x20)
                    .with_active_high(flags.polarity() == Polarity::ActiveHigh)
                    .with_trigger_at_level(flags.trigger_mode() == TriggerMode::LevelTriggered)
                    .with_masked(masked),
            );
        },
    );
}

pub fn set_irq_mask(irq: u8, masked: bool) {
    let state = unsafe { &mut *crate::system::state::SYS_STATE.get() };
    let madt = state.madt.as_ref().unwrap().lock();
    madt.isos.iter().find(|v| v.irq == irq).map_or_else(
        || {
            let ioapic = find_for_gsi(&madt, 0).unwrap();
            ioapic.write_redir(
                u32::from(irq),
                ioapic.read_redir(u32::from(irq)).with_masked(masked),
            );
        },
        |v| {
            let ioapic =
                find_for_gsi(&madt, v.gsi).unwrap_or_else(|| find_for_gsi(&madt, 0).unwrap());
            ioapic.write_redir(
                v.gsi - ioapic.gsi_base,
                ioapic
                    .read_redir(v.gsi - ioapic.gsi_base)
                    .with_masked(masked),
            );
        },
    );
}

pub fn find_for_gsi(madt: &super::madt::MADTData, gsi: u32) -> Option<&'static InputOutputAPIC> {
    madt.ioapics
        .iter()
        .find(|ioapic| {
            gsi >= ioapic.gsi_base
                && gsi < (ioapic.gsi_base + u32::from(ioapic.read_ver().max_redir()))
        })
        .copied()
}
