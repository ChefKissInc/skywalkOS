// Copyright (c) ChefKiss 2021-2025. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

use alloc::boxed::Box;

use uefi::{
    boot::{EventType, TimerTrigger, Tpl},
    proto::console::text::Key,
    Char16,
};

pub fn setup() {
    trace!("Setting up higher-half paging mappings:");
    trace!("    1. Turning off write protection...");
    unsafe {
        core::arch::asm!(
            "mov rax, cr0",
            "and rax, {wp_bit}",
            "mov cr0, rax",
            wp_bit = const !(1u64 << 16),
            options(nostack, preserves_flags),
        );
    }

    trace!("    2. Modifying paging mappings to map higher-half...");
    unsafe {
        amd64::paging::PageTable::<0>::from_cr3().map_higher_half(&|| {
            Box::leak(Box::new(amd64::paging::PageTable::<0>::new())) as *mut _ as u64
        });
    }
}

pub fn check_boot_flags() -> (bool, bool) {
    let timer =
        match unsafe { uefi::boot::create_event(EventType::TIMER, Tpl::CALLBACK, None, None) } {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to create timer: {e}.");
                return (false, false);
            }
        };
    if let Err(e) = uefi::boot::set_timer(&timer, TimerTrigger::Relative(5 * 1000 * 1000)) {
        warn!("Failed to set timer: {e}.");
        uefi::boot::close_event(timer).unwrap();
        return (false, false);
    };
    let mut events = unsafe {
        [
            timer.unsafe_clone(),
            uefi::system::with_stdin(|v| v.wait_for_key_event()).unwrap(),
        ]
    };
    let i = match uefi::boot::wait_for_event(&mut events) {
        Ok(v) => v,
        Err(e) => {
            warn!("Failed to wait for event: {e}.");
            uefi::boot::close_event(timer).unwrap();
            return (false, false);
        }
    };

    uefi::boot::close_event(timer).unwrap();
    if i == 0 {
        return (false, false);
    }

    uefi::system::with_stdin(|stdin| {
        let mut verbose = false;
        let mut serial_enabled = false;
        while let Ok(v) = stdin.read_key() {
            match v {
                Some(Key::Printable(v)) if v == Char16::try_from('v').unwrap() => {
                    verbose = true;
                    break;
                }
                Some(Key::Printable(v)) if v == Char16::try_from('s').unwrap() => {
                    serial_enabled = true;
                    break;
                }
                _ => {}
            }
        }
        (verbose, serial_enabled)
    })
}

pub fn get_rsdp() -> *const u8 {
    uefi::system::with_config_table(|cfg_table| {
        let mut iter = cfg_table.iter();
        let rsdp: *const u8 = iter
            .find(|ent| ent.guid == uefi::table::cfg::ACPI2_GUID)
            .unwrap_or_else(|| {
                iter.find(|ent| ent.guid == uefi::table::cfg::ACPI_GUID)
                    .unwrap()
            })
            .address
            .cast();
        super::pa_to_kern_va(rsdp)
    })
}
