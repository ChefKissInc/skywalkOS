// Copyright (c) ChefKiss Inc 2021-2024. Licensed under the Thou Shalt Not Profit License version 1.5. See LICENSE for details.

#![no_std]
#![no_main]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![allow(clippy::multiple_crate_versions)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

use alloc::{boxed::Box, vec::Vec};

use uefi::{boot::MemoryType, mem::memory_map::MemoryMap, prelude::*};

mod helpers;

#[export_name = "efi_main"]
extern "efiapi" fn efi_main(image: Handle, st: *const core::ffi::c_void) -> Status {
    unsafe {
        uefi::boot::set_image_handle(image);
        uefi::table::set_system_table(st.cast());
    }
    uefi::helpers::init().unwrap();
    if let Err(e) = uefi::boot::set_watchdog_timer(0, 0x10000, None) {
        warn!("Failed to disarm watchdog timer: {e}.");
    }
    let fb_info = helpers::fb::init();
    helpers::setup::setup();

    let (verbose, serial_enabled) = helpers::setup::check_boot_flags();

    let (kernel_buf, fkcache_buf) = {
        let mut esp = uefi::fs::FileSystem::new(uefi::boot::get_image_file_system(image).unwrap());
        (
            esp.read(cstr16!("\\System\\Sky.exec")).unwrap().leak(),
            esp.read(cstr16!("\\System\\SkyKitExtensions"))
                .unwrap()
                .leak(),
        )
    };

    let mut mem_mgr = helpers::mem::MemoryManager::new();
    mem_mgr.allocate((fkcache_buf.as_ptr() as _, fkcache_buf.len() as _));

    let (kernel_main, symbols) = helpers::elf::parse(&mut mem_mgr, kernel_buf);

    let stack = vec![0u8; 0x14000].leak();
    let stack_ptr = unsafe { helpers::pa_to_kern_va(stack.as_ptr()).add(stack.len()) };
    mem_mgr.allocate((stack.as_ptr() as _, stack.len() as _));

    let boot_info = Box::leak(Box::new(skyliftkit::BootInfo::new(
        symbols.leak(),
        verbose,
        serial_enabled,
        fb_info.map(|v| helpers::phys_to_kern_ref(Box::leak(v))),
        helpers::setup::get_rsdp(),
        helpers::phys_to_kern_slice_ref(fkcache_buf),
    )));

    trace!("Exiting boot services and jumping to kernel...");
    let memory_map_entry_count = uefi::boot::memory_map(MemoryType::LOADER_DATA)
        .unwrap()
        .len()
        + 8;
    let mut memory_map_entries = Vec::with_capacity(memory_map_entry_count);

    for v in unsafe { uefi::boot::exit_boot_services(MemoryType::LOADER_DATA).entries() } {
        if let Some(v) = mem_mgr.mem_type_from_desc(v) {
            memory_map_entries.push(v);
        }
    }
    boot_info.memory_map = helpers::phys_to_kern_slice_ref(memory_map_entries.leak());

    unsafe {
        core::arch::asm!(
            "cli",
            "cld",
            "mov rsp, {}",
            "xor rbp, rbp",
            "call {}",
            in(reg) stack_ptr,
            in(reg) kernel_main,
            in("rdi") helpers::phys_to_kern_ref(boot_info),
            options(nostack, noreturn),
        );
    }
}
