// Copyright (c) ChefKiss Inc 2021-2023. Licensed under the Thou Shalt Not Profit License version 1.0. See LICENSE for details.

#![no_std]
#![no_main]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![allow(clippy::multiple_crate_versions)]
#![feature(asm_const, core_intrinsics)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

use alloc::{boxed::Box, vec::Vec};

use uefi::{
    prelude::*,
    proto::console::text::Key,
    table::boot::{EventType, TimerTrigger, Tpl},
    Char16,
};

mod helpers;

#[export_name = "efi_main"]
extern "efiapi" fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    unsafe { system_table.boot_services().set_image_handle(image_handle) }
    if let Err(e) = system_table
        .boot_services()
        .set_watchdog_timer(0, 0x10000, None)
    {
        warn!("Failed to disarm watchdog timer: {e}.");
    };
    uefi_services::init(&mut system_table).unwrap();
    helpers::setup::init_output();
    helpers::setup::setup();

    let verbose = 'a: {
        let Ok(timer) = (unsafe {
            system_table
                .boot_services()
                .create_event(EventType::TIMER, Tpl::CALLBACK, None, None)
        }) else {
            break 'a false;
        };
        if system_table
            .boot_services()
            .set_timer(&timer, TimerTrigger::Relative(5 * 1000 * 1000))
            .is_err()
        {
            system_table.boot_services().close_event(timer).unwrap();
            break 'a false;
        };
        let mut events = unsafe {
            [
                timer.unsafe_clone(),
                system_table.stdin().wait_for_key_event().unsafe_clone(),
            ]
        };
        let Ok(index) = system_table.boot_services().wait_for_event(&mut events) else {
            system_table.boot_services().close_event(timer).unwrap();
            break 'a false;
        };

        system_table.boot_services().close_event(timer).unwrap();
        index != 0
            && system_table
                .stdin()
                .read_key()
                .map(|v| v == Some(Key::Printable(Char16::try_from('v').unwrap())))
                .unwrap_or_default()
    };

    let (kernel_buf, tkcache_buf) = {
        let mut esp = system_table
            .boot_services()
            .get_image_file_system(image_handle)
            .unwrap();
        (
            esp.read(cstr16!("\\System\\Kernel.exec")).unwrap().leak(),
            esp.read(cstr16!("\\System\\Extensions.tkcache"))
                .unwrap()
                .leak(),
        )
    };

    let mut mem_mgr = helpers::mem::MemoryManager::new();
    mem_mgr.allocate((tkcache_buf.as_ptr() as _, tkcache_buf.len() as _));

    let (kernel_main, symbols) = helpers::elf::parse(&mut mem_mgr, kernel_buf);

    let stack = vec![0u8; 0x14000].leak();
    let stack_ptr = unsafe { helpers::pa_to_kern_va(stack.as_ptr()).add(stack.len()) };
    mem_mgr.allocate((stack.as_ptr() as _, stack.len() as _));

    let gop = helpers::setup::get_gop();
    let fbinfo = helpers::phys_to_kern_ref(Box::leak(helpers::fb::fbinfo_from_gop(gop)));
    let rsdp = helpers::setup::get_rsdp();

    let boot_info = Box::leak(Box::new(sulphur_dioxide::BootInfo::new(
        symbols.leak(),
        verbose,
        Some(fbinfo),
        rsdp,
        helpers::phys_to_kern_slice_ref(tkcache_buf),
    )));

    trace!("Exiting boot services and jumping to kernel...");
    let sizes = system_table.boot_services().memory_map_size();
    let mut memory_map_entries = Vec::with_capacity(sizes.map_size / sizes.entry_size + 8);

    system_table.exit_boot_services().1.entries().for_each(|v| {
        if let Some(v) = mem_mgr.mem_type_from_desc(v) {
            memory_map_entries.push(v);
        }
    });
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
            options(nostack, preserves_flags, noreturn),
        );
    }
}
