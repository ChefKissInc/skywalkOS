// Copyright (c) ChefKiss Inc 2021-2022.
// This project is licensed by the Creative Commons Attribution-NoCommercial-NoDerivatives license.

#![no_std]
#![no_main]
#![deny(warnings, clippy::cargo, unused_extern_crates)]
#![feature(abi_efiapi, asm_const, core_intrinsics)]

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate log;

use alloc::{boxed::Box, vec::Vec};

use uefi::{
    prelude::*,
    proto::media::file::{FileAttribute, FileMode},
};

mod helpers;

#[used]
#[no_mangle]
static __security_cookie: usize = 0x595e9fbd94fda766;

#[no_mangle]
unsafe extern "C" fn __security_check_cookie(cookie: usize) {
    if cookie != __security_cookie {
        core::intrinsics::abort();
    }
}

#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("Failed to initialize utilities");
    helpers::setup::init_output();
    helpers::setup::setup();

    let mut esp = helpers::file::open_esp(image_handle);

    let buffer = helpers::file::load(
        &mut esp,
        cstr16!("\\System\\cardboard.exec"),
        FileMode::Read,
        FileAttribute::empty(),
    )
    .leak();

    let drv_buffer = helpers::file::load(
        &mut esp,
        cstr16!("\\System\\Drivers\\Test.dcext\\test-drv.exec"),
        FileMode::Read,
        FileAttribute::empty(),
    )
    .leak();
    trace!("{:#X?}", drv_buffer.as_ptr());

    let mut mem_mgr = helpers::mem::MemoryManager::new();
    mem_mgr.allocate((drv_buffer.as_ptr() as _, drv_buffer.len() as _));

    let (kernel_main, symbols) = helpers::parse_elf::parse_elf(&mut mem_mgr, buffer);

    let stack = vec![0u8; 0x14000].leak();
    let stack_ptr = unsafe { helpers::pa_to_kern_va(stack.as_ptr()).add(stack.len()) };
    mem_mgr.allocate((stack.as_ptr() as _, stack.len() as _));

    let fbinfo = helpers::phys_to_kern_ref(Box::leak(helpers::fb::fbinfo_from_gop(
        helpers::setup::get_gop(),
    )));
    let rsdp = helpers::setup::get_rsdp();

    let mut boot_info = Box::new(sulphur_dioxide::BootInfo::new(
        symbols.leak(),
        sulphur_dioxide::boot_attrs::BootSettings {
            verbose: cfg!(debug_assertions),
        },
        Some(fbinfo),
        rsdp,
    ));

    let modules = vec![sulphur_dioxide::module::Module {
        name: core::str::from_utf8(helpers::phys_to_kern_slice_ref(b"Text.dcext")).unwrap(),
        data: helpers::phys_to_kern_slice_ref(drv_buffer),
    }];
    boot_info.modules = modules.leak();

    trace!("{:#X?}", boot_info.as_ref() as *const _);
    trace!("Exiting boot services and jumping to kernel...");
    let sizes = system_table.boot_services().memory_map_size();
    let mut mmap_buf = vec![0; sizes.map_size + 4 * sizes.entry_size];
    let mut memory_map_entries = Vec::with_capacity(
        mmap_buf.capacity() / core::mem::size_of::<uefi::table::boot::MemoryDescriptor>() - 2,
    );

    system_table
        .exit_boot_services(image_handle, &mut mmap_buf)
        .expect("Failed to exit boot services.")
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
            "mov rsp, {}",
            "xor rbp, rbp",
            "call {}",
            in(reg) stack_ptr,
            in(reg) kernel_main,
            in("rdi") helpers::phys_to_kern_ref(Box::leak(boot_info)),
            options(noreturn)
        )
    }
}
