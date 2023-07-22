// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

use amd64::paging::pml4::PML4;
use uefi::{
    proto::console::{
        gop::GraphicsOutput,
        text::{Color, Key},
    },
    table::boot::{
        EventType, OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol, TimerTrigger, Tpl,
    },
    Char16,
};

pub fn init_output() {
    unsafe {
        let stdout = uefi_services::system_table().as_mut().stdout();
        stdout.reset(false).unwrap();
        let desired_mode = stdout
            .modes()
            .max_by_key(|v| (v.columns(), v.rows()))
            .unwrap();
        stdout.set_mode(desired_mode).unwrap();
        stdout.set_color(Color::White, Color::Black).unwrap();
        stdout.clear().unwrap();
    }
}

pub fn setup() {
    trace!("Setting up higher-half paging mappings:");
    trace!("    1. Turning off write protection...");
    unsafe {
        core::arch::asm!(
            "mov rax, cr0",
            "and rax, {wp_bit}",
            "mov cr0, rax",
            wp_bit = const !(1u64 << 16),
            options(nostack, preserves_flags, nomem),
        );
    }

    trace!("    2. Modifying paging mappings to map higher-half...");
    unsafe { super::PML4::get().map_higher_half() }
}

pub fn get_gop<'a>() -> ScopedProtocol<'a, GraphicsOutput> {
    let boot_services = unsafe { uefi_services::system_table().as_mut().boot_services() };
    let handle = boot_services
        .get_handle_for_protocol::<uefi::proto::console::gop::GraphicsOutput>()
        .unwrap();
    unsafe {
        boot_services
            .open_protocol(
                OpenProtocolParams {
                    handle,
                    agent: boot_services.image_handle(),
                    controller: None,
                },
                OpenProtocolAttributes::GetProtocol,
            )
            .unwrap()
    }
}

pub fn check_for_verbose() -> bool {
    let system_table = unsafe { uefi_services::system_table().as_mut() };
    let timer = match unsafe {
        system_table
            .boot_services()
            .create_event(EventType::TIMER, Tpl::CALLBACK, None, None)
    } {
        Ok(v) => v,
        Err(e) => {
            warn!("Failed to create timer: {e}.");
            return false;
        }
    };
    if let Err(e) = system_table
        .boot_services()
        .set_timer(&timer, TimerTrigger::Relative(5 * 1000 * 1000))
    {
        warn!("Failed to set timer: {e}.");
        system_table.boot_services().close_event(timer).unwrap();
        return false;
    };
    let mut events = unsafe {
        [
            timer.unsafe_clone(),
            system_table.stdin().wait_for_key_event().unsafe_clone(),
        ]
    };
    let i = match system_table.boot_services().wait_for_event(&mut events) {
        Ok(v) => v,
        Err(e) => {
            warn!("Failed to wait for event: {e}.");
            system_table.boot_services().close_event(timer).unwrap();
            return false;
        }
    };

    system_table.boot_services().close_event(timer).unwrap();
    if i == 0 {
        return false;
    }

    system_table
        .stdin()
        .read_key()
        .map(|v| v == Some(Key::Printable(Char16::try_from('v').unwrap())))
        .unwrap_or_default()
}

pub fn get_rsdp() -> *const u8 {
    let mut iter = unsafe { uefi_services::system_table().as_mut().config_table().iter() };
    let rsdp: *const u8 = iter
        .find(|ent| ent.guid == uefi::table::cfg::ACPI2_GUID)
        .unwrap_or_else(|| {
            iter.find(|ent| ent.guid == uefi::table::cfg::ACPI_GUID)
                .unwrap()
        })
        .address
        .cast();
    super::pa_to_kern_va(rsdp)
}
