// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

use acpi::tables::rsdp::RSDP;
use amd64::paging::pml4::PML4;
use uefi::{
    proto::console::{gop::GraphicsOutput, text::Color},
    table::boot::{OpenProtocolAttributes, OpenProtocolParams, ScopedProtocol},
};

pub fn init_output() {
    unsafe {
        let stdout = uefi_services::system_table().as_mut().stdout();
        stdout.reset(false).expect("Failed to reset stdout");
        let desired_mode = stdout.modes().last().unwrap();
        stdout.set_mode(desired_mode).expect("Failed to set mode");
        stdout
            .set_color(Color::White, Color::Black)
            .expect("Failed to set color");
        stdout.clear().expect("Failed to clear console");
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
        );
    }

    trace!("    2. Enabling SSE...");
    unsafe {
        core::arch::asm!(
            "mov rax, cr0",
            "and rax, {em_bit}",
            "or rax, {mp_bit}",
            "mov cr0, rax",
            "mov rax, cr4",
            "or rax, {osfxsr_osxmmexcpt_bit}",
            "mov cr4, rax",
            em_bit = const !(1u64 << 2),
            mp_bit = const (1u64 << 1),
            osfxsr_osxmmexcpt_bit = const (1u64 << 9) | (1u64 << 10),
        );
    }

    trace!("    2. Modifying paging mappings to map higher-half...");

    unsafe {
        let pml4 = super::PML4::get();
        pml4.map_higher_half();
        pml4.set();
    }
}

pub fn get_gop<'a>() -> ScopedProtocol<'a, GraphicsOutput<'a>> {
    let system_table = unsafe { uefi_services::system_table().as_mut() };
    let handle = system_table
        .boot_services()
        .get_handle_for_protocol::<uefi::proto::console::gop::GraphicsOutput>()
        .unwrap();
    unsafe {
        system_table
            .boot_services()
            .open_protocol(
                OpenProtocolParams {
                    handle,
                    agent: system_table.boot_services().image_handle(),
                    controller: None,
                },
                OpenProtocolAttributes::GetProtocol,
            )
            .expect("Failed to get GOP protocol")
    }
}

pub fn get_rsdp() -> &'static RSDP {
    let mut iter = unsafe { uefi_services::system_table().as_mut().config_table().iter() };
    let rsdp: *const RSDP = iter
        .find(|ent| ent.guid == uefi::table::cfg::ACPI2_GUID)
        .unwrap_or_else(|| {
            iter.find(|ent| ent.guid == uefi::table::cfg::ACPI_GUID)
                .expect("No ACPI found on the system!")
        })
        .address
        .cast();
    super::phys_to_kern_ref(unsafe { &*rsdp })
}
