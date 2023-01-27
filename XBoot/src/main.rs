// Copyright (c) ChefKiss Inc 2021-2023. All rights reserved.

#![no_std]
#![no_main]
#![deny(warnings, clippy::cargo, clippy::nursery, unused_extern_crates)]
#![feature(asm_const, core_intrinsics)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

use alloc::{boxed::Box, vec::Vec};

use uefi::{
    prelude::*,
    proto::{
        console::text::Key,
        media::file::{FileAttribute, FileMode},
    },
    table::boot::{EventType, TimerTrigger, Tpl},
    Char16,
};

mod helpers;

#[used]
#[no_mangle]
static __security_cookie: usize = 0x595E_9FBD_94FD_A766;

#[no_mangle]
unsafe extern "C" fn __security_check_cookie(cookie: usize) {
    if cookie != __security_cookie {
        core::intrinsics::abort();
    }
}

#[export_name = "efi_main"]
extern "efiapi" fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    unsafe { system_table.boot_services().set_image_handle(image_handle) }
    system_table
        .boot_services()
        .set_watchdog_timer(0, 0x10000, None)
        .unwrap();
    system_table.stdout().reset(false).unwrap();
    system_table.stdin().reset(false).unwrap();
    uefi_services::init(&mut system_table).unwrap();
    helpers::setup::init_output();
    helpers::setup::setup();

    let verbose = {
        let timer = unsafe {
            system_table
                .boot_services()
                .create_event(EventType::TIMER, Tpl::CALLBACK, None, None)
                .unwrap()
        };
        system_table
            .boot_services()
            .set_timer(&timer, TimerTrigger::Relative(5 * 1000 * 1000))
            .unwrap();
        let mut events = unsafe {
            [
                timer.unsafe_clone(),
                system_table.stdin().wait_for_key_event().unsafe_clone(),
            ]
        };
        let index = system_table
            .boot_services()
            .wait_for_event(&mut events)
            .unwrap();

        system_table.boot_services().close_event(timer).unwrap();
        index != 0
            && system_table.stdin().read_key().unwrap()
                == Some(Key::Printable(Char16::try_from('v').unwrap()))
    };

    let mut esp = helpers::file::open_esp(image_handle);

    let kernel_buffer = helpers::file::load(
        &mut esp,
        cstr16!("\\System\\Kernel.exec"),
        FileMode::Read,
        FileAttribute::empty(),
    )
    .leak();

    let dc_ext_buffer = helpers::file::load(
        &mut esp,
        cstr16!("\\System\\Extensions.dccache"),
        FileMode::Read,
        FileAttribute::empty(),
    )
    .leak();

    let mut mem_mgr = helpers::mem::MemoryManager::new();
    mem_mgr.allocate((dc_ext_buffer.as_ptr() as _, dc_ext_buffer.len() as _));

    let (kernel_main, symbols) = helpers::parse_elf::parse_elf(&mut mem_mgr, kernel_buffer);

    let stack = vec![0u8; 0x14000].leak();
    let stack_ptr = unsafe { helpers::pa_to_kern_va(stack.as_ptr()).add(stack.len()) };
    mem_mgr.allocate((stack.as_ptr() as _, stack.len() as _));

    let gop = helpers::setup::get_gop();
    let fbinfo = helpers::phys_to_kern_ref(Box::leak(helpers::fb::fbinfo_from_gop(gop)));
    let rsdp = helpers::setup::get_rsdp();

    let mut boot_info = Box::leak(Box::new(sulphur_dioxide::BootInfo::new(
        symbols.leak(),
        sulphur_dioxide::BootSettings { verbose },
        Some(fbinfo),
        rsdp,
        helpers::phys_to_kern_slice_ref(dc_ext_buffer),
    )));

    trace!("Exiting boot services and jumping to kernel...");
    let sizes = system_table.boot_services().memory_map_size();
    let mut mmap_buf = vec![0; sizes.map_size + 4 * sizes.entry_size];
    let mut memory_map_entries = Vec::with_capacity(sizes.map_size / sizes.entry_size + 2);

    system_table
        .exit_boot_services(image_handle, &mut mmap_buf)
        .unwrap()
        .1
        .for_each(|v| {
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
